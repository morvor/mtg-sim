//! Shared helpers for the tests of rulings batch P143 (`r_p143_*.rs`): repeatable card
//! advantage — "look at the top N" and impulse-draw abilities, draw replacement, Clues
//! and tokens that enter attacking. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::obj;
pub use crate::r_s01_common::{custom_card, supported};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// Replaces `p`'s library with the real cards `top_first` (the first one on top).
pub fn set_library(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    t.g.players[p.idx()].library.clear();
    let ids: Vec<ObjectId> = top_first.iter().rev().map(|n| t.library_top(p, n)).collect();
    ids.into_iter().rev().collect()
}

/// `p` draws `n` cards (as a resolving effect would), then settles.
pub fn draw(t: &mut TestGame, p: PlayerId, n: u32) -> Vec<ObjectId> {
    let out = t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
    out
}

/// The cards of `p`'s library, top first.
pub fn library_top_first(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.player(p).library.iter().rev().copied().collect()
}

/// Whether the object (followed across zone changes) is in `zone`.
pub fn is_in(t: &TestGame, id: ObjectId, zone: Zone) -> bool {
    t.zone(id) == zone
}

/// The number of decisions asked so far.
pub fn n_asked(t: &TestGame) -> usize {
    t.asked().len()
}

/// The decisions asked of `p` since `from`, rendered for debugging and matching.
pub fn asked_of(t: &TestGame, p: PlayerId, from: usize) -> Vec<Decision> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .map(|(_, d)| d.clone())
        .collect()
}

/// The `(min, max, candidates)` of the entity choices asked of `p` since `from`.
pub fn entity_choices(t: &TestGame, p: PlayerId, from: usize) -> Vec<(u32, u32, Vec<Entity>)> {
    asked_of(t, p, from)
        .into_iter()
        .filter_map(|d| match d {
            Decision::ChooseEntities {
                min,
                max,
                candidates,
                ..
            } => Some((min, max, candidates)),
            _ => None,
        })
        .collect()
}
