//! Shared helpers for rulings batch P038 (`r_p038_*.rs`): effects that count creatures
//! (CR 608.2h), creating bigger bodies, phasing and "until ~ leaves" exile.

#![allow(dead_code)]

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// `p` casts the real card `name` from hand (with the mana for it) with the given targets,
/// leaving it on the stack.
pub fn cast_only(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    give_mana_for(t, p, name);
    let s = t.hand(p, name);
    t.g.turn.priority = Some(p);
    let b = t.cast(p, s);
    let b = if targets.is_empty() {
        b
    } else {
        b.targets(targets)
    };
    b.go()
}

/// `p` casts `name` and everything resolves.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let s = cast_only(t, p, name, targets);
    t.resolve_all();
    s
}

/// Puts a real card onto the battlefield (no ETB) and settles.
pub fn add(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.flush_events();
    t.settle();
    id
}

/// `n` vanilla 2/2 creatures for `p`.
pub fn bears(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(p, "Grizzly Bears")).collect()
}

/// A real card enters the battlefield (ETB triggers fire) and its triggers go on the stack.
pub fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    supported(name);
    let id = t.enter(p, name);
    t.g.flush_events();
    t.settle();
    id
}

/// Destroys a permanent (as a response would) and settles.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
}

/// Exiles a permanent (as a response would) and settles.
pub fn exile_now(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.exile_object(id, None);
    t.g.flush_events();
    t.settle();
}

/// Taps a permanent (as a response would).
pub fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.tap(id);
    t.g.flush_events();
}

pub fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// The creatures on the battlefield (phased in) controlled by `p`.
pub fn creature_count(t: &TestGame, p: PlayerId) -> usize {
    creatures(t, p).len()
}

/// Advances from P0's turn through P1's turn to P0's next upkeep.
pub fn next_p0_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
}

/// The total mana in `p`'s pool.
pub fn pool_total(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).mana_pool.total() as u32
}
