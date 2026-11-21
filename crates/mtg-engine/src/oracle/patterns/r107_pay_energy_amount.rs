//! "[effect] unless you pay an amount of {E} equal to [value]" (Volatile Stormdrake:
//! "you get {E}{E}{E}{E}, then sacrifice that creature unless you pay an amount of {E}
//! equal to its mana value"): the controller may pay that many energy counters (CR 107.14,
//! 118.12a); if they don't, the effect happens. The amount is determined as the cost is
//! paid. "Unless" governs only the last instruction: anything before ", then" happens
//! first, whatever the player chooses.

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_simple, Builder};
use crate::oracle::phrases::end;

fn unless_you_pay_energy_amount(l: &str, b: &mut Builder) -> Option<Effect> {
    let (eff, amount) = end(l).rsplit_once(" unless you pay an amount of {e} equal to ")?;
    let (before, eff) = match eff.rsplit_once(", then ") {
        Some((before, eff)) => (Some(parse_simple(before, b)?), eff),
        None => (None, eff),
    };
    let effect = parse_simple(eff, b)?;
    let (value, tail) = crate::oracle::statics::parse_value_phrase(amount, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let unless = Effect::PayOptional {
        who: PlayerRef::You,
        cost: Cost {
            mana: None,
            parts: vec![CostPart::PayEnergy(value)],
        },
        then: Box::new(Effect::Noop),
        otherwise: Box::new(effect),
    };
    Some(match before {
        Some(first) => Effect::seq(vec![first, unless]),
        None => unless,
    })
}

inventory::submit! { EffectPattern { name: "r107: effect unless you pay an amount of {E} equal to [value]", priority: 90, parse: unless_you_pay_energy_amount } }
