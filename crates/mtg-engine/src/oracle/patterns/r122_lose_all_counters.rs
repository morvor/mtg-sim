//! "Each opponent loses all counters." (Final Act), "Target opponent loses all counters."
//! (Suncleanser): every counter of every kind the player has is removed (CR 122.1 —
//! energy, experience, poison, rad, ticket counters, and any others).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{player_ref, Builder};
use crate::oracle::phrases::end;

fn loses_all_counters(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let subject = l
        .strip_suffix(" loses all counters")
        .or_else(|| l.strip_suffix(" lose all counters"))?;
    let saved = b.targets.len();
    let Some((who, rest)) = player_ref(subject, b) else {
        b.targets.truncate(saved);
        return None;
    };
    if !rest.trim().is_empty() {
        b.targets.truncate(saved);
        return None;
    }
    Some(Effect::RemoveCounters {
        what: Sel::Players(who),
        kind: None,
        n: Value::Const(i32::MAX),
    })
}

inventory::submit! { EffectPattern { name: "[player] loses all counters", priority: 100, parse: loses_all_counters } }
