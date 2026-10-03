//! Shared helpers for rulings batch P009 (`r_p009_*.rs`): burn spells and abilities that
//! damage players, planeswalkers and creatures.

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// The damage marked on the permanent now (0 if it left the battlefield).
pub fn dmg(t: &TestGame, id: ObjectId) -> u32 {
    if t.on_battlefield(id) {
        t.obj_now(id).damage
    } else {
        0
    }
}

/// `p` loses `n` life (as a resolving effect would), then settles.
pub fn lose(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.lose_life(p, n);
    t.g.flush_events();
    t.settle();
}

/// `p` gains `n` life (as a resolving effect would), then settles.
pub fn gain(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.gain_life(p, n);
    t.g.flush_events();
    t.settle();
}

/// Destroys the permanent and settles.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
}

/// Puts a real card onto the battlefield (as [`TestGame::battlefield`]) and settles.
pub fn add(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.flush_events();
    t.settle();
    id
}
