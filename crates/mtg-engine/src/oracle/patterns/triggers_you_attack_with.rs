//! "Whenever you attack with one or more [creatures]" (Metropolis Angel, Hired Claw): you
//! declared one or more such creatures as attackers (CR 508.1). The same trigger as
//! "whenever one or more [creatures] you control attack", which triggers once for the
//! batch (CR 603.2c).

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::end;

fn you_attack_with(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let what = end(r).strip_prefix("you attack with one or more ")?;
    if what.contains(" you control") || what.contains(" and whenever ") {
        return None;
    }
    // "creatures with counters on them": "creatures you control with counters on them".
    let candidates = [
        Some(format!("whenever one or more {what} you control attack")),
        what.split_once(" with ")
            .map(|(a, b)| format!("whenever one or more {a} you control with {b} attack")),
    ];
    candidates
        .into_iter()
        .flatten()
        .find_map(|l| crate::oracle::triggers::parse_trigger_condition(&l))
        .filter(|(t, _, _)| {
            matches!(t, TriggerCond::Batched { trigger, .. }
                if matches!(**trigger, TriggerCond::Attacks(_)))
        })
}

inventory::submit! { TriggerPattern { name: "you attack with one or more", priority: 100, parse: you_attack_with } }
