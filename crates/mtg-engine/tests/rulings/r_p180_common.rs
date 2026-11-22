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

/// `p` casts the real card `name` face down with disguise (paying {3} with Wastes) and it
/// resolves. Returns the face-down permanent.
pub fn disguise(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    use mtg_engine::keywords::KeywordKind;
    use mtg_engine::object::CastMethod;
    t.lands(p, "Wastes", 3);
    let card = t.hand(p, name);
    let spell = t
        .cast(p, card)
        .method(CastMethod::FaceDown(KeywordKind::Disguise))
        .go();
    t.resolve_all();
    let id = t.g.current(spell);
    assert!(t.obj(id).face_down && t.on_battlefield(id), "{name} not face down");
    id
}

/// `p` turns the face-down permanent `id` face up (with lands named `lands` for its
/// cost) and the resulting triggered abilities are put on the stack.
pub fn turn_up(t: &mut TestGame, p: PlayerId, id: ObjectId, lands: &[(&str, usize)]) {
    for (land, n) in lands {
        t.lands(p, land, *n);
    }
    assert!(turn_face_up(t, p, id), "couldn't turn it face up");
    t.settle();
}
