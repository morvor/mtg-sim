//! Shared helpers for the tests of rulings batch P152 (`r_p152_*.rs`): permanents that
//! sacrifice themselves (as a cost or as part of a triggered ability's effect), and
//! abilities scaling with damage dealt or with mana value. (The helpers of earlier
//! batches are used too.)

#![allow(dead_code)]

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, put_counters, two_headed_giant};
pub use crate::r_p117_common::{exile, gain_control};
pub use crate::r_p151_common::{act, damage, next_upkeep, reanimate, steal_for_good};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, stack_library, supported, tokens, triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, destroy, target_candidates};
pub use crate::r_s13_common::{commander, commander_game};
pub use crate::r_s25_common::cast_new;

/// Adds `n` mana of each of `types` to `p`'s pool.
pub fn pool(t: &mut TestGame, p: PlayerId, types: &[ManaType], n: u32) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add_type(*ty, n);
    }
}

/// Sets `p`'s life total.
pub fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    t.g.players[p.idx()].life = life;
}

/// The number of permanents named `name` on the battlefield.
pub fn count_named(t: &TestGame, name: &str) -> usize {
    t.named_on_battlefield(name).len()
}
