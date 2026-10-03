//! "If the {2}{U} cost was paid, ..." (Ingenious Mastery and the other Masteries: "You
//! may pay {2}{U} rather than pay this spell's mana cost."): whether the spell was cast
//! for that alternative cost (CR 118.9) — even if cost increases or reductions changed
//! how much was paid. "If that cost wasn't paid, ..." is its negation.

use super::{ConditionPattern, FollowupPattern};
use crate::ability::*;
use crate::mana::ManaCost;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::end;

/// "the {2}{u} cost was paid" (lowercased oracle text).
fn the_cost_was_paid(c: &str) -> Option<Condition> {
    let m = end(c)
        .strip_prefix("the ")?
        .strip_suffix(" cost was paid")?;
    if !m.starts_with('{') || !m.ends_with('}') {
        return None;
    }
    let cost = ManaCost::parse(&m.to_uppercase())?;
    if cost.symbols.is_empty() {
        return None;
    }
    Some(Condition::CostPaid(
        crate::spell_costs::alternative_cost_name(&cost),
    ))
}

inventory::submit! { ConditionPattern { name: "the {N} cost was paid", priority: 100, parse: the_cost_was_paid } }

/// The condition of the previous sentence's "If the {2}{U} cost was paid, ...", if it's
/// one.
fn last_cost_paid(e: &Effect) -> Option<Condition> {
    match e {
        Effect::If {
            cond: c @ Condition::CostPaid(name),
            ..
        } if name.starts_with("alternative cost ") => Some(c.clone()),
        Effect::Seq(v) => v.last().and_then(last_cost_paid),
        _ => None,
    }
}

/// "If that cost wasn't paid, [effect]." after "If the {2}{U} cost was paid, ...".
fn that_cost_wasnt_paid(l: &str, prev: &mut Effect, b: &mut Builder) -> bool {
    let Some(x) = end(l).strip_prefix("if that cost wasn't paid, ") else {
        return false;
    };
    let Some(cond) = last_cost_paid(prev) else {
        return false;
    };
    let Some(e) = crate::oracle::effects::parse_sentence(x, b) else {
        return false;
    };
    *prev = Effect::seq(vec![
        std::mem::take(prev),
        Effect::If {
            cond: Condition::Not(Box::new(cond)),
            then: Box::new(e),
            otherwise: Box::new(Effect::Noop),
        },
    ]);
    true
}

inventory::submit! { FollowupPattern { name: "if that cost wasn't paid, [effect]", priority: 100, apply: that_cost_wasnt_paid } }
