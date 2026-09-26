//! CR 702.114 Devoid: "Devoid" means "This object is colorless." It's a
//! characteristic-defining ability that functions everywhere, even outside the game
//! (CR 702.114a, 604.3).
//!
//! The keyword stands for a static ability setting the object's colors to colorless in
//! layer 5 (CR 613.1e), applied before other color-changing effects (CR 613.3). An object
//! that has devoid from the start of the layer system (printed, or through a copy effect)
//! is colorless even if an effect later removes devoid in layer 6; one that only gains
//! devoid from an ability-adding effect (layer 6) keeps its colors, since the color
//! change would have to apply in an earlier layer (see `Game::compute_characteristics`).

use super::{KeywordRegistration, KeywordRules};
use crate::ability::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::types::ColorSet;

pub struct Devoid;

impl KeywordRules for Devoid {
    fn kinds(&self) -> &'static [KeywordKind] {
        &[KeywordKind::Devoid]
    }

    fn derived(&self, _kw: &Keyword) -> Option<Vec<Ability>> {
        let mut s = StaticAbility::new(StaticEffect::Continuous {
            affected: Filter::Source,
            mods: vec![Modification::SetColors(ColorSet::NONE)],
        });
        s.is_cda = true;
        s.zone = FunctionZone::Anywhere;
        Some(vec![AbilityDef::new(
            AbilityKind::Static(s),
            KeywordKind::Devoid.name(),
        )])
    }
}

inventory::submit! { KeywordRegistration(&Devoid) }
