//! Shared helpers for the tests of rulings batch P127 (`r_p127_*.rs`): returning cards
//! from graveyards to the battlefield (reanimation from any graveyard, of lands and other
//! permanents, and of the card itself), casting cards from graveyards, and finality
//! counters. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p053_common::concede;
pub use crate::r_p108_common::{a_creature_dies, end_step, put_counters, resolved};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, supported, tokens, triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, can_cast, can_play_land, create_token, destroy};
pub use crate::r_s06_common::{activate_containing, attach_new, attached_to, damage};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Moves the object (followed across zone changes) to `to` as an effect would, and settles.
pub fn move_to(t: &mut TestGame, id: ObjectId, to: Zone) -> Option<ObjectId> {
    let id = t.g.current(id);
    let r = t.g.move_object(id, to, MoveCause::Effect, None);
    t.g.flush_events();
    t.settle();
    r
}

/// The object (followed across zone changes) is exiled by an effect.
pub fn exile_now(t: &mut TestGame, id: ObjectId) {
    move_to(t, id, Zone::Exile);
}

/// The object (followed across zone changes) is returned to its owner's hand.
pub fn bounce(t: &mut TestGame, id: ObjectId) {
    let owner = t.obj_now(id).owner;
    move_to(t, id, Zone::Hand(owner));
}

/// Sacrifices the permanent (followed across zone changes) and settles.
pub fn sacrifice(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let p = t.g.obj(id).controller;
    t.g.sacrifice(id, p);
    t.g.flush_events();
    t.settle();
}

/// The permanents named `name` controlled by `p`.
pub fn named_by(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// Whether the object (followed across zone changes) is in `zone` now.
pub fn is_in(t: &TestGame, id: ObjectId, zone: Zone) -> bool {
    t.zone(id) == zone
}

/// Taps / untaps the permanent (followed across zone changes) and settles triggers.
pub fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.tap(id);
    t.g.flush_events();
    t.settle();
}
pub fn untap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.untap(id);
    t.g.flush_events();
    t.settle();
}

/// The number of `kind` counters on the object (followed across zone changes).
pub fn finality(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, "finality")
}

/// Whether the object (followed across zone changes) is a creature now.
pub fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

/// The yes/no questions asked of `p` since decision `from`.
pub fn yes_no_asked(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
        .count()
}
