//! "Unless [a player] pays" with a scaled cost: "Counter target spell unless its controller
//! pays {1} for each card in your graveyard.", "... pays {1} plus an additional {1} for
//! each Faerie you control", "... pays {X}, where X is its mana value", and "sacrifice ~
//! unless you pay {1} for each card in your hand".
//!
//! The amount is determined as the payment is proposed, while the spell or ability
//! resolves (CR 118.12, 608.2h); the player may pay it or not (CR 118.12a).

use super::counters_resources_pay::resolution_cost;
use super::statics::parse_for_each;
use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{object_ref, parse_simple, Builder};
use crate::oracle::phrases::*;
use crate::oracle::statics::parse_value_phrase;

inventory::submit! {
    EffectPattern { name: "sweep: counter unless its controller pays a scaled cost", priority: 95, parse: counter_unless_scaled }
}
inventory::submit! {
    EffectPattern { name: "sweep: effect unless you pay a scaled cost", priority: 95, parse: unless_you_pay_scaled }
}

/// "{1} for each [thing]", "{1} plus an additional {1} for each [thing]", "{x}, where x is
/// [value]" (only these: a plain cost is parsed by the core patterns). `it` is what "it"
/// refers to in the counted phrase.
fn scaled_cost(s: &str, b: &mut Builder) -> Option<Cost> {
    let s = end(s);
    if let Some(v) = s.strip_prefix("{x}, where x is ") {
        let (value, rest) = parse_value_phrase(v, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        return Some(Cost {
            mana: None,
            parts: vec![CostPart::Repeated {
                cost: Box::new(resolution_cost("{1}")?),
                times: value,
            }],
        });
    }
    let (base, each) = s.split_once(" for each ")?;
    let it = b.it.clone();
    let times = parse_for_each(each, Some(&it))?;
    let (fixed, per) = match base.split_once(" plus an additional ") {
        Some((fixed, per)) => (Some(resolution_cost(fixed)?), per),
        None => (None, base),
    };
    let per = resolution_cost(per)?;
    // Only mana is repeated this way here.
    if !per.parts.is_empty() || per.mana.is_none() {
        return None;
    }
    let mut cost = fixed.unwrap_or_default();
    if !cost.parts.is_empty() {
        return None;
    }
    cost.parts.push(CostPart::Repeated {
        cost: Box::new(per),
        times,
    });
    Some(cost)
}

/// "counter target spell unless its controller pays {1} for each card in your graveyard".
fn counter_unless_scaled(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("counter ")?;
    let (spell, pays) = r.split_once(" unless its controller pays ")?;
    let (what, tail) = object_ref(spell, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let cost = scaled_cost(pays, b)?;
    Some(Effect::PayOptional {
        who: PlayerRef::ControllerOf(Box::new(what.clone())),
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::CounterSpell { what }),
    })
}

/// "sacrifice ~ unless you pay {1} for each card in your hand".
fn unless_you_pay_scaled(l: &str, b: &mut Builder) -> Option<Effect> {
    let (eff, cost) = end(l).rsplit_once(" unless you pay ")?;
    let effect = parse_simple(eff, b)?;
    let cost = scaled_cost(cost, b)?;
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(effect),
    })
}
