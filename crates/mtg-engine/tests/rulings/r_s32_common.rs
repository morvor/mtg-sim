//! Shared helpers for the tests of rulings batch S32 (`r_s32_*.rs`): graveyards (crimes,
//! descending, mana values among cards, graveyard order, cards that work from the
//! graveyard), hands (opening hands, maximum hand size, discarding) and lands (land
//! types, basic lands, mana abilities). (The helpers of batches S01–S29 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Names of the cards in `p`'s library, top first.
pub fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

/// Whether `p`'s library holds a card named `name`.
pub fn in_library(t: &TestGame, p: PlayerId, name: &str) -> bool {
    library_names(t, p).iter().any(|n| n == name)
}

/// The options offered by the `ChooseOption` decisions asked since decision `from`.
pub fn options_offered(t: &TestGame, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect()
}

/// The subtypes of the object (followed across zone changes) now.
pub fn subtypes_now(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .subtypes
        .iter()
        .map(|s| s.to_string())
        .collect()
}

/// Whether the object (followed across zone changes) is a basic land now.
pub fn basic_now(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.supertypes.contains(Supertype::Basic)
}
