//! Determinism: the same seed and the same decisions give the same game (see
//! `mtg_engine::properties::determinism`; replays through the protocol are tested in
//! `mtg-api/tests/replay.rs`).

use super::common::*;
use mtg_engine::properties::*;

fn check(i: usize) {
    let s = &fixed_set()[i];
    determinism(s).unwrap_or_else(|e| panic!("seed {}: {e}", s.seed));
}

#[test]
fn the_same_two_player_game_twice_is_the_same_game() {
    check(0);
    check(1);
}

#[test]
fn the_same_multiplayer_game_twice_is_the_same_game() {
    check(2);
    check(3);
}
