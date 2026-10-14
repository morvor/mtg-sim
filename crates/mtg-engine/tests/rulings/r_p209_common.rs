//! Shared helpers for the tests of rulings batch P209 (`r_p209_*.rs`): enchant.

#![allow(dead_code)]

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p` casts the real Aura `name` (with lands for its cost) targeting `target`, and
/// everything resolves. The Aura (followed to its new zone), if it could be cast.
pub fn cast_aura(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    target: impl Into<Entity>,
) -> Option<ObjectId> {
    supported(name);
    let aura = in_hand_with_mana(t, p, name);
    t.g.turn.priority = Some(p);
    let r = t.cast(p, aura).target(target.into()).try_go().ok();
    t.resolve_all();
    r.map(|_| t.g.current(aura))
}

/// `p` casts the real spell `name` (with lands for its cost) with the given targets
/// (one per slot), leaving it on the stack.
pub fn cast_spell(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    let c = in_hand_with_mana(t, p, name);
    t.cast_with(p, c, targets).expect("couldn't cast")
}

/// Taps the permanent as an effect would, then settles (triggers go on the stack).
pub fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.tap(id);
    t.g.flush_events();
    t.settle();
}

/// Untaps the permanent as an effect would, then settles.
pub fn untap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.untap(id);
    t.g.flush_events();
    t.settle();
}
