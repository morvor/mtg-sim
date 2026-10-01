//! "you get an amount of {E} equal to [value]" (CR 107.14, 122.1): the player gets that
//! many energy counters, the number read as the instruction is performed ("Counter target
//! spell. You get an amount of {E} equal to its mana value.", "you get an amount of {E}
//! equal to the number of creatures you control").

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;
use crate::oracle::statics::parse_value_phrase;
use crate::types::counters;

fn get_energy_equal_to(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l)
        .strip_prefix("you get an amount of {e} equal to ")?
        .trim();
    let (v, tail) = parse_value_phrase(r, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    Some(Effect::AddPlayerCounters {
        who: PlayerRef::You,
        kind: counters::ENERGY.into(),
        n: Value::Max(Box::new(v), Box::new(Value::c(0))),
    })
}

inventory::submit! { EffectPattern { name: "r107 get an amount of energy equal to", priority: 70, parse: get_energy_equal_to } }
