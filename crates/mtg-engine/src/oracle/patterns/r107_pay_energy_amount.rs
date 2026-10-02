//! "[effect] unless you pay an amount of {E} equal to [value]" (Volatile Stormdrake:
//! "sacrifice that creature unless you pay an amount of {E} equal to its mana value"):
//! the controller may pay that many energy counters (CR 107.14, 118.12a); if they don't,
//! the effect happens. The amount is determined as the cost is paid.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_simple, Builder};
use crate::oracle::phrases::end;

fn unless_you_pay_energy_amount(l: &str, b: &mut Builder) -> Option<Effect> {
    let (eff, amount) = end(l).rsplit_once(" unless you pay an amount of {e} equal to ")?;
    let effect = parse_simple(eff, b)?;
    let (value, tail) = crate::oracle::statics::parse_value_phrase(amount, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost: Cost {
            mana: None,
            parts: vec![CostPart::PayEnergy(value)],
        },
        then: Box::new(Effect::Noop),
        otherwise: Box::new(effect),
    })
}

inventory::submit! { EffectPattern { name: "r107: effect unless you pay an amount of {E} equal to [value]", priority: 90, parse: unless_you_pay_energy_amount } }
