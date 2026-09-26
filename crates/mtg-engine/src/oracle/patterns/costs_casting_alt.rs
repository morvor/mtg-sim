//! A spell's own alternative costs (CR 118.9, 601.2b): "You may pay {W}{U}{B}{R}{G}
//! rather than pay this spell's mana cost.", "You may pay 1 life and exile a blue card
//! from your hand rather than pay this spell's mana cost.", "If you control a Swamp, you
//! may pay 4 life rather than pay this spell's mana cost.", "You may return two Islands
//! you control to their owner's hand rather than pay this spell's mana cost."
//!
//! The alternative cost is offered as a way of casting the spell only while its
//! condition, if any, is true (see [`crate::spell_costs::alternative_cost_allowed`]).

use super::costs_casting_self::{cost_condition, this_spell_cost_ability};
use super::AbilityPattern;
use crate::ability::*;
use crate::mana::ManaCost;
use crate::oracle::costs::parse_cost;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

/// One action of an alternative cost ("pay {1}", "pay 4 life", "sacrifice a Mountain",
/// "exile a blue card from your hand", "return an Island you control to its owner's
/// hand", "tap an untapped creature you control", "discard a Forest card").
fn cost_action(s: &str) -> Option<Cost> {
    let s = end(s);
    if let Some(m) = s.strip_prefix("pay {") {
        let m = ManaCost::parse(&format!("{{{m}"))?;
        if m.has_x() || format!("{m}").to_lowercase() != format!("{{{}", &s[5..]) {
            return None;
        }
        return Some(Cost::mana(m));
    }
    let (c, false) = parse_cost(s)? else {
        return None;
    };
    let ok = c.mana.is_none()
        && c.parts.len() == 1
        && matches!(
            c.parts[0],
            CostPart::PayLife(Value::Const(_))
                | CostPart::Sacrifice { .. }
                | CostPart::Exile { .. }
                | CostPart::ReturnToHand { .. }
                | CostPart::TapUntapped { .. }
                | CostPart::Discard { random: false, .. }
        );
    ok.then_some(c)
}

/// "pay {1} and return a basic land you control to its owner's hand": the actions joined
/// by "and", paid together.
fn alternative_cost(s: &str) -> Option<Cost> {
    // A value defined by the card paid ("a black card with mana value X") isn't a cost
    // this compiler can express.
    if s.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return None;
    }
    let mut cost = Cost::free();
    for part in s.split(" and ") {
        let c = cost_action(part)?;
        if let Some(m) = c.mana {
            match cost.mana.as_mut() {
                Some(t) => t.add(&m),
                None => cost.mana = Some(m),
            }
        }
        cost.parts.extend(c.parts);
    }
    Some(cost)
}

/// "[If <condition>, ]you may <cost> rather than pay ~'s mana cost[ if <condition>]."
fn own_alternative_cost(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let l = end(&lower);
    if l.contains(". ") {
        return None;
    }
    let (pre, body) = match l.split_once(", you may ") {
        Some((head, body)) => (Some(cost_condition(head, ctx)?), body),
        None => (None, l.strip_prefix("you may ")?),
    };
    let (cost_s, tail) = body.split_once(" rather than pay ~'s mana cost")?;
    let post = match tail.trim() {
        "" => None,
        t => Some(cost_condition(t, ctx)?),
    };
    let cost = alternative_cost(cost_s)?;
    let cond = match (pre, post) {
        (Some(a), Some(b)) => Some(Condition::And(vec![a, b])),
        (a, b) => a.or(b),
    };
    Some(vec![this_spell_cost_ability(
        CostChange::AlternativeCost(cost),
        cond,
        text,
    )])
}

inventory::submit! { AbilityPattern { name: "costs_casting: rather than pay this spell's mana cost", priority: 80, parse: own_alternative_cost } }
