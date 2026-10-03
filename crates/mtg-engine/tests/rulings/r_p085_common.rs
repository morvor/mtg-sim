//! Shared helpers for the tests of rulings batch P085 (`r_p085_*.rs`): graveyard hate,
//! hosers of green, high mana value, high or low power and toughness, instants, lifegain,
//! and nonbasic lands. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p076_common::mana;
pub use crate::r_p125_common::shuffled;
pub use crate::r_s01_common::supported;

/// Gives `p` `n` mana of each listed type.
pub fn pool(t: &mut TestGame, p: PlayerId, types: &[(ManaType, u32)]) {
    for (ty, n) in types {
        mana(t, p, *ty, *n);
    }
}

/// The ids of the objects in `p`'s graveyard.
pub fn graveyard_ids(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.player(p).graveyard.clone()
}

/// The number of tokens with creature type `subtype` controlled by `p`.
pub fn tokens_of(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    crate::r_s05_common::tokens_with_subtype(t, p, subtype).len()
}

/// Whether `p` was asked to choose targets or objects.
pub fn chose_anything(t: &TestGame, p: PlayerId) -> bool {
    use mtg_engine::decision::Decision;
    t.asked().iter().any(|(q, d)| {
        *q == p
            && matches!(
                d,
                Decision::ChooseTargets { .. }
                    | Decision::ChooseEntities { .. }
                    | Decision::ChooseOption { .. }
                    | Decision::YesNo { .. }
            )
    })
}
