//! Shared helpers for the tests of rulings batch P093 (`r_p093_*.rs`): cards that trade
//! life for cards ("you draw a card and you lose 1 life", "you may pay N life. If you do,
//! draw"). (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p044_common::{cast_only, cast_spell, kill, wrath};
pub use crate::r_p108_common::put_counters;
pub use crate::r_s01_common::{attack_with, block_and_finish, supported, tokens};
pub use crate::r_s04_common::add_mana;
pub use crate::r_s25_common::cast_new;
pub use crate::r_s26_common::modify_until_eot;
pub use crate::r_s33_common::set_life;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The number of yes/no questions asked of `p` since decision `from`.
pub fn yes_nos(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

/// The number of number or X choices asked since decision `from`.
pub fn numbers_asked(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseNumber { .. } | Decision::ChooseX { .. }))
        .count()
}

/// The number of cards `p` drew since index `from` of this turn's events.
pub fn drawn_since(t: &TestGame, from: usize, p: PlayerId) -> usize {
    t.g.turn_events[from..]
        .iter()
        .filter(|e| matches!(e, Event::Drew { player, .. } if *player == p))
        .count()
}

/// The amounts of the life-loss events of `p` since index `from` of this turn's events.
pub fn losses_since(t: &TestGame, from: usize, p: PlayerId) -> Vec<u32> {
    crate::r_p096_common::losses_since(t, from, p)
}

/// The number of triggered or activated abilities on the stack whose source is `source`.
pub fn from_source(t: &TestGame, source: ObjectId) -> usize {
    crate::r_s25_common::abilities_from(t, source).len()
}
