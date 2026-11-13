//! "[Effect on a target] unless its controller pays [cost]" (CR 118.12a): "Change the
//! target of target spell with a single target unless its controller pays {2}." (Divert).
//! The controller of the targeted object may pay; if they don't, the effect happens.

use super::counters_resources_pay::resolution_cost;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_clause, Builder};
use crate::oracle::phrases::end;

fn unless_its_controller_pays(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let (eff, cost_text) = l.rsplit_once(" unless its controller pays ")?;
    let cost = resolution_cost(cost_text)?;
    let before = b.targets.len();
    let effect = parse_clause(eff, b)?;
    // "its" is the object the instruction targets: exactly one new target slot.
    if b.targets.len() != before + 1 {
        return None;
    }
    let slot = before as u8;
    Some(Effect::PayOptional {
        who: PlayerRef::ControllerOf(Box::new(Sel::Target(slot))),
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(effect),
    })
}

inventory::submit! { EffectPattern { name: "r118 effect unless its controller pays", priority: 100, parse: unless_its_controller_pays } }
