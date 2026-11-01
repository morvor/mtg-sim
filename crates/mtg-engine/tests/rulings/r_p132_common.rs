//! Shared helpers for rulings batch P132 (`r_p132_*.rs`): cards that return creature
//! cards (or themselves) from a graveyard to their owner's hand, and mass bounce.

#![allow(dead_code)]

use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Flushes events and puts any triggered abilities on the stack.
pub fn flush(t: &mut TestGame) {
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Moves the card (followed across zone changes) to `zone` as an effect would, then
/// settles; returns its new id.
pub fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) -> ObjectId {
    let id = t.g.current(id);
    let new = t
        .g
        .move_object(id, zone, MoveCause::Effect, None)
        .expect("moved");
    flush(t);
    new
}

/// Exiles a card from its graveyard and puts it back (a new object, CR 400.7).
pub fn blink_in_graveyard(t: &mut TestGame, id: ObjectId) -> ObjectId {
    let owner = t.obj_now(id).owner;
    let e = move_to(t, id, Zone::Exile);
    move_to(t, e, Zone::Graveyard(owner))
}

/// Whether the card (followed across zone changes) is now in its owner's `zone`
/// ("hand", "graveyard", "battlefield", "exile").
pub fn where_is(t: &TestGame, id: ObjectId) -> &'static str {
    let o = t.obj_now(id);
    match o.zone {
        Zone::Hand(_) => "hand",
        Zone::Graveyard(_) => "graveyard",
        Zone::Battlefield => "battlefield",
        Zone::Exile => "exile",
        Zone::Library(_) => "library",
        Zone::Stack => "stack",
        _ => "other",
    }
}

/// `p` gains `n` life (as an effect would), and triggers are put on the stack.
pub fn gain(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.gain_life(p, n);
    flush(t);
}

/// `source` deals `n` damage to `to`, then state-based actions and triggers.
pub fn damage(t: &mut TestGame, source: ObjectId, to: Entity, n: u32) {
    let source = t.g.current(source);
    t.g.deal_damage(source, to, n, false);
    flush(t);
}

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}
