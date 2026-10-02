//! Shared helpers for the tests of rulings batch P125 (`r_p125_*.rs`): effects that
//! protect a group of permanents (the group is fixed as the effect resolves), fogs and
//! damage prevention for groups, "blocks [creature] this turn if able" requirements,
//! commander-matters modal spells, and skipping combat. (The helpers of batches S01–P120
//! are used too.)

#![allow(dead_code)]

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, put_counters, resolved};
pub use crate::r_s01_common::{attack_with, block_and_finish, creatures, supported, triggers_on_stack};
pub use crate::r_s02_common::{create_token, destroy};
pub use crate::r_s05_common::run_from;
pub use crate::r_s20_common::to_beginning_of_combat;
pub use crate::r_s21_common::{blocks_now, castable, go_to, legal_blocks};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// Attack declarations of the given creatures, each attacking player P1.
pub fn at_p1(attackers: &[ObjectId]) -> Vec<(ObjectId, Entity)> {
    attackers.iter().map(|a| (*a, Entity::Player(P1))).collect()
}

/// Whether the object (followed across zone changes) has the keyword now.
pub fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

/// Whether the object (followed across zone changes) is indestructible now.
pub fn indestructible(t: &TestGame, id: ObjectId) -> bool {
    has(t, id, KeywordKind::Indestructible)
}

/// Whether the object (followed across zone changes) has hexproof now.
pub fn hexproof(t: &TestGame, id: ObjectId) -> bool {
    has(t, id, KeywordKind::Hexproof)
}

/// Queues `p`'s answer to a "choose a color" decision.
pub fn choose_color(t: &mut TestGame, p: PlayerId, c: mtg_engine::types::Color) {
    let i = mtg_engine::types::Color::ALL
        .iter()
        .position(|x| *x == c)
        .unwrap();
    t.answer(p, DecisionKind::Option, mtg_engine::decision::Answer::Index(i));
}
