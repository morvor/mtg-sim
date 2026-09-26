//! Oracle patterns for trigger conditions that look back in time (CR 603.10): becoming
//! unattached, losing control, phasing out; and "sacrifice that permanent".

use super::{EffectPattern, TriggerPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;

fn lookback_triggers(r: &str) -> Option<(TriggerCond, Sel, PlayerRef)> {
    match r {
        // CR 603.10c: "Whenever this Equipment becomes unattached from a permanent".
        "~ becomes unattached from a permanent" => Some((
            TriggerCond::BecomesUnattached(Filter::Source),
            Sel::TriggerObject,
            PlayerRef::ControllerOf(Box::new(Sel::TriggerObject)),
        )),
        // CR 603.10d.
        "you lose control of ~" => Some((
            TriggerCond::LoseControl(Filter::Source),
            Sel::This,
            PlayerRef::You,
        )),
        // CR 603.10b.
        "~ phases out" => Some((
            TriggerCond::PhasesOut(Filter::Source),
            Sel::This,
            PlayerRef::You,
        )),
        _ => None,
    }
}

/// "sacrifice that permanent" / "sacrifice it": its controller sacrifices it.
fn sacrifice_that(l: &str, b: &mut Builder) -> Option<Effect> {
    match l {
        "sacrifice that permanent" | "sacrifice it" | "sacrifice that creature" => {
            Some(Effect::SacrificeObjects { what: b.it.clone() })
        }
        _ => None,
    }
}

inventory::submit! { TriggerPattern { name: "look-back triggers", priority: 0, parse: lookback_triggers } }
inventory::submit! { EffectPattern { name: "sacrifice that permanent", priority: 0, parse: sacrifice_that } }
