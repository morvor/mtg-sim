//! Shared helpers for the tests of item `rulings-residue-burn-freeze`
//! (`r_residue_burn_freeze_*.rs`): the open rulings left in batches P009 (burn that hits
//! players, planeswalkers and each player) and P062 (free sacrifice outlets, casting
//! without paying, and tapping effects that keep a creature from untapping). The helpers
//! of the batches' earlier rounds are used too.

#![allow(dead_code)]

pub use crate::r_p007_common::{cast_x, divide, dmg, loyalty, obj, pl, player_hexproof, pw};
pub use crate::r_p008_common::{damage_events, hit, hit_player};
pub use crate::r_p062_common::{
    is_tapped, misses_one_untap, tap, tapped_creature, through_untap_step, untaps_next,
};
pub use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, stack_library, supported, tokens,
};
pub use crate::r_s06_common::give_control;
pub use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::testing::*;
use mtg_engine::*;

/// The number of the player's energy counters.
pub fn energy(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter(mtg_engine::types::counters::ENERGY)
}

/// Everyone passes until the stack is empty, then the state settles.
pub fn settle_all(t: &mut TestGame) {
    t.resolve_all();
    t.settle();
}
