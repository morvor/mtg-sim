//! Shared helpers for the tests of rulings batch P118 (`r_p118_*.rs`): "power matters"
//! abilities of a creature that look at its own power — last known information, values
//! fixed as an ability resolves, comparisons checked only as blockers are declared, and
//! targets whose legality depends on the source's power. (The helpers of earlier batches
//! are used too.)

#![allow(dead_code)]

use mtg_engine::ability::{Modification, Value};
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p117_common::{
    attack_with, beginning_of_combat, block_and_finish, cast_new, counter_spell, damage,
    destroy, exile, gain_control, gain_life, giant_growth, give_minus1, give_plus1,
    grow_then_exile, lost, minus_power, modify, obj, plus1, put_counters, resolved, stack,
    supported, tokens_named, triggers_on_stack, with_counters,
};
pub use crate::r_p147_common::{
    at, attacking, blocking, declare_blocks, enter_attacking, legal_attack, legal_blocks, to_combat,
    two_headed_giant,
};
pub use crate::r_p116_common::{n_asked, priority_asked_since, set_life};
pub use crate::r_p119_common::{enter_with, has_kw, pool_total};
pub use crate::r_s25_common::{lands_for_cost, x_of};
pub use crate::r_s02_common::{can_activate, can_cast, can_play_land, target_candidates};

/// Gives `id` +n/+0 until end of turn (an effect that resolved, not a counter).
pub fn plus_power(t: &mut TestGame, id: ObjectId, n: i32) {
    modify(t, id, vec![Modification::ModifyPT(Value::c(n), Value::c(0))]);
}

/// Whether `attacker` is blocked in the current combat.
pub fn is_blocked(t: &TestGame, attacker: ObjectId) -> bool {
    t.g.combat.as_ref().is_some_and(|c| c.is_blocked(attacker))
}

/// The number of `kind` counters `p` has.
pub fn player_counters(t: &TestGame, p: PlayerId, kind: &str) -> u32 {
    t.g.player(p).counters.get(kind).copied().unwrap_or(0)
}
