//! Shared helpers for the tests of rulings batch P142 (`r_p142_*.rs`): repeatable card
//! advantage — "reveal until" and "look at the top card" engines, recursion from the
//! graveyard, cascade-granting permanents, cost reducers and draw replacements. Batch P141
//! covered the same cards, so its helpers are reused.

#![allow(dead_code)]

pub use crate::r_p016_common::lots;
pub use crate::r_p141_common::*;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The names of the cards in `p`'s library, top first.
pub fn library_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .rev()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}

/// The name of the top card of `p`'s library.
pub fn top_name(t: &TestGame, p: PlayerId) -> String {
    library_names(t, p).first().cloned().unwrap_or_default()
}

/// The number of times a library was shuffled this turn.
pub fn shuffles(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { .. }))
        .count()
}

/// The names of the cards in `p`'s library, sorted (to compare contents, not order).
pub fn library_contents(t: &TestGame, p: PlayerId) -> Vec<String> {
    let mut v = library_names(t, p);
    v.sort();
    v
}

/// Queues a "no" for `p`'s next yes/no decision.
pub fn no(t: &mut TestGame, p: PlayerId) {
    t.answer_yes(p, false);
}
