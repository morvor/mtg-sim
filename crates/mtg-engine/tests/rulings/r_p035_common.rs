//! Shared helpers for the tests of rulings batch P035 (`r_p035_*.rs`): "counters matter"
//! cards — abilities that count, check, add or remove counters of any kind. (The helpers
//! of batches S01–S30 are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts `n` counters of `kind` on the object (followed across zone changes) or player,
/// as an effect would, and settles.
pub fn put(t: &mut TestGame, e: impl Into<Entity>, kind: &str, n: u32) {
    let e = match e.into() {
        Entity::Object(id) => Entity::Object(t.g.current(id)),
        other => other,
    };
    t.g.add_counters(e, kind, n, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// P1 casts Lightning Bolt (with exactly {R} available) at `target`, and the stack settles
/// with the ward trigger (if any) on top of it. Returns the Bolt.
pub fn opponent_bolts(t: &mut TestGame, target: ObjectId) -> ObjectId {
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(target).go();
    t.settle();
    bolt
}

/// Advances `p`'s turn to its end step and puts the "at the beginning of your end step"
/// triggers on the stack.
pub fn to_end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.settle();
}

/// Total number of counters (of every kind) on the object.
pub fn total_counters(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).counters.values().sum()
}
