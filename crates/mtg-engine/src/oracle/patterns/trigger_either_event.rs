//! Trigger conditions naming two different events joined by "or": "Whenever you cast a
//! white spell or a Plains you control enters, you gain 1 life." (the Staffs of the
//! Magi), "When a player casts a spell or a creature attacks, ..." (Norin the Wary). The
//! ability triggers on each event that matches either condition (CR 603.2, 603.2c).

use super::TriggerPattern;
use crate::ability::*;

type Parsed = (TriggerCond, Sel, PlayerRef);

/// A complete trigger condition ("you cast a white spell", "a plains you control
/// enters").
fn condition(s: &str) -> Option<Parsed> {
    crate::oracle::triggers::parse_trigger_condition(&format!("whenever {s}"))
}

fn either_event(r: &str) -> Option<Parsed> {
    // "[event] or a[n] [event]": each side must be a whole trigger condition of its own,
    // so an "or" inside one description ("a creature or planeswalker you control dies")
    // isn't split.
    for (i, _) in r.match_indices(" or a") {
        let (a, b) = (&r[..i], &r[i + " or ".len()..]);
        if !(b.starts_with("a ") || b.starts_with("an ")) {
            continue;
        }
        let (Some(pa), Some(pb)) = (condition(a), condition(b)) else {
            continue;
        };
        let batch = super::trigger_grammar_events::is_batched;
        if batch(&pa.0)
            || batch(&pb.0)
            || matches!(pa.0, TriggerCond::AnyOf(_))
            || matches!(pb.0, TriggerCond::AnyOf(_))
        {
            return None;
        }
        let same =
            |x: &dyn std::fmt::Debug, y: &dyn std::fmt::Debug| format!("{x:?}") == format!("{y:?}");
        let it = if same(&pa.1, &pb.1) {
            pa.1.clone()
        } else {
            Sel::None
        };
        let player = if same(&pa.2, &pb.2) {
            pa.2.clone()
        } else {
            PlayerRef::Iterated
        };
        return Some((TriggerCond::AnyOf(vec![pa.0, pb.0]), it, player));
    }
    None
}

inventory::submit! { TriggerPattern { name: "[event] or a[n] [event]", priority: 900, parse: either_event } }
