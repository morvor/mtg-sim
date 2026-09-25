//! Paying resources while an ability resolves: "you may pay [cost]" (an optional cost; a
//! following "If you do" / "When you do" sentence reads whether it was paid, CR 603.12)
//! and "[effect] unless you pay [cost]" (CR 118.12a), with costs in mana, life (CR 119.4)
//! or energy (CR 107.14).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_simple, Builder};
use crate::oracle::phrases::*;

/// A cost paid during resolution: "{2}{B}", "{E}{E}", "2 life". Costs with X are left to
/// patterns that know what defines X (CR 107.3f), and "one or more {E}" / "any amount of
/// {E}" (the amount is a choice the effect refers to later) aren't costs of this form.
pub fn resolution_cost(s: &str) -> Option<Cost> {
    let s = end(s);
    let has_x = s.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x");
    if s.is_empty() || has_x || s.contains(" and ") || s.contains(" or ") {
        return None;
    }
    let (cost, loyalty) = crate::oracle::costs::parse_cost(s)
        .or_else(|| crate::oracle::costs::parse_cost(&format!("pay {s}")))?;
    if loyalty || (cost.mana.is_none() && cost.parts.is_empty()) {
        return None;
    }
    // Only resources a player pays: mana, life and energy.
    if !cost
        .parts
        .iter()
        .all(|p| matches!(p, CostPart::PayLife(_) | CostPart::PayEnergy(_)))
    {
        return None;
    }
    Some(cost)
}

/// "you may pay {1}", "you may pay 2 life", "you may pay {E}{E}": the controller may pay
/// the cost; the result is recorded for "if you do" / "when you do".
fn may_pay(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix("you may pay ")?;
    let cost = resolution_cost(r)?;
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::Noop),
    })
}

inventory::submit! { EffectPattern { name: "counters_resources: you may pay cost", priority: 100, parse: may_pay } }

/// "tap ~ unless you pay {E}", "sacrifice it unless you pay {E}{E}", "discard a card at
/// random unless you pay {E}{E}" (CR 118.12a): the controller may pay; if they don't, the
/// effect happens.
fn unless_you_pay(l: &str, b: &mut Builder) -> Option<Effect> {
    let (eff, cost) = end(l).rsplit_once(" unless you pay ")?;
    let cost = resolution_cost(cost)?;
    let effect = parse_simple(eff, b)?;
    Some(Effect::PayOptional {
        who: PlayerRef::You,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(effect),
    })
}

inventory::submit! { EffectPattern { name: "counters_resources: effect unless you pay", priority: 100, parse: unless_you_pay } }
