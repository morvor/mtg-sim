//! CR 702.120 Escalate: "Escalate [cost]" means "For each mode you choose beyond the first
//! as you cast this spell, you pay an additional [cost]." (CR 702.120a). A static ability
//! of modal spells that functions on the stack; paying it follows the rules for additional
//! costs (CR 601.2f–h).
//!
//! It stands for an additional cost of the spell itself: the escalate cost repeated once
//! per mode chosen beyond the first ([`CostPart::Repeated`]), counted once the modes are
//! chosen (CR 601.2b) and added to the total cost with the other increases, before any
//! reductions apply (CR 601.2f).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};

/// `Value::Custom`: the number of modes chosen for the spell `ctx.source` beyond the
/// first (none before its modes are chosen).
pub const MODES_BEYOND_FIRST: &str = "escalate:modes chosen beyond the first";

pub struct Escalate;

impl KeywordRules for Escalate {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Escalate]
    }

    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        let cost = kw.cost.clone()?;
        let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::ThisSpell,
            who: PlayerRel::You,
            change: CostChange::AdditionalCost(Cost::free().with(CostPart::Repeated {
                cost: Box::new(cost),
                times: Value::Custom(MODES_BEYOND_FIRST.into()),
            })),
        }));
        s.zone = FunctionZone::Stack;
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Escalate.name(),
        )])
    }

    fn custom_value(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<i64> {
        if name != MODES_BEYOND_FIRST {
            return None;
        }
        let modes = ctx
            .source
            .and_then(|s| g.obj(s).stack.as_deref())
            .map_or(0, |si| si.chosen.iter().filter(|c| c.mode.is_some()).count());
        Some(modes.saturating_sub(1) as i64)
    }
}

inventory::submit! { KeywordRegistration(&Escalate) }
