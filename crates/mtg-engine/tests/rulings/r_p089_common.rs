//! Shared helpers for the tests of rulings batch P089 (`r_p089_*.rs`): high X, homeward
//! effects, "humble" effects (lose all abilities, base power and toughness), hunger
//! (creatures dealt damage by this creature die), Illusions, impact effects, imprint,
//! improvise and impulse. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_s01_common::{attack_with, supported};
pub use crate::r_s06_common::{attach_new, give_control};
pub use crate::r_s20_common::to_beginning_of_combat;
pub use crate::r_s25_common::cast_new;
pub use crate::r_s29_common::{cast_and_resolve, put_counters};

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn player(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// The permanent's (followed across zone changes) abilities' texts now.
pub fn ability_texts(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .map(|a| a.text.to_string())
        .collect()
}

/// Whether the permanent (followed across zone changes) has no abilities now.
pub fn no_abilities(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.abilities.is_empty()
}

/// From P0's beginning of combat: P0 attacks `defender` with `attacker`; the attack
/// triggers are put on the stack (targets answered from the queue) and all resolve.
pub fn attack_and_resolve(t: &mut TestGame, attacker: ObjectId, defender: PlayerId) {
    to_beginning_of_combat(t, P0);
    attack_with(t, &[(attacker, Entity::Player(defender))]);
    t.resolve_all();
}
