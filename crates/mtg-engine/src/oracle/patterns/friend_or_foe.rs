//! "For each player, choose friend or foe. Each friend searches their library for a land
//! card, puts it onto the battlefield tapped, then shuffles. Each foe sacrifices an
//! artifact or enchantment of their choice." (Pir's Whim): see `kw/friend_or_foe.rs`.
//! "Each friend [instruction]" is read as "each player [instruction]" for the players
//! called friends; it's understood for instructions whose performer is a player reference
//! (sacrificing, searching, drawing, discarding, gaining or losing life).

use super::EffectPattern;
use crate::ability::*;
use crate::kw::friend_or_foe::{CHOOSE, FOES, FRIENDS};
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

fn choose_friend_or_foe(l: &str, _b: &mut Builder) -> Option<Effect> {
    (end(l) == "for each player, choose friend or foe").then(|| Effect::Custom(CHOOSE.into()))
}

inventory::submit! { EffectPattern { name: "for each player, choose friend or foe", priority: 100, parse: choose_friend_or_foe } }

/// Makes the players who perform `e` (each player) the players in `var`.
fn retarget(e: &mut Effect, var: Var) -> bool {
    let each = |who: &mut PlayerRef| {
        if matches!(who, PlayerRef::EachPlayer) {
            *who = PlayerRef::Var(var);
            true
        } else {
            false
        }
    };
    match e {
        Effect::Seq(v) => !v.is_empty() && v.iter_mut().all(|x| retarget(x, var)),
        Effect::Sacrifice { who, .. }
        | Effect::Search { who, .. }
        | Effect::Draw { who, .. }
        | Effect::Discard { who, .. }
        | Effect::DiscardHand { who }
        | Effect::GainLife { who, .. }
        | Effect::LoseLife { who, .. } => each(who),
        _ => false,
    }
}

/// Replaces every "each player" in `e` with the players in `var`; the number replaced.
fn retarget_all(e: &Effect, var: Var) -> Option<(Effect, usize)> {
    fn walk(v: serde_json::Value, from: &serde_json::Value, to: &serde_json::Value, n: &mut usize) -> serde_json::Value {
        use serde_json::Value as J;
        if v == *from {
            *n += 1;
            return to.clone();
        }
        match v {
            J::Array(a) => J::Array(a.into_iter().map(|x| walk(x, from, to, n)).collect()),
            J::Object(m) => J::Object(m.into_iter().map(|(k, x)| (k, walk(x, from, to, n))).collect()),
            x => x,
        }
    }
    let from = serde_json::to_value(PlayerRef::EachPlayer).ok()?;
    let to = serde_json::to_value(PlayerRef::Var(var)).ok()?;
    let mut n = 0;
    let v = walk(serde_json::to_value(e).ok()?, &from, &to, &mut n);
    Some((serde_json::from_value(v).ok()?, n))
}

fn each_friend_or_foe(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    // "each friend [instruction]", or "each foe" elsewhere in the instruction ("~ deals
    // damage to each foe equal to the number of cards in their hand").
    let (var, text) = if let Some(r) = l.strip_prefix("each friend ") {
        (FRIENDS, format!("each player {r}"))
    } else if let Some(r) = l.strip_prefix("each foe ") {
        (FOES, format!("each player {r}"))
    } else if l.matches("each foe").count() == 1 && !l.contains("friend") {
        (FOES, l.replacen("each foe", "each player", 1))
    } else if l.matches("each friend").count() == 1 && !l.contains("foe") {
        (FRIENDS, l.replacen("each friend", "each player", 1))
    } else {
        return None;
    };
    if l.contains("each player") || l.contains("each opponent") {
        return None;
    }
    let targets = b.targets.len();
    let parsed = crate::oracle::effects::parse_clause(&text, b);
    let Some(mut e) = parsed.filter(|_| b.targets.len() == targets) else {
        b.targets.truncate(targets);
        return None;
    };
    if !retarget(&mut e, var) {
        // Any instruction for "each player": every "each player" in it is those players.
        let (x, n) = retarget_all(&e, var)?;
        if n == 0 {
            return None;
        }
        e = x;
    }
    // Only if there's such a player (an instruction with no player to perform it does
    // nothing).
    Some(Effect::If {
        cond: Condition::Compare(
            Value::CountSel(Box::new(Sel::Var(var))),
            Cmp::Gt,
            Value::c(0),
        ),
        then: Box::new(e),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "each friend/foe [instruction]", priority: 100, parse: each_friend_or_foe } }
