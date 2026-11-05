//! Shared helpers for the tests of rulings batch P179 (`r_p179_*.rs`): cast and draw
//! triggers ("toll"), activated card-draw artifacts ("tome"), team pumps, toughness-
//! matters abilities and trigger doublers. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p171_common::{cast_targeting, obj};
pub use crate::r_p177_common::{cast_resolve, ffa};
pub use crate::r_s01_common::{give_mana_for, supported, tokens, triggers_on_stack};
pub use crate::r_s02_common::{can_activate, can_cast, create_token, destroy};
pub use crate::r_s04_common::{stack_items, top_of_stack};
pub use crate::r_s06_common::{activate_containing, damage};
pub use crate::r_s14_common::triggers_from;
pub use crate::r_s24_common::{choose_creature_type, enter_together};
pub use crate::r_s33_common::{set_life, two_headed_giant};

/// The options of every `ChooseOption` decision `p` was asked since decision `from`.
pub fn options_asked_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<String>> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect()
}

/// Every decision `p` was asked since decision `from` (debug-formatted).
pub fn decisions_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .map(|(_, d)| format!("{d:?}"))
        .collect()
}

/// Puts `n` filler cards into `p`'s hand.
pub fn fill_hand(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        let id = t
            .g
            .create_card_object(filler_card(), p, mtg_engine::object::Zone::Hand(p));
        t.g.players[p.idx()].hand.push(id);
    }
}
