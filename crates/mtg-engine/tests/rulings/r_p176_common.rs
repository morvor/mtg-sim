//! Shared helpers for the tests of rulings batch P176 (`r_p176_*.rs`): casting spells from
//! other players' hands, libraries and graveyards ("theft-cast"), and gaining control of
//! creatures (control-changing effects and their durations, reanimating opponents' cards,
//! replacement effects that exile opponents' dying creatures). (The helpers of batches
//! S01–S35 and P171/P177 are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p177_common::*;
pub use crate::r_s01_common::{attack_with, block_and_finish, triggers_on_stack};

/// P0 casts the real card `name` from hand (paid with basic lands) with the given
/// targets, leaving it on the stack.
pub fn cast_only(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    cast_targeting(t, p, name, targets)
}

/// The real card `name` enters the battlefield under `p`'s control and its triggered
/// abilities are put on the stack (targets from the queued answers), unresolved.
pub fn enter_and_trigger(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    supported(name);
    let id = t.enter(p, name);
    t.settle();
    id
}
