//! Shared helpers for the tests of rulings batch P128 (`r_p128_*.rs`): cards that return
//! themselves from the graveyard (casting permissions, activated abilities and triggers
//! that work from the graveyard), recursion from exile, and reflexive triggered abilities
//! ("When you do, ..."). (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p127_common::*;
pub use crate::r_s01_common::{place, tapped_lands, watch};
pub use crate::r_s06_common::give_control;
pub use crate::r_s08_common::{legal_cast_methods, mana_value};

/// Whether `p` may cast the card (in any way) now.
pub fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    !legal_cast_methods(t, p, card).is_empty()
}

/// Casts the card from wherever it is with the first legal method; returns the spell.
pub fn cast_any(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    let m = legal_cast_methods(t, p, card);
    assert!(!m.is_empty(), "no legal way to cast it");
    let card = t.g.current(card);
    t.cast(p, card).method(m[0].clone()).go()
}

/// Whether `p` may activate an ability of `source` (followed across zone changes) now.
pub fn activatable(t: &mut TestGame, p: PlayerId, source: ObjectId) -> bool {
    let s = t.g.current(source);
    can_activate(t, p, s)
}

/// The number of priority decisions anyone was asked since decision `from`.
pub fn priority_asks(t: &TestGame, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Priority { .. }))
        .count()
}

/// The power of the object (followed across zone changes) now.
pub fn power(t: &TestGame, id: ObjectId) -> i32 {
    t.pt(t.g.current(id)).0
}
