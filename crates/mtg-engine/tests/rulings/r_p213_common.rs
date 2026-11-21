//! Shared helpers for the tests of rulings batch P213 (`r_p213_*.rs`): Equipment (equip),
//! escalate, escape, eternalize, evoke and evolve. (The helpers of the S batches are used
//! too.)

#![allow(dead_code)]

use crate::r_s04_common::activate_named;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p` activates the `nth` equip ability of `equipment` (in printed order: "Equip Knight
/// {1}" before "Equip {3}") targeting `target`, with whatever mana and answers are
/// available.
pub fn equip_nth(
    t: &mut TestGame,
    p: PlayerId,
    equipment: ObjectId,
    nth: usize,
    target: ObjectId,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.answer_targets(p, &[Entity::Object(t.g.current(target))]);
    let r = activate_named(t, p, equipment, "Equip", nth);
    if r.is_err() {
        t.clear_answers();
    }
    r
}

/// [`equip_nth`] for the first (usually the only) equip ability.
pub fn equip(
    t: &mut TestGame,
    p: PlayerId,
    equipment: ObjectId,
    target: ObjectId,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    equip_nth(t, p, equipment, 0, target)
}

/// The damage marked on the permanent now.
pub fn marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}
