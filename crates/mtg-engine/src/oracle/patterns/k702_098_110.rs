//! Oracle text of the keywords of CR 702.98–702.110 that the generic keyword parser
//! doesn't handle, and phrases that go with them:
//!
//! * "Whenever you activate ~'s outlast ability" (CR 702.107a).

use super::TriggerPattern;
use crate::ability::*;

/// "Whenever you activate ~'s outlast ability" (Herald of Anafenza).
fn activate_outlast(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    if r != "you activate ~'s outlast ability" {
        return None;
    }
    Some((
        TriggerCond::Where {
            trigger: Box::new(TriggerCond::AbilityActivated {
                who: PlayerRel::You,
                source: Filter::Source,
                include_mana: false,
            }),
            cond: Condition::Custom(crate::kw::outlast::OUTLAST_ACTIVATED.into()),
        },
        Sel::This,
        PlayerRef::You,
    ))
}

inventory::submit! { TriggerPattern { name: "you activate ~'s outlast ability", priority: 100, parse: activate_outlast } }
