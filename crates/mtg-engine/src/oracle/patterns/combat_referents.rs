//! Combat referents and player phrases of combat:
//!
//! * object phrases: "creatures the active player controls", "non-Wall creature the
//!   active player has controlled continuously since the beginning of the turn" (CR
//!   302.6), "creatures blocking or blocked by target creature" (CR 509.1g);
//! * a player subject shared by two instructions: "the attacking player gains control of
//!   ~ and untaps it".

use super::{EffectPattern, FilterSuffixPattern};
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_clause, player_ref, Builder};
use crate::oracle::phrases::end;

/// "[creatures] the active player controls", "[creature] the active player has
/// controlled continuously since the beginning of the turn".
fn active_player_suffix<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    if let Some(rest) = r.strip_prefix(
        "the active player has controlled continuously since the beginning of the turn",
    ) {
        return Some((
            Filter::and(vec![
                Filter::ControlledByPlayer(Box::new(PlayerRef::ActivePlayer)),
                Filter::ControlledSinceTurnBegan,
            ]),
            rest,
        ));
    }
    let rest = r.strip_prefix("the active player controls")?;
    Some((
        Filter::ControlledByPlayer(Box::new(PlayerRef::ActivePlayer)),
        rest,
    ))
}

inventory::submit! { FilterSuffixPattern { name: "combat referents: the active player controls", priority: 100, parse: active_player_suffix } }

/// "[creatures that player controls] that didn't attack" (Total War, as attackers are
/// declared): not declared as attackers this combat (CR 508.1).
fn didnt_attack_suffix<'a>(r: &'a str, _f: &Filter) -> Option<(Filter, &'a str)> {
    let rest = r.strip_prefix("that didn't attack")?;
    if rest.starts_with(" this turn") || rest.starts_with(" during") {
        return None;
    }
    Some((Filter::not(Filter::AttackedThisCombat), rest))
}

inventory::submit! { FilterSuffixPattern { name: "combat referents: that didn't attack", priority: 100, parse: didnt_attack_suffix } }

/// Verbs (third person) that can continue a player subject after "and".
const PLAYER_VERBS: &[&str] = &[
    "untaps",
    "taps",
    "draws",
    "discards",
    "loses",
    "gains",
    "sacrifices",
    "mills",
    "creates",
    "shuffles",
    "exiles",
    "returns",
    "puts",
    "reveals",
];

/// "[player] gains control of ~ and untaps it": both instructions have the player as
/// their subject.
fn shared_player_subject(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    let (_who, rest) = player_ref(l, b)?;
    let subject = l[..l.len() - rest.len()].trim();
    if subject.is_empty() || subject.contains("target") {
        return None;
    }
    let (first, second) = rest
        .match_indices(" and ")
        .map(|(i, _)| (&rest[..i], &rest[i + " and ".len()..]))
        .find(|(_, s)| {
            s.split(' ')
                .next()
                .is_some_and(|w| PLAYER_VERBS.contains(&w))
        })?;
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let a = parse_clause(&format!("{subject} {}", first.trim()), b);
    // Tapping and untapping don't depend on who does it: "untap it".
    let second = match second.split_once(' ') {
        Some(("untaps", r)) => format!("untap {r}"),
        Some(("taps", r)) => format!("tap {r}"),
        _ => format!("{subject} {second}"),
    };
    // "gains control of ~ and untaps it": "it" is the object just named.
    if first.trim().ends_with(" ~")
        && super::oracle_hardening_referents::is_no_referent(&super::pronoun_groups::singular_it(b))
    {
        b.it = Sel::This;
    }
    let c = a.as_ref().and_then(|_| parse_clause(&second, b));
    match (a, c) {
        (Some(a), Some(c)) => Some(Effect::seq(vec![a, c])),
        _ => {
            b.targets.truncate(saved.0);
            b.it = saved.1;
            b.it_player = saved.2;
            None
        }
    }
}

inventory::submit! { EffectPattern { name: "combat referents: a player subject shared by two instructions", priority: 990, parse: shared_player_subject } }

/// "[all creatures] blocking or blocked by target creature [rest]": the creatures blocking
/// that creature or blocked by it as the effect happens (CR 509.1g).
fn blocking_or_blocked_by_target(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l.trim());
    const PHRASE: &str = " blocking or blocked by target creature";
    let at = l.find(PHRASE)?;
    let (head, tail) = (&l[..at], &l[at + PHRASE.len()..]);
    if head.contains("target") || head.contains('~') || tail.contains("target") {
        return None;
    }
    let saved = (b.targets.len(), b.it.clone(), b.it_player.clone());
    let (t, rest) = object_ref("target creature", b)?;
    let Sel::Target(slot) = t else {
        b.targets.truncate(saved.0);
        return None;
    };
    if !rest.trim().is_empty() {
        b.targets.truncate(saved.0);
        return None;
    }
    // Parse with "~" standing in for the target, then point the relation at the target.
    let e = parse_clause(&format!("{head} blocking or blocked by ~{tail}"), b);
    let ok = e.and_then(|mut e| {
        let mut n = 0;
        retarget(&mut e, slot, &mut n);
        (n == 1 && !mentions_source(&e)).then_some(e)
    });
    if ok.is_none() {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        b.it_player = saved.2;
    }
    ok
}

/// Replaces "blocking or blocked by ~" in an effect's selections with the relation to the
/// target in `slot`, counting the replacements.
fn retarget(e: &mut Effect, slot: u8, n: &mut u32) {
    let mut v = serde_json::to_value(&*e).unwrap_or_default();
    fn walk(v: &mut serde_json::Value, slot: u8, n: &mut u32) {
        let either = serde_json::json!({"Or": ["BlockingSource", "BlockedBySource"]});
        if *v == either {
            *v = serde_json::to_value(Filter::BlockRelation {
                rel: BlockRel::BlockingOrBlockedBy,
                other: Box::new(Sel::Target(slot)),
                when: BlockWhen::Now,
            })
            .unwrap_or_default();
            *n += 1;
            return;
        }
        match v {
            serde_json::Value::Array(a) => a.iter_mut().for_each(|x| walk(x, slot, n)),
            serde_json::Value::Object(o) => o.values_mut().for_each(|x| walk(x, slot, n)),
            _ => {}
        }
    }
    walk(&mut v, slot, n);
    if let Ok(new) = serde_json::from_value(v) {
        *e = new;
    } else {
        *n = 0;
    }
}

/// Whether the effect still refers to the source.
fn mentions_source(e: &Effect) -> bool {
    let s = serde_json::to_string(e).unwrap_or_default();
    s.contains("\"Source\"") || s.contains("BlockingSource") || s.contains("BlockedBySource")
}

inventory::submit! { EffectPattern { name: "combat referents: blocking or blocked by target creature", priority: 80, parse: blocking_or_blocked_by_target } }
