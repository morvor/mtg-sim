//! "Unless [a player] pays" with a scaled cost: "Counter target spell unless its controller
//! pays {1} for each card in your graveyard.", "... pays {1} plus an additional {1} for
//! each Faerie you control", "... pays {X}, where X is its mana value", and "sacrifice ~
//! unless you pay {1} for each card in your hand".
//!
//! The amount is determined as the payment is proposed, while the spell or ability
//! resolves (CR 118.12, 608.2h); the player may pay it or not (CR 118.12a). "{X}, where X
//! is [value]" stays a cost of {X}, X defined by the text (CR 107.3c): it's a cost that
//! contains {X}, which mana spendable only on such costs can pay (Elementalist's Palette,
//! Rosheen Meanderer), unlike "{1} for each [thing]".

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
/// refers to in the counted phrase. With "{x}, where x is [value]", the cost is {X} and
/// the value is what X stands for.
fn scaled_cost(s: &str, b: &mut Builder) -> Option<(Cost, Option<Value>)> {
    let s = end(s);
    if let Some(v) = s.strip_prefix("{x}, where x is ") {
        let (value, rest) = parse_value_phrase(v, b)?;
        if !end(&rest).is_empty() {
            return None;
        }
        let x = crate::mana::ManaCost::parse("{X}")?;
        return Some((Cost::mana(x), Some(value)));
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
    Some((cost, None))
}

/// The payment `pay`, preceded by setting X to the value the text defines it as, if any
/// (X is determined as the payment is offered and keeps that value, CR 107.3c, 608.2h).
fn defining_x(x: Option<Value>, pay: Effect) -> Effect {
    match x {
        Some(value) => Effect::seq(vec![Effect::SetX { value }, pay]),
        None => pay,
    }
}

/// "counter target spell unless its controller pays {1} for each card in your graveyard".
fn counter_unless_scaled(l: &str, b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("counter ")?;
    let (spell, pays) = r.split_once(" unless its controller pays ")?;
    let (what, tail) = object_ref(spell, b)?;
    if !end(&tail).is_empty() {
        return None;
    }
    let (cost, x) = scaled_cost(pays, b)?;
    let pay = Effect::PayOptional {
        who: PlayerRef::ControllerOf(Box::new(what.clone())),
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::CounterSpell { what }),
    };
    Some(defining_x(x, pay))
}

/// "sacrifice ~ unless you pay {1} for each card in your hand".
fn unless_you_pay_scaled(l: &str, b: &mut Builder) -> Option<Effect> {
    // "~ deals 3 damage to that player unless they pay {1} for each vortex counter on ~"
    // (Energy Vortex): the player the instruction is about decides (CR 118.12a).
    let (eff, cost, who) = match end(l).rsplit_once(" unless you pay ") {
        Some((e, c)) => (e, c, PlayerRef::You),
        None => {
            let (e, c) = end(l)
                .rsplit_once(" unless they pay ")
                .or_else(|| end(l).rsplit_once(" unless that player pays "))?;
            if !e.contains("that player") {
                return None;
            }
            (e, c, b.it_player.clone())
        }
    };
    let effect = parse_simple(eff, b)?;
    let (cost, x) = scaled_cost(cost, b)?;
    let pay = Effect::PayOptional {
        who,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(effect),
    };
    Some(defining_x(x, pay))
}
