//! Shared helpers for the tests of rulings batch P213 (`r_p213_*.rs`): Equipment (equip),
//! escalate, escape, eternalize, evoke and evolve. (The helpers of the S batches are used
//! too.)

#![allow(dead_code)]

use crate::r_s06_common::activate_containing;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p` activates the equip ability of `equipment` (whose text contains `needle`, usually
/// "Equip") targeting `target`, with whatever mana and answers are available.
pub fn equip_with(
    t: &mut TestGame,
    p: PlayerId,
    equipment: ObjectId,
    needle: &str,
    target: ObjectId,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.answer_targets(p, &[Entity::Object(t.g.current(target))]);
    let r = activate_containing(t, p, equipment, needle);
    if r.is_err() {
        t.clear_answers();
    }
    r
}

/// [`equip_with`] for the plain "Equip" ability.
pub fn equip(
    t: &mut TestGame,
    p: PlayerId,
    equipment: ObjectId,
    target: ObjectId,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    equip_with(t, p, equipment, "Equip", target)
}

/// The damage marked on the permanent now.
pub fn marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}
