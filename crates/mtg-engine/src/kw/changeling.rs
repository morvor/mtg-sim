//! CR 702.73 Changeling: "Changeling" means "This object is every creature type." It's a
//! characteristic-defining ability that works everywhere, even outside the game
//! (CR 702.73a, 604.3).
//!
//! A printed changeling (or one an object has through a copy effect) comes with the
//! characteristic-defining static ability that gives the object every creature type in
//! layer 4 (see `oracle::keywords::compile_keyword`): so an effect that later removes the
//! ability (in layer 6) leaves the object every creature type, and an effect setting its
//! creature types overwrites them. That characteristic-defining ability exists only when
//! the keyword is printed, granted to a token by the effect creating it, or acquired
//! through a copy or text-changing effect (CR 604.3a). A changeling keyword any other
//! effect grants isn't a characteristic-defining ability: it's added in layer 6
//! (CR 613.1f), after creature types were determined in layer 4 (CR 613.1d), so it
//! doesn't make the object every creature type. The layer-4 flag `all_creature_types`
//! is the single source of truth.

use crate::ability::*;
use crate::object::Characteristics;
use crate::types::CardType;

/// Whether an object with these characteristics is every creature type: through
/// changeling's characteristic-defining ability or another "is every creature type"
/// effect (the layer-4 flag). A bare changeling keyword granted by an effect doesn't count
/// (CR 604.3a, 613.1d, 613.1f). Only creatures and kindred objects have creature types
/// (CR 205.3d).
pub fn every_creature_type(c: &Characteristics) -> bool {
    c.all_creature_types && (c.is(CardType::Creature) || c.is(CardType::Kindred))
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
