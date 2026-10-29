//! Shared helpers for the tests of rulings batch P180 (`r_p180_*.rs`): trigger doublers,
//! triggers from the graveyard, "tunneling" evasion, shuffling a permanent into its
//! owner's library, turning face-down permanents face up, defender and tutors.

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p171_common::{cast_targeting, damage_on, obj};
pub use crate::r_p177_common::cast_resolve;
pub use crate::r_s01_common::{give_mana_for, stack_library, supported};
pub use crate::r_s02_common::{can_attack, destroy};
pub use crate::r_s11_common::{can_turn_face_up, turn_face_up};
pub use crate::r_s12_common::morph;

/// The +1/+1 counters on the object now.
pub fn plus(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(t.g.current(id), mtg_engine::types::counters::PLUS1)
}

/// Whether `blocker` could be declared as a blocker of `attacker` now.
pub fn can_block(t: &mut TestGame, blocker: ObjectId, attacker: ObjectId) -> bool {
    t.g.recompute();
    let (b, a) = (t.g.current(blocker), t.g.current(attacker));
    let dp = t.g.obj(b).controller;
    mtg_engine::combat::block_options(&t.g, &[dp])
        .iter()
        .any(|(c, attackers)| *c == b && attackers.contains(&a))
}
