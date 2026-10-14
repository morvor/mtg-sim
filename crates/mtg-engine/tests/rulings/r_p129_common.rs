//! Shared helpers for the tests of rulings batch P129 (`r_p129_*.rs`): reflexive triggered
//! abilities ("When you do, ...", CR 603.12) and the cards that have them. (The helpers of
//! batch P130 and of the batches it uses are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::types::{counters, CardType};
use mtg_engine::*;

pub use crate::r_p130_common::*;
pub use crate::r_s01_common::{stack_library, tokens, with_subtype};
pub use crate::r_s04_common::{
    ability_targets, add_mana, cycle, next_upkeep, run_with, spell_targets, stack_items,
};
pub use crate::r_s25_common::{creature_tokens, x_of};
pub use crate::r_s28_common::energy;

/// Gives player `p` `n` energy counters.
pub fn give_energy(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.add_counters(Entity::Player(p), counters::ENERGY, n, None);
}

/// Puts `n` counters of `kind` on the permanent `id`.
pub fn put_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
}

/// The number of crimes `p` has committed this game (CR 700.13).
pub fn crimes(t: &TestGame, p: PlayerId) -> u32 {
    t.g.history.crimes.get(&p).copied().unwrap_or(0)
}

/// Whether the object (followed across zone changes) has the subtype now.
pub fn has_subtype(t: &TestGame, id: ObjectId, subtype: &str) -> bool {
    t.obj_now(id).chars.has_subtype(subtype)
}

/// Whether the object (followed across zone changes) is a creature now.
pub fn is_creature_now(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

/// The "you may ... When you do, [targeted effect]" shape: from the moment the first
/// ability is on the stack (with `p` asked nothing about targets since decision `from`),
/// resolving it puts the reflexive ability on the stack with targets `expect`; returns it.
pub fn untargeted_then_reflexive(
    t: &mut TestGame,
    p: PlayerId,
    from: usize,
    expect: &[Entity],
) -> ObjectId {
    no_target_yet(t, p, from);
    resolve_into_reflexive(t, expect)
}

