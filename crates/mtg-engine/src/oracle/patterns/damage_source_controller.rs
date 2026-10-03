//! "Whenever a source deals damage to ~, that source's controller [instruction]." (Crag
//! Saurian, Reaper of Sheoldred, Belltower Sphinx, Phyrexian Obliterator, Archfiend of
//! Spite): "that source's controller" is the player who controls the source of the damage
//! as the ability resolves, or who last controlled it if it's gone (CR 608.2h). The
//! instruction is read with "that player" meaning that player; "that many" / "that much"
//! is the amount of damage dealt.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::CompileContext;

fn that_sources_controller(block: &str, ctx: &CompileContext) -> Option<Vec<Ability>> {
    let t = block.trim();
    let lower = t.to_lowercase();
    if !lower.starts_with("whenever a source ") {
        return None;
    }
    let (cond, rest) = lower.split_once(", that source's controller ")?;
    let (trigger, it, _) = crate::oracle::triggers::parse_trigger_condition(cond)?;
    // The damage's source is the trigger's other object (`EventInfo::other`).
    if !matches!(
        trigger,
        TriggerCond::DealsDamage {
            to: DamageRecipient::Object(_),
            ..
        }
    ) {
        return None;
    }
    // "That many" / "that much": the damage dealt, read as an X defined for the text.
    if rest.split(|c: char| !c.is_alphanumeric()).any(|w| w == "x") {
        return None;
    }
    let rest = rest.replace("that many", "x").replace("that much", "x");
    // Read with "that player" as the trigger's player, then made the source's controller
    // (the text names no other player the trigger is about).
    let body = super::value_grammar::with_x_defined(true, || {
        crate::oracle::effects::parse_trigger_body(
            &format!("that player {rest}"),
            ctx,
            it,
            PlayerRef::TriggerPlayer,
        )
    })?;
    let body = sources_controller(body)?;
    let body = Body {
        effect: super::r107_numbers::substitute_x(&body.effect, &Value::EventAmount)?,
        ..body
    };
    let tr = TriggeredAbility {
        trigger,
        intervening_if: None,
        body,
        once_per_turn: false,
        is_mana_ability: false,
        zone: FunctionZone::Battlefield,
        cant_be_countered: false,
        do_once_per_turn: false,
    };
    Some(vec![AbilityDef::new(AbilityKind::Triggered(tr), t)])
}

/// `body` with the trigger's player replaced by the controller of the damage's source.
fn sources_controller(body: Body) -> Option<Body> {
    use serde_json::Value as J;
    fn walk(v: &mut J, with: &J) {
        match v {
            J::String(s) if s == "TriggerPlayer" => *v = with.clone(),
            J::Array(a) => a.iter_mut().for_each(|x| walk(x, with)),
            J::Object(m) => m.values_mut().for_each(|x| walk(x, with)),
            _ => {}
        }
    }
    let who = PlayerRef::ControllerOf(Box::new(Sel::TriggerOtherObject));
    let with = serde_json::to_value(&who).ok()?;
    let mut v = serde_json::to_value(&body).ok()?;
    walk(&mut v, &with);
    serde_json::from_value(v).ok()
}

inventory::submit! { AbilityPattern { name: "whenever a source deals damage to ~, that source's controller [instruction]", priority: 100, parse: that_sources_controller } }
