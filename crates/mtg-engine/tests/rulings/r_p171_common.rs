//! Shared helpers for the tests of rulings batch P171 (`r_p171_*.rs`): spell-cast
//! triggers, damage-increasing replacement effects, lands that check for land types,
//! tapped-creature checks and token-doubling effects. (The helpers of batches S01–S35
//! are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_s01_common::{give_mana_for, supported};
pub use crate::r_s04_common::stack_items;
pub use crate::r_s14_common::triggers_from;
pub use crate::r_s28_common::cast_card;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// A real card from `p`'s hand, cast with basic lands for its mana cost and the given
/// targets (one per target slot).
pub fn cast_targeting(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    give_mana_for(t, p, name);
    let card = t.hand(p, name);
    t.cast_with(p, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// Damage marked on the object.
pub fn damage_on(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}
