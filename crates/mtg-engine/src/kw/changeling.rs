//! CR 702.73 Changeling: "Changeling" means "This object is every creature type." It's a
//! characteristic-defining ability that works everywhere, even outside the game
//! (CR 702.73a, 604.3).
//!
//! A printed changeling (or one an object has through a copy effect) comes with the
//! characteristic-defining static ability that gives the object every creature type in
//! layer 4 (see `oracle::keywords::compile_keyword`): so an effect that later removes the
//! ability (in layer 6) leaves the object every creature type, and an effect setting its
//! creature types overwrites them. An object given the keyword alone by an effect is
//! treated as every creature type while it has the keyword.

use crate::ability::*;
use crate::keywords::KeywordKind;
use crate::object::Characteristics;

/// Whether the characteristics include changeling's characteristic-defining ability.
pub fn has_changeling_cda(c: &Characteristics) -> bool {
    c.abilities.iter().any(|a| match &a.kind {
        AbilityKind::Static(s) if s.is_cda => matches!(
            &s.effect,
            StaticEffect::Continuous { mods, .. } if mods.iter().any(|m| matches!(m, Modification::AllCreatureTypes))
        ),
        _ => false,
    })
}

/// Whether an object with these characteristics is every creature type because it has the
/// changeling keyword without the characteristic-defining ability that already gave it
/// every creature type (a keyword granted by an effect).
pub fn every_creature_type_by_keyword(c: &Characteristics) -> bool {
    c.has_keyword(KeywordKind::Changeling) && !has_changeling_cda(c)
}

/// The characteristic-defining ability changeling stands for: "This object is every
/// creature type" (layer 4, functioning in every zone).
pub fn changeling_cda() -> StaticAbility {
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods: vec![Modification::AllCreatureTypes],
    });
    s.is_cda = true;
    s.zone = FunctionZone::Anywhere;
    s
}
