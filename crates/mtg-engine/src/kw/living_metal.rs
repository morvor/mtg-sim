//! CR 702.161 Living Metal: "Living metal" means "During your turn, this permanent is an
//! artifact creature in addition to its other types" (CR 702.161a). It's a static ability
//! of the permanent (layer 4, CR 613.1d): while it applies, the Vehicle is a creature with
//! its printed power and toughness, and effects that apply only to noncreature permanents
//! don't apply to it (the dependency is handled by the layer system, CR 613.8).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::CardType;

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
            "During your turn, this permanent is an artifact creature in addition to its other types.",
        )])
    }
}

inventory::submit! { KeywordRegistration(&LivingMetal) }
