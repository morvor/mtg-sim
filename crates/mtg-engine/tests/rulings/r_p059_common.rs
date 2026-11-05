//! Shared helpers for the tests of rulings batch P059 (`r_p059_*.rs`): flicker effects that
//! return the card later (at the beginning of the next end step), fling effects, flowstone
//! pumps, and "attacks (or blocks) this turn if able" requirements. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use crate::r_s01_common::custom_card;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// A historic, nonland permanent (a legendary artifact creature) with "At the beginning
/// of each end step, you gain 1 life." under `p`'s control.
pub fn end_step_keeper(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let def = custom_card(
        "Dusk Keeper",
        "Legendary Artifact Creature — Construct",
        "{2}",
        Some((1, 1)),
        "At the beginning of each end step, you gain 1 life.",
    );
    t.custom(p, def, Zone::Battlefield)
}

/// Whether the token is gone: neither on the battlefield nor in exile (CR 111.8).
pub fn token_gone(t: &TestGame, token: ObjectId) -> bool {
    let z = t.zone(token);
    z != Zone::Battlefield && !t.g.exile.contains(&t.g.current(token))
}

/// Advances to the beginning of `p`'s end step and resolves everything there.
pub fn end_step_of(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.resolve_all();
}
