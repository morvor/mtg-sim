//! Shared helpers for the tests of rulings batch P206 (`r_p206_*.rs`): domain, doubling
//! (power and toughness, counters, damage, life totals), double strike, dredge, earthbend,
//! echo. (The helpers of the S batches are used too.)

#![allow(dead_code)]

use mtg_engine::ability::{Modification, Value};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Gives the permanent +p/+t (or -p/-t) until end of turn, as a resolving spell would.
pub fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
    );
}

/// Adds `n` colorless mana and one mana of each color to `p`'s pool.
pub fn rainbow_pool(t: &mut TestGame, p: PlayerId, n: u32) {
    use crate::r_s04_common::add_mana;
    add_mana(t, p, ManaType::C, n);
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ] {
        add_mana(t, p, ty, 2);
    }
}

/// Empties `p`'s mana pool.
pub fn empty_pool(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].mana_pool = Default::default();
}

/// The +1/+1 counters on the object (followed across zone changes).
pub fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}
