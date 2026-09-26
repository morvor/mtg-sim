//! Oracle text that goes with the keywords of CR 702.125–702.139:
//!
//! * mentor (CR 702.134c): "whenever ~ mentors a creature", "whenever equipped creature
//!   mentors a creature".

use super::TriggerPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use smol_str::SmolStr;

// ---------------------------------------------------------------------------
// Mentor (CR 702.134)
// ---------------------------------------------------------------------------

/// "~ mentors a creature" / "equipped creature mentors a creature": a mentor ability of
/// that creature resolved targeting a creature, which is "that creature" (CR 702.134c).
fn mentors(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    let name = match end(r) {
        "~ mentors a creature" | "this creature mentors a creature" => crate::kw::mentor::MENTORS,
        "equipped creature mentors a creature" => crate::kw::mentor::EQUIPPED_MENTORS,
        _ => return None,
    };
    Some((
        TriggerCond::Custom(SmolStr::new(name)),
        Sel::TriggerObject,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "k702.134 mentors a creature", priority: 100, parse: mentors } }
