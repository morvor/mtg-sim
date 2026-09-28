//! Shared helpers for the tests of rulings batch S28 (`r_s28_*.rs`): counters — energy,
//! rad, experience and poison counters players have (CR 122.1), shield and stun counters
//! (CR 122.1c, 122.1d), counters on permanents and how effects modify them, and spells and
//! abilities that are countered. (The helpers of batches S01–S25 are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

/// The number of counters of `kind` player `p` has.
pub fn player_counters(t: &TestGame, p: PlayerId, kind: &str) -> u32 {
    t.g.player(p).counter(kind)
}

/// The number of energy counters player `p` has.
pub fn energy(t: &TestGame, p: PlayerId) -> u32 {
    player_counters(t, p, "energy")
}

/// Puts the real card `name` into `p`'s hand with lands for its mana cost and casts it
/// (the answers for its choices are queued by the caller). Returns the spell.
pub fn cast_card(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    crate::r_s25_common::lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    t.cast(p, card).go()
}
