//! "You may cast this spell as though it had flash. If you cast it any time a sorcery
//! couldn't have been cast, the controller of the permanent it becomes sacrifices it at the
//! beginning of the next cleanup step." (Spider Climb, Armor of Thorns, ...; CR 307.5a).
//! The sacrifice happens only if the spell was cast using its own ability.

use super::AbilityPattern;
use crate::ability::*;
use crate::oracle::phrases::end;
use crate::oracle::CompileContext;

/// `Condition::Custom` name: the permanent was cast as though it had flash using its own
/// ability, at a time a sorcery couldn't have been cast (CR 307.5a).
pub const CAST_BY_OWN_FLASH_AT_INSTANT_TIMING: &str = "cast by own flash at instant timing";

fn flash_then_sacrifice(block: &str, _ctx: &CompileContext) -> Option<Vec<Ability>> {
    let lower = block.to_lowercase().replace('\u{2019}', "'");
    let rest = end(&lower).strip_prefix("you may cast ~ as though it had flash. ")?;
    if rest
        != "if you cast it any time a sorcery couldn't have been cast, the controller of the permanent it becomes sacrifices it at the beginning of the next cleanup step"
    {
        return None;
    }
    let mut flash = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: CostChange::FlashForAdditionalCost(Cost::free()),
    }));
    flash.zone = FunctionZone::Anywhere;
    let sacrifice = StaticEffect::Replacement(ReplacementDef {
        event: ReplacementEvent::EntersBattlefield(Filter::Source),
        // Checked as the spell resolves and becomes the permanent; the delayed trigger is
        // created for the permanent as it's put onto the battlefield.
        action: ReplacementAction::AsEnters(Box::new(Effect::If {
            cond: Condition::Custom(CAST_BY_OWN_FLASH_AT_INSTANT_TIMING.into()),
            then: Box::new(Effect::OnEntry(Box::new(Effect::AtNext {
                step: TriggerStep::Cleanup,
                effect: Box::new(Effect::SacrificeObjects { what: Sel::This }),
            }))),
            otherwise: Box::new(Effect::Noop),
        })),
        self_replacement: false,
        optional: false,
    });
    Some(vec![
        AbilityDef::new(AbilityKind::Static(flash), block),
        AbilityDef::new(
            AbilityKind::Static(StaticAbility::new(sacrifice)),
            block,
        ),
    ])
}

inventory::submit! {
    AbilityPattern { name: "r307 flash, sacrificed if cast at instant timing", priority: 40, parse: flash_then_sacrifice }
}
