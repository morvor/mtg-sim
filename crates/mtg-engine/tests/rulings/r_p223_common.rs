//! Shared helpers for the tests of rulings batch P223 (`r_p223_*.rs`): scry, spell
//! mastery, splice and a few others.

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The scries asked since decision `from`: (player, number of cards looked at).
pub fn scries_since(t: &TestGame, from: usize) -> Vec<(PlayerId, usize)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::Scry { cards } => Some((*p, cards.len())),
            _ => None,
        })
        .collect()
}

/// The sizes of the scries `p` was asked since decision `from`.
pub fn scry_sizes(t: &TestGame, p: PlayerId, from: usize) -> Vec<usize> {
    scries_since(t, from)
        .into_iter()
        .filter(|(q, _)| *q == p)
        .map(|(_, n)| n)
        .collect()
}

/// Number of "scry" events (CR 701.22d: what "whenever you scry" abilities see) for `p`
/// this turn.
pub fn scry_events(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name, player: Some(q), .. } if name == "scry" && *q == p))
        .count()
}

/// Index of the first scry decision asked since `from`, if any.
pub fn first_scry(t: &TestGame, from: usize) -> Option<usize> {
    t.asked()
        .iter()
        .enumerate()
        .skip(from)
        .find(|(_, (_, d))| matches!(d, Decision::Scry { .. }))
        .map(|(i, _)| i)
}
