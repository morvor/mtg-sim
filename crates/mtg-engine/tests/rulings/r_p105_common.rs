//! Shared helpers for the tests of rulings batch P105 (`r_p105_*.rs`): assorted rulings of
//! cards that mill themselves (Scryfall Tagger "mill-self") and of mana abilities with a
//! rider ("mini refund"). The helpers of batches P104 and S01–S35 are used too.

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Sets the number of `kind` counters on the object (no events, as a test setup).
pub fn set_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].counters.insert(kind.into(), n);
    t.g.dirty = true;
    t.g.recompute();
}

/// Sets a planeswalker's loyalty.
pub fn set_loyalty(t: &mut TestGame, id: ObjectId, n: u32) {
    set_counters(t, id, counters::LOYALTY, n);
}

/// Taps or untaps the permanent (no events, as a test setup).
pub fn set_tapped(t: &mut TestGame, id: ObjectId, tapped: bool) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].tapped = tapped;
}

/// The names of the cards in `p`'s graveyard, bottom first.
pub fn gy_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .graveyard
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}

/// `p`'s mana pool total.
pub fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

/// Whether the attacking creature (followed across zone changes) is blocked.
pub fn blocked(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.combat
        .as_ref()
        .is_some_and(|c| c.attackers.iter().any(|a| a.id == id && a.blocked))
}
