//! Shared helpers for the tests of rulings batch P197 (`r_p197_*.rs`): effects with no
//! duration (CR 611.2) — control changes that outlast the turn and their source, and what
//! ends them (a player leaving the game, CR 800.4a), characteristic changes that last
//! indefinitely, and the timing and cost rulings of the same cards.

#![allow(dead_code)]

pub use crate::r_p053_common::concede;
pub use crate::r_p196_common::*;
pub use crate::r_s01_common::custom_card;
pub use crate::r_s06_common::give_control;
pub use crate::r_s19_common::add_lore;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// A vanilla creature with the given types for `p`, on the battlefield.
pub fn vanilla(t: &mut TestGame, p: PlayerId, name: &str, types: &str, pt: (i32, i32)) -> ObjectId {
    creature_with(t, p, name, types, pt, "")
}

/// A creature with the given types and rules text for `p`, on the battlefield.
pub fn creature_with(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    types: &str,
    pt: (i32, i32),
    text: &str,
) -> ObjectId {
    let def = custom_card(name, types, "{1}", Some(pt), text);
    t.custom(p, def, Zone::Battlefield)
}

/// `n` vanilla 1/1 Vampires for `p`.
pub fn vampires(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n)
        .map(|_| vanilla(t, p, "Test Vampire", "Creature — Vampire", (1, 1)))
        .collect()
}

/// A Saga with no lore counters on P0's battlefield (its chapter abilities trigger as
/// counters are added with [`add_lore`]).
pub fn blank_saga(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    supported(name);
    let s = t.battlefield(p, name);
    t.g.objects[s.0 as usize].counters.clear();
    s
}

/// Puts `p`'s planeswalker `name` onto the battlefield with `loyalty` loyalty counters.
pub fn walker(t: &mut TestGame, p: PlayerId, name: &str, loyalty: u32) -> ObjectId {
    let w = t.battlefield(p, name);
    t.g.objects[w.0 as usize].counters.clear();
    t.g.add_counters(obj(w), "loyalty", loyalty, None);
    t.g.recompute();
    w
}

/// Whether the object (followed across zone changes) is still in the game: false once its
/// owner has left (CR 800.4a).
pub fn in_game(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).zone != Zone::Nowhere
}
