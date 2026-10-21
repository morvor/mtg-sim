//! Determinism: the same seed and the same decisions give the same game (see
//! `mtg_engine::properties::determinism`; replays through the protocol are tested in
//! `mtg-api/tests/replay.rs`).

use super::common::*;
use mtg_engine::properties::*;

#[test]
fn the_same_game_twice_is_the_same_game() {
    for s in fixed_set() {
        determinism(&s).unwrap_or_else(|e| panic!("seed {}: {e}", s.seed));
    }
}
