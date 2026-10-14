//! "Whenever ~ becomes the target of a spell or ability [an opponent controls], counter that
//! spell or ability [unless its controller pays {2}]." (Frost Titan, Shimmering
//! Glasskite), "... ~ deals 2 damage to that spell's controller." (Bonecrusher Giant),
//! "that spell or ability's controller sacrifices a land" (Lava Runner), "counter that
//! spell unless its controller discards a card" (Reality Smasher).
//!
//! In these triggers "that spell or ability" is the spell or ability that targeted the
//! object (CR 603.2, 115.1), and its controller is the trigger's player. The block is
//! rewritten to internal forms ("counter the targeting spell", "that player") and parsed
//! as an ordinary triggered ability; the ability keeps its printed text.

use super::statics::parse_for_each;
use super::{AbilityPattern, EffectPattern};
use crate::ability::*;
use crate::oracle::effects::Builder;
use crate::oracle::phrases::*;
use crate::oracle::CompileContext;

inventory::submit! {
    AbilityPattern { name: "sweep: becomes the target — that spell or ability", priority: 50, parse: targeted_by_block }
}
inventory::submit! {
    EffectPattern { name: "sweep: counter the targeting spell", priority: 95, parse: counter_targeting_spell }
}

/// Internal form of "counter that spell or ability" inside a "becomes the target" trigger.
const COUNTER_IT: &str = "counter the targeting spell";

fn is_becomes_target(t: &TriggerCond) -> bool {
    match t {
        TriggerCond::BecomesTarget { .. } => true,
        // "Whenever you [or a permanent you control] become the target of …".
        TriggerCond::Custom(name) => crate::kw::activated_ability_kind::is_player_targeted(name),
        TriggerCond::AnyOf(v) => !v.is_empty() && v.iter().all(is_becomes_target),
        TriggerCond::Where { trigger, .. } | TriggerCond::Batched { trigger, .. } => {
            is_becomes_target(trigger)
        }
        TriggerCond::FirstTimeEachTurn(inner) => is_becomes_target(inner),
        _ => false,
    }
}

fn targeted_by_block(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    if !lower.starts_with("whenever ") || t.contains('"') {
        return None;
    }
    let (cond, _) = lower.split_once(", ")?;
    let (trigger, _, _) = crate::oracle::triggers::parse_trigger_condition(cond)?;
    if !is_becomes_target(&trigger) {
        return None;
    }
    let mut text = t.to_string();
    let mut changed = false;
    for (from, to) in [
        ("that spell or ability's controller", "that player"),
        ("that spell's controller", "that player"),
        ("that ability's controller", "that player"),
        ("counter that spell or ability", COUNTER_IT),
        ("counter that spell", COUNTER_IT),
        ("counter that ability", COUNTER_IT),
    ] {
        let mut cap = from.to_string();
        cap[..1].make_ascii_uppercase();
        for f in [from.to_string(), cap] {
            if text.contains(&f) {
                text = text.replace(&f, to);
                changed = true;
            }
        }
    }
    let rest = text.to_lowercase();
    if !changed || rest.contains("that spell") || rest.contains("that ability") {
        return None;
    }
    let a = crate::oracle::triggers::parse_triggered(&text, ctx)?;
    Some(vec![AbilityDef::new(a.kind.clone(), t)])
}

/// "counter the targeting spell [unless its controller pays {2}]" (internal form, see
/// above): the spell or ability that targeted the object; its controller may pay.
fn counter_targeting_spell(l: &str, _b: &mut Builder) -> Option<Effect> {
    let r = end(l).strip_prefix(COUNTER_IT)?;
    let counter = Effect::CounterSpell {
        what: Sel::TriggerSpell,
    };
    let r = r.trim();
    if r.is_empty() {
        return Some(counter);
    }
    let cost = if let Some(c) = r.strip_prefix("unless its controller pays ") {
        match c.split_once(" for each ") {
            Some((per, each)) => {
                let per = super::counters_resources_pay::resolution_cost(per)?;
                let times = parse_for_each(each, None)?;
                Cost {
                    mana: None,
                    parts: vec![CostPart::Repeated {
                        cost: Box::new(per),
                        times,
                    }],
                }
            }
            None => super::counters_resources_pay::resolution_cost(c)?,
        }
    } else if end(r) == "unless its controller discards a card" {
        // Reality Smasher: discarding a card is the cost its controller may pay.
        let (cost, _) = crate::oracle::costs::parse_cost("discard a card")?;
        if cost.mana.is_some()
            || !matches!(cost.parts.as_slice(), [CostPart::Discard { random: false, .. }])
        {
            return None;
        }
        cost
    } else {
        return None;
    };
    Some(Effect::PayOptional {
        who: PlayerRef::TriggerPlayer,
        cost,
        then: Box::new(Effect::Noop),
        otherwise: Box::new(counter),
    })
}
