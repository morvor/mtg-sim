//! "You may pay X life, where X is the amount of life you gained this turn. If you do,
//! create an X/X black Demon creature token with flying." (Tivash, Gloom Summoner), "Then
//! you may pay {X}, where X is the number of +1/+1 counters on it. If you don't, tap ~
//! and it deals X damage to you." (Primordial Ooze): the text defines X (CR 107.3c), and
//! the X of the following sentences is the same number. X is determined as the payment
//! is offered and keeps that value for the rest of the resolution.

use super::r107_numbers::value_phrase;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;

fn pay_x_where(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let l = l.strip_prefix("then ").unwrap_or(l);
    let (clause, value_s) = l.rsplit_once(", where x is ")?;
    if !clause.starts_with("you may pay ") {
        return None;
    }
    let saved = b.targets.len();
    let parsed = (|| {
        let (v, tail) = value_phrase(value_s, b)?;
        if !end(&tail).is_empty() {
            return None;
        }
        let cost = match clause.strip_prefix("you may pay ")? {
            "x life" => Cost::free().with(CostPart::PayLife(Value::X)),
            "{x}" => Cost::mana(crate::mana::ManaCost::parse("{X}")?),
            _ => return None,
        };
        let pay = Effect::PayOptional {
            who: PlayerRef::You,
            cost,
            then: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Noop),
        };
        // The following sentences' X is this X.
        b.named
            .push((super::tokens_x_x::X_DEFINED.to_string(), Sel::None));
        // CR 107.1b: a negative result is 0.
        let value = Value::Max(Box::new(v), Box::new(Value::c(0)));
        Some(Effect::seq(vec![Effect::SetX { value }, pay]))
    })();
    if parsed.is_none() {
        b.targets.truncate(saved);
    }
    parsed
}

inventory::submit! { EffectPattern { name: "r107 you may pay X ..., where X is [value]", priority: 60, parse: pay_x_where } }
