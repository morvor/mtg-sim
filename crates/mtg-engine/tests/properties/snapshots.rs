//! Snapshot equivalence: a copy of a game continued with copies of the agents plays out
//! as the original (see `mtg_engine::properties::snapshot_equivalence`).

use super::common::*;
use mtg_engine::properties::*;

/// The game `i` of the fixed set, copied before it starts and in the middle.
fn check(i: usize) {
    let s = &fixed_set()[i];
    for at in [0, 120 + 101 * i as u64] {
        snapshot_equivalence(s, at).unwrap_or_else(|e| panic!("seed {}: {e}", s.seed));
    }
}

#[test]
fn a_copied_two_player_game_plays_out_as_the_original() {
    check(0);
    check(1);
}

#[test]
fn a_copied_three_player_game_plays_out_as_the_original() {
    check(2);
}

#[test]
fn a_copied_four_player_game_plays_out_as_the_original() {
    check(3);
}
