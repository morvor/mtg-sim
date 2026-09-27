//! CR 702.161 Living metal: "During your turn, this permanent is an artifact creature in
//! addition to its other types." (CR 702.161a). A type-changing effect (layer 4) of the
//! permanent's own static ability, applying only during its controller's turn.

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::*;

pub struct LivingMetal;

impl KeywordRules for LivingMetal {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::LivingMetal]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::AddTypes(vec![
                CardType::Artifact,
                CardType::Creature,
            ])],
        });
        s.condition = Some(Condition::YourTurn);
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::LivingMetal.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&LivingMetal) }
