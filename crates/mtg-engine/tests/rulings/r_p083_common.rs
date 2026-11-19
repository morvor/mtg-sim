//! Shared helpers for the tests of rulings batch P083 (`r_p083_*.rs`): "hate" cards —
//! counterspell hate (spells that can't be countered, Glen Elendra's Answer, City of
//! Solitude), discard hate (cards that punish an opponent for making you discard), flying
//! hate, "dealt damage this turn" removal, enters-the-battlefield hate (Torpor Orb,
//! Mistcaller) and graveyard hate. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_s01_common::{give_mana_for, supported};
pub use crate::r_s04_common::{ability_targets, spell_targets};
pub use crate::r_s25_common::cast_new;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn player(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// `p` casts the real spell `name` (lands for its mana cost are put onto the battlefield
/// first) with the given targets and lets everything on the stack resolve.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    cast_new(t, p, name, targets);
    t.resolve_all();
}

/// `p` casts the real counterspell `name` targeting `spell` and it resolves (only it).
pub fn counter_with(t: &mut TestGame, p: PlayerId, name: &str, spell: ObjectId) {
    supported(name);
    cast_new(t, p, name, &[obj(spell)]);
    t.resolve();
}

/// Whether `id` (not followed across zones) is still on the stack.
pub fn on_stack(t: &TestGame, id: ObjectId) -> bool {
    t.g.stack.contains(&id)
}

/// The damage marked on the permanent now.
pub fn damage_on(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}
