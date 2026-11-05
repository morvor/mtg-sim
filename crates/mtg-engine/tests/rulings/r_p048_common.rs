//! Shared helpers for the tests of rulings batch P048 (`r_p048_*.rs`): giving permanents
//! and tokens to other players ("its controller creates a token", "target player gains
//! control of"), the D&D monsters and mechanics, and "doesn't untap". (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p053_common::concede;
pub use crate::r_p124_common::*;
pub use crate::r_p127_common::{move_to, named_by, tap};

/// The tokens `p` controls with the subtype `subtype`.
pub fn tokens_of(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .count()
}

/// The tokens with the subtype `subtype` every player controls, by player.
pub fn tokens_by_player(t: &TestGame, subtype: &str) -> Vec<usize> {
    (0..t.g.players.len())
        .map(|i| tokens_of(t, PlayerId(i as u8), subtype))
        .collect()
}

/// The controller of the permanent (followed across zone changes) now.
pub fn controller(t: &mut TestGame, id: ObjectId) -> PlayerId {
    t.g.recompute();
    t.obj_now(id).controller
}

/// The permanent's (followed across zone changes) activated ability whose text contains
/// `needle` is activated by `p` with the given targets (one per slot).
pub fn activate_with(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    needle: &str,
    targets: &[Entity],
) -> Option<ObjectId> {
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    activate_containing(t, p, source, needle)
        .unwrap_or_else(|e| panic!("activating {needle:?} failed: {e:?}"))
}
