//! CR 702.98 Unleash: two static abilities, "You may have this permanent enter with an
//! additional +1/+1 counter on it" and "This permanent can't block as long as it has a
//! +1/+1 counter on it" (CR 702.98a).
//!
//! The first is an optional replacement effect: the choice is made as the permanent
//! enters, from wherever it enters. The second applies to any +1/+1 counter, not only the
//! one unleash put on it; it restricts declaring blockers, so a blocking creature that gets
//! a counter keeps blocking.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::counters;

pub struct Unleash;

impl KeywordRules for Unleash {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Unleash]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let text = KeywordKind::Unleash.name();
        let enter = StaticAbility::new(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::Source),
            action: ReplacementAction::EnterWithCounters(counters::PLUS1.into(), Value::c(1)),
            self_replacement: false,
            optional: true,
        }));
        let mut cant_block =
            StaticAbility::new(StaticEffect::Restriction(Restriction::CantBlock(Filter::Source)));
        cant_block.condition = Some(Condition::Compare(
            Value::CountersOn(Box::new(Sel::This), Some(counters::PLUS1.into())),
            Cmp::Ge,
            Value::c(1),
        ));
        Some(vec![
            AbilityDef::new(AbilityKind::Static(enter), text),
            AbilityDef::new(AbilityKind::Static(cant_block), text),
        ])
    }
}

inventory::submit! { KeywordRegistration(&Unleash) }
