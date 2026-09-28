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

fn each_friend_or_foe(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (var, rest) = if let Some(r) = l.strip_prefix("each friend ") {
        (FRIENDS, r)
    } else if let Some(r) = l.strip_prefix("each foe ") {
        (FOES, r)
    } else {
        return None;
    };
    let targets = b.targets.len();
    let parsed = crate::oracle::effects::parse_clause(&format!("each player {rest}"), b);
    let Some(mut e) = parsed.filter(|_| b.targets.len() == targets) else {
        b.targets.truncate(targets);
        return None;
    };
    if !retarget(&mut e, var) {
        return None;
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
