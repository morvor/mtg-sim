//! Shared helpers for rulings batch P060 (`r_p060_*.rs`): effects that make players
//! draw cards ("force draw"): wheels, symmetrical draws, draws for an opponent, and
//! triggers on drawing. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::game::GameResult;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p007_common::{obj, pl, player_hexproof, pw};
pub use crate::r_s01_common::{give_mana_for, stack_library, supported};

/// The hand sizes of all players, P0 first.
pub fn hands(t: &TestGame) -> Vec<usize> {
    (0..t.g.players.len())
        .map(|i| t.hand_size(PlayerId(i as u8)))
        .collect()
}

/// The library sizes of all players, P0 first.
pub fn libraries(t: &TestGame) -> Vec<usize> {
    (0..t.g.players.len())
        .map(|i| t.library_size(PlayerId(i as u8)))
        .collect()
}

/// The cards each player drew since `before` (from [`libraries`]), measured by how much
/// each library shrank.
pub fn drawn_since(t: &TestGame, before: &[usize]) -> Vec<usize> {
    libraries(t)
        .iter()
        .zip(before)
        .map(|(now, was)| was - now)
        .collect()
}

/// Empties `p`'s library down to `n` cards (removing filler cards from the bottom).
pub fn library_of(t: &mut TestGame, p: PlayerId, n: usize) {
    let lib = &mut t.g.players[p.idx()].library;
    let keep = lib.len().min(n);
    lib.truncate(keep);
}

/// Puts `n` Grizzly Bears into `p`'s hand.
pub fn bears_in_hand(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.hand(p, "Grizzly Bears")).collect()
}

/// Whether the game ended in a draw.
pub fn is_draw(t: &TestGame) -> bool {
    t.g.result == Some(GameResult::Draw)
}

/// Destroys a permanent (as an opponent's removal spell would) and settles.
pub fn destroy(t: &mut TestGame, id: ObjectId) {
    crate::r_s02_common::destroy(t, id);
}
