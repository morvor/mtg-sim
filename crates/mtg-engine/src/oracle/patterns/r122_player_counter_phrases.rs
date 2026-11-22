//! Counters players have (CR 122.1): "target player gets four rad counters", "target
//! opponent gets two poison counters", and the conditions "defending player has a rad
//! counter" ("~ can't be blocked as long as defending player has a rad counter") and
//! "defending player is poisoned".

use super::{ConditionPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

/// The kinds of counters players get from effects.
fn player_counter_kind(w: &str) -> bool {
    matches!(w, "poison" | "experience" | "rad" | "ticket")
}

/// "[n] [kind] counter(s)": the number and kind.
fn n_player_counters(s: &str) -> Option<(Value, &str)> {
    let (n, r) = parse_number(s)?;
    let (kind, r) = split_word(r);
    if !player_counter_kind(kind) || !matches!(end(r), "counter" | "counters") {
        return None;
    }
    Some((n, kind))
}

fn target_player_gets_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (who, rest) = ["target player", "target opponent"]
        .into_iter()
        .find_map(|w| Some((w, l.strip_prefix(w)?.strip_prefix(" gets ")?)))?;
    let (n, kind) = n_player_counters(rest)?;
    let (spec, tail) = parse_target(who)?;
    if !end(tail).is_empty() || !matches!(spec.what, TargetKind::Player(_)) {
        return None;
    }
    let slot = b.add_target(spec, who);
    b.it_player = PlayerRef::Target(slot);
    Some(Effect::AddPlayerCounters {
        who: PlayerRef::Target(slot),
        kind: kind.into(),
        n,
    })
}

inventory::submit! { EffectPattern { name: "r122 target player gets N [kind] counters", priority: 60, parse: target_player_gets_counters } }

/// "defending player has a rad counter", "defending player has one or more poison
/// counters", "defending player is poisoned".
fn defending_player_has_counters(c: &str) -> Option<Condition> {
    // CR 122.1f: a poisoned player has one or more poison counters.
    if end(c) == "defending player is poisoned" {
        return Some(Condition::PlayerMatches(
            PlayerRef::DefendingPlayer,
            PlayerFilter::Counters("poison".into(), Cmp::Ge, Box::new(Value::c(1))),
        ));
    }
    let r = end(c).strip_prefix("defending player has ")?;
    let r = r.strip_prefix("one or more ").map_or(r, |x| x);
    let (n, kind) = match n_player_counters(r) {
        Some(x) => x,
        None => {
            // "one or more" was stripped: "[kind] counters".
            let (kind, tail) = split_word(r);
            if !player_counter_kind(kind) || end(tail) != "counters" {
                return None;
            }
            (Value::c(1), kind)
        }
    };
    Some(Condition::PlayerMatches(
        PlayerRef::DefendingPlayer,
        PlayerFilter::Counters(kind.into(), Cmp::Ge, Box::new(n)),
    ))
}

inventory::submit! { ConditionPattern { name: "r122 defending player has [kind] counters", priority: 60, parse: defending_player_has_counters } }
