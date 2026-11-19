//! Shared helpers for the tests of item `rulings-residue-counters` (`r_residue_counters_*.rs`):
//! the open rulings left in batches P035, P119, P120 and P121 — counters being put,
//! removed, moved, doubled and counted. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p002_common::replacement_choices;
pub use crate::r_p035_common::{put, to_end_step, total_counters};
pub use crate::r_p103_common::prefer_replacement;
pub use crate::r_p108_common::{end_step, obj, put_counters, resolved, two_headed_giant};
pub use crate::r_p119_common::{cast_slots, enter_with, giant_growth, with_counters};
pub use crate::r_p120_common::{gain_life, give_minus1, give_plus1, plus1, tokens_named};
pub use crate::r_p121_common::{cast_resolve, damage, exile_now, power, remove_plus1};
pub use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, custom_card, place, stack_library, supported,
    triggers_on_stack,
};
pub use crate::r_s02_common::{create_token, destroy};
pub use crate::r_s24_common::{controller, enter_together, tapped};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// The damage marked on the object (followed across zone changes).
pub fn marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(t.g.current(id)).damage
}

/// Removes `n` counters of `kind` from the permanent as one event and settles.
pub fn remove(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.remove_counters(Entity::Object(id), kind, n);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Whether a decision of the given kind was asked of `p` since decision `from`.
pub fn was_asked(t: &TestGame, p: PlayerId, from: usize, kind: fn(&Decision) -> bool) -> bool {
    t.asked()[from..].iter().any(|(q, d)| *q == p && kind(d))
}
