//! Shared helpers for the tests of rulings batch P208 (`r_p208_*.rs`): enchant.

#![allow(dead_code)]

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use mtg_engine::testing::*;
use mtg_engine::object::Zone;
use mtg_engine::*;

/// `p` casts the real Aura `name` (with lands for its cost) targeting `target` and only
/// the Aura spell resolves: its enters trigger (if any) is left on the stack. The Aura
/// (followed onto the battlefield).
pub fn cast_aura_leave_trigger(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    target: impl Into<Entity>,
) -> ObjectId {
    supported(name);
    let aura = in_hand_with_mana(t, p, name);
    t.g.turn.priority = Some(p);
    t.cast(p, aura).target(target.into()).go();
    t.g.resolve_top();
    t.settle();
    t.g.current(aura)
}

/// Moves the permanent (followed) to its owner's graveyard by an effect, without settling.
pub fn put_in_graveyard(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let owner = t.g.obj(id).owner;
    t.g.move_object(
        id,
        Zone::Graveyard(owner),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.flush_events();
}
