//! Shared helpers for the tests of rulings batch P045 (`r_p045_*.rs`): discard effects —
//! who chooses and when (CR 101.4, 701.9), discarding fewer cards than asked, discard
//! costs (CR 118, 601.2h) and the abilities around them. (The helpers of batch S01 are
//! used too.)

#![allow(dead_code)]

use crate::r_s01_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether a decision is a player's choice of cards to discard.
pub fn is_discard_choice(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("discard"))
}

/// The players asked to choose cards to discard since decision `from`, in order.
pub fn discard_choosers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    asked_since(t, from)
        .into_iter()
        .filter(|(_, d)| is_discard_choice(d))
        .map(|(p, _)| p)
        .collect()
}

/// Every player's hand size.
pub fn hand_sizes(g: &mtg_engine::game::Game) -> Vec<usize> {
    g.players.iter().map(|p| p.hand.len()).collect()
}

/// Records every player's hand size whenever `p` is asked to choose cards to discard.
pub fn watch_discards(t: &mut TestGame, p: PlayerId) -> Seen<Vec<usize>> {
    watch(t, p, is_discard_choice, hand_sizes)
}

/// Puts the named cards into `p`'s hand.
pub fn give_hand(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.hand(p, n)).collect()
}

/// Puts `n` Grizzly Bears into `p`'s hand.
pub fn bears_in_hand(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.hand(p, "Grizzly Bears")).collect()
}
