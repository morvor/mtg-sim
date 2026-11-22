//! Shared helpers for the tests of rulings batch P068 (`r_p068_*.rs`): more cards that let
//! a player cast cards from exile ("gives castable from exile").

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p067_common::{
    cast_free, castable, exiled, mana, move_to, o, offered_since, playable_land,
    target_named_when_offered, was_offered,
};

/// Adds one mana of each type in `tys` to `p`'s pool.
pub fn pool(t: &mut TestGame, p: PlayerId, tys: &[ManaType]) {
    for ty in tys {
        mana(t, p, *ty, 1);
    }
}

/// Whether anyone was asked to choose a casting method (an alternative cost) since `from`.
pub fn asked_method(t: &TestGame, from: usize) -> bool {
    t.asked()[from..].iter().any(|(_, d)| match d {
        Decision::ChooseCastingMethod { .. } => true,
        Decision::ChooseOption { prompt, .. } => prompt.contains("how to cast"),
        _ => false,
    })
}

/// The methods `p` could cast `card` (followed across zone changes) with right now.
pub fn methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    crate::r_s07_common::cast_methods(t, p, card)
}

/// Whether the object (followed across zone changes) is in `p`'s library.
pub fn in_library(t: &TestGame, id: ObjectId, p: PlayerId) -> bool {
    t.zone(id) == Zone::Library(p)
}

/// The position of `id` (followed across zone changes) in `p`'s library, 0 = top.
pub fn library_pos(t: &TestGame, p: PlayerId, id: ObjectId) -> Option<usize> {
    let now = t.g.current(id);
    let lib = &t.g.player(p).library;
    // Libraries are stored bottom first.
    lib.iter().rev().position(|x| *x == now)
}

/// The number of `p`'s permanents that are tapped lands.
pub fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s01_common::tapped_lands(t, p)
}
