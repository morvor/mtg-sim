//! "As an additional cost to cast this spell, you may blight N." (CR 701.68, 118.8): an
//! optional additional cost announced and paid while casting the spell (CR 601.2b,
//! 601.2f–h), which a player who controls no creatures can't choose to pay (CR 701.68b);
//! and the condition that refers back to such a cost, "this spell's additional cost was
//! paid" (Burning Curiosity).

use super::{AbilityPattern, ConditionPattern};
use crate::ability::*;
use crate::oracle::costs::parse_cost;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;
use smol_str::SmolStr;

/// "As an additional cost to cast ~, you may blight N."
fn optional_blight(text: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    if text.contains('\n') {
        return None;
    }
    let lower = text.to_lowercase();
    let r = end(&lower).strip_prefix("as an additional cost to cast ~, you may ")?;
    let (cost, false) = parse_cost(r)? else {
        return None;
    };
    let is_blight = |e: &Effect| {
        matches!(
            e,
            Effect::KeywordAction {
                action: KeywordAction::Blight,
                ..
            }
        )
    };
    if cost.mana.is_some() || !matches!(cost.parts.as_slice(), [CostPart::Effect(e)] if is_blight(e))
    {
        return None;
    }
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: CostChange::OptionalAdditionalCost {
            name: SmolStr::new(crate::kwa::blight::BLIGHTED_EVENT),
            cost,
        },
    }));
    s.zone = FunctionZone::Anywhere;
    Some(vec![AbilityDef::new(AbilityKind::Static(s), text.trim())])
}

inventory::submit! { AbilityPattern { name: "a701 optional blight cost", priority: 90, parse: optional_blight } }

/// "this spell's additional cost was paid": one of its own optional additional costs was
/// paid as it was cast.
fn additional_cost_was_paid(c: &str) -> Option<Condition> {
    match end(c) {
        "~'s additional cost was paid" | "this spell's additional cost was paid" => Some(
            Condition::Custom(SmolStr::new(crate::player_control::ADDITIONAL_COST_PAID)),
        ),
        _ => None,
    }
}

inventory::submit! { ConditionPattern { name: "this spell's additional cost was paid", priority: 100, parse: additional_cost_was_paid } }
