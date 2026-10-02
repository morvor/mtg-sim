//! Shared helpers for the tests of rulings batch P108 (`r_p108_*.rs`): modal spells and
//! abilities, monarch, monstrosity, morbid ("if a creature died this turn"), moving and
//! copying counters, "look at the top N, put one into your hand and the rest into your
//! graveyard", and multi-land ramp. (The helpers of batches S01–S30 are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub use crate::r_s01_common::{supported, tokens};
pub use crate::r_s02_common::{create_token, destroy};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// A creature controlled by `p` dies (a Grizzly Bears is put onto the battlefield and
/// destroyed).
pub fn a_creature_dies(t: &mut TestGame, p: PlayerId) {
    let bears = t.battlefield(p, "Grizzly Bears");
    destroy(t, bears);
    assert!(t.in_graveyard(p, "Grizzly Bears"));
}

/// Advances to `p`'s end step (triggered abilities are put on the stack) and resolves the
/// stack.
pub fn end_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::End);
    t.resolve_all();
}

/// Puts `n` counters of `kind` on the object (followed across zone changes) and settles.
pub fn put_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// The creature tokens `p` controls with the given subtype.
pub fn tokens_with(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .count()
}

/// A Two-Headed Giant game: P0 and P1 against P2 and P3.
pub fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}
