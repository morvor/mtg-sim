//! Snapshot equivalence: a copy of a game continued with copies of the agents plays out
//! as the original (see `mtg_engine::properties::snapshot_equivalence`).

use super::common::*;
use mtg_engine::properties::*;

#[test]
fn a_copied_game_plays_out_as_the_original() {
    for (i, s) in fixed_set().into_iter().enumerate() {
        // Before the game starts, early, and in the middle of a game.
        for at in [0, 40 + 37 * i as u64, 300 + 101 * i as u64] {
            snapshot_equivalence(&s, at).unwrap_or_else(|e| panic!("seed {}: {e}", s.seed));
        }
    }
}
