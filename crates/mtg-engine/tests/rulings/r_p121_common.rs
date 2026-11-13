//! Shared helpers for the tests of rulings batch P121 (`r_p121_*.rs`): "+1/+1 counters
//! matter" cards (counted on resolution, last known information, triggers, cost
//! reductions, distributions), looking at and playing the top card of a library, and
//! effects that stop activated abilities. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p119_common::{
    cast_slots, counter_spell, enter_with, has_kw, stack, with_counters,
};
pub use crate::r_p120_common::{
    attack_with, block_and_finish, cast_new, create_token, destroy, end_step, gain_life,
    give_minus1, give_plus1, lands_for_cost, obj, plus1, put_counters, resolved, supported,
    tokens_named, triggers_on_stack, MINUS1, PLUS1,
};
pub use crate::r_s24_common::enter_together;

/// Removes `n` +1/+1 counters from the permanent and settles.
pub fn remove_plus1(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.remove_counters(Entity::Object(id), PLUS1, n);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Casts the real card `name` from `p`'s hand (lands for its cost are added) with the
/// given targets and resolves the whole stack.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let id = cast_new(t, p, name, targets);
    t.resolve_all();
    id
}

/// Exiles the permanent (followed across zone changes) and settles.
pub fn exile_now(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.exile_object(id, None);
    t.g.flush_events();
    t.settle();
}

/// Marks `n` damage on the creature from no source, as an effect would, and settles.
pub fn damage(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].damage += n;
    t.g.dirty = true;
    t.settle();
}

/// The power of the object (followed across zone changes) now.
pub fn power(t: &mut TestGame, id: ObjectId) -> i32 {
    t.g.recompute();
    t.pt(t.g.current(id)).0
}
