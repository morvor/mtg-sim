//! Triggers, grammar V (cards4-triggers-durations): delayed triggered abilities and
//! triggers created by effects (CR 603.7).
//!
//! - "create X 1/1 Sand Warrior creature tokens ... at the beginning of your next upkeep,
//!   where X is the number of lands you control at that time" (Hazezon Tamar): a value
//!   the delayed ability determines as it resolves (CR 603.7c, 608.2h).

use super::EffectPattern;
use crate::ability::*;
use crate::oracle::effects::{parse_effect_text, Builder};
use crate::oracle::phrases::end;

/// "[instruction] at the beginning of your next upkeep, where X is [value] at that time":
/// the delayed ability's instruction, with X determined as it resolves (the core grammar
/// reads the X of a delayed instruction inside its body).
fn where_x_at_that_time(l: &str, b: &mut Builder) -> Option<Effect> {
    let l = end(l);
    let head = l.strip_suffix(" at that time")?;
    let (instruction, value) = head.split_once(", where x is ")?;
    let (trigger, inner) = super::triggers_delayed::split_delay(instruction)?;
    let saved = (b.targets.len(), b.it.clone());
    let effect = parse_effect_text(&format!("{inner}, where x is {value}"), b);
    // The delayed ability's own instruction: nothing it refers to comes from the creating
    // ability (no targets, objects or event of its own).
    let json = effect
        .as_ref()
        .and_then(|e| serde_json::to_string(e).ok())
        .unwrap_or_default();
    if effect.is_none()
        || b.targets.len() != saved.0
        || ["Target", "Trigger", "EventAmount", "\"This\""]
            .iter()
            .any(|p| json.contains(p))
    {
        b.targets.truncate(saved.0);
        b.it = saved.1;
        return None;
    }
    Some(Effect::DelayedTrigger {
        trigger,
        body: Box::new(Body::effect(effect?)),
        once: true,
    })
}

inventory::submit! { EffectPattern { name: "triggers 5: delayed instruction, where X is [value] at that time", priority: 100, parse: where_x_at_that_time } }
