//! Shared helpers for the tests of rulings batch P070 (`r_p070_*.rs`): still more cards
//! that let a player cast (or play) cards from exile ("gives castable from exile").

#![allow(dead_code)]

use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub use crate::r_p068_common::{
    cast_free, castable, exiled, mana, move_to, o, offered_since, playable_land, pool,
    target_named_when_offered, was_offered,
};
pub use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};

/// Whether `log` shows `first` before `then`.
pub fn before(log: &str, first: &str, then: &str) -> bool {
    match (log.find(first), log.find(then)) {
        (Some(a), Some(b)) => a < b,
        _ => false,
    }
}

/// Asserts that P0 can't cast `card` in its own main phase with an empty stack and plenty
/// of mana: the permission was only for a resolution that's over.
pub fn not_castable_later(t: &mut TestGame, card: ObjectId) {
    assert!(exiled(t, card), "{}", t.dump_log());
    t.g.stack.clear();
    t.set_step(P0, Step::PostcombatMain);
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ] {
        mana(t, P0, ty, 2);
    }
    mana(t, P0, ManaType::C, 6);
    assert!(!castable(t, P0, card), "{}", t.dump_log());
}

/// The names of the objects on the stack, bottom first.
pub fn stack_names(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}

/// The object on the stack named `name`, if any.
pub fn on_stack(t: &TestGame, name: &str) -> Option<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .find(|id| t.g.obj(*id).chars.name == name)
}

/// Whether the object (followed across zone changes) is on the stack.
pub fn is_on_stack(t: &TestGame, id: ObjectId) -> bool {
    t.zone(id) == Zone::Stack
}

/// Fills `p`'s pool with plenty of mana of every type.
pub fn lots_of_mana(t: &mut TestGame, p: PlayerId) {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        mana(t, p, ty, 4);
    }
}
