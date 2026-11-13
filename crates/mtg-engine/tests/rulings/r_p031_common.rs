//! Shared helpers for rulings batch P031 (`r_p031_*.rs`): cards that copy themselves —
//! permanents that create token copies of themselves and spells that copy themselves.
//! (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use crate::r_s01_common::attack_with;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The permanents named `name` controlled by `p` (tokens and nontokens).
pub fn named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// The tokens `p` controls that entered after the battlefield snapshot `before`.
pub fn tokens_since(t: &TestGame, p: PlayerId, before: &[ObjectId]) -> Vec<ObjectId> {
    crate::r_s26_common::new_tokens(t, p, before)
}

/// Attacks P1 with `c`, unblocked, and advances to the combat damage step (combat damage
/// triggers on the stack).
pub fn hit(t: &mut TestGame, c: ObjectId) {
    attack_with(t, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
}

/// Moves the permanent to its owner's graveyard (destroying it) and settles.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    crate::r_p030_common::kill(t, id)
}
