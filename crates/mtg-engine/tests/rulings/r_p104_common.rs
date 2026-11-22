//! Shared helpers for the tests of rulings batch P104 (`r_p104_*.rs`): milling (yourself
//! and opponents) and what's done with the milled cards.

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `p` mills `n` cards (as a resolving effect would), then settles.
pub fn mill(t: &mut TestGame, p: PlayerId, n: u32) -> Vec<ObjectId> {
    let milled = t.g.mill(p, n);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    milled
}

/// Puts `n` filler cards into `p`'s graveyard from their library (one mill).
pub fn graveyard_of(t: &mut TestGame, p: PlayerId, n: u32) {
    mill(t, p, n);
}

/// Whether the object (followed across zone changes) is face down now.
pub fn face_down(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).face_down
}

/// The objects in exile owned by `p`.
pub fn exiled_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.exile
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).owner == p)
        .collect()
}

/// Number of decisions of the given kind asked of `p` since index `from` of the log.
pub fn asked_kind(t: &TestGame, p: PlayerId, from: usize, f: fn(&Decision) -> bool) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && f(d))
        .count()
}

/// Whether the object (followed across zone changes) is in `zone`.
pub fn is_in(t: &TestGame, id: ObjectId, zone: Zone) -> bool {
    t.zone(id) == zone
}

