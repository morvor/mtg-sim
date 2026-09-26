//! Max speed (CR 702.178a): "Max speed — [Ability]" means "As long as your speed is 4,
//! this object has '[Ability].'" It's a keyword ability, not an ability word (CR 207.2c),
//! so the ability it grants only functions while its controller has max speed
//! (CR 702.179e):
//!
//! - a static ability applies only while you have max speed;
//! - an activated ability can be activated only while you have max speed;
//! - a triggered ability triggers only if you have max speed as its trigger event occurs
//!   (it isn't an intervening "if" clause, so it isn't checked again on resolution).
//!
//! The granted ability keeps the zones it functions from (CR 702.178b): "Max speed —
//! {3}, Exile this card from your graveyard: Draw a card." works from the graveyard.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;
use std::sync::Arc;

fn and_max_speed(c: Option<Condition>) -> Condition {
    match c {
        None => Condition::MaxSpeed,
        Some(Condition::And(mut v)) => {
            v.push(Condition::MaxSpeed);
            Condition::And(v)
        }
        Some(c) => Condition::And(vec![c, Condition::MaxSpeed]),
    }
}

/// The granted ability, functioning only while its controller has max speed.
fn gated(a: &Ability, text: &str) -> Option<Ability> {
    let mut def = (**a).clone();
    match &mut def.kind {
        AbilityKind::Static(s) => s.condition = Some(and_max_speed(s.condition.take())),
        AbilityKind::Activated(act) => act.condition = Some(and_max_speed(act.condition.take())),
        AbilityKind::Triggered(t) => {
            t.trigger = TriggerCond::Where {
                trigger: Box::new(t.trigger.clone()),
                cond: Condition::MaxSpeed,
            };
        }
        // Keyword lines and anything else can't be made conditional here.
        _ => return None,
    }
    def.text = text.to_string();
    Some(Arc::new(def))
}

fn max_speed(text: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let text = text.trim();
    let rest = text.strip_prefix("Max speed — ")?;
    let inner = crate::oracle::parse_ability(rest, ctx)?;
    if inner.is_empty() {
        return None;
    }
    inner.iter().map(|a| gated(a, text)).collect()
}

inventory::submit! { AbilityPattern { name: "k702.178 max speed", priority: 50, parse: max_speed } }
