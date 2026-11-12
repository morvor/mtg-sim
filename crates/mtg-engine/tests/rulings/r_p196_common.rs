//! Shared helpers for the tests of rulings batch P196 (`r_p196_*.rs`): effects with no
//! duration (CR 611.2) — control changes and exchanges (CR 701.12), reanimation that
//! changes characteristics, type- and text-changing effects, and "for the rest of the
//! game" effects.

#![allow(dead_code)]

pub use crate::r_p198_common::*;
pub use crate::r_s06_common::{attach_new, attached_to};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether an ability from `source` resolved this turn (CR 608.2: an ability whose
/// targets are all illegal doesn't resolve, 608.2b).
pub fn ability_resolved_from(t: &TestGame, source: ObjectId) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::AbilityResolved { source: s, .. } if *s == source))
}

/// Gives the permanent +`p`/+`q` until end of turn.
pub fn pump(t: &mut TestGame, id: ObjectId, p: i32, q: i32) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![mtg_engine::ability::Modification::ModifyPT(
            mtg_engine::ability::Value::c(p),
            mtg_engine::ability::Value::c(q),
        )],
    );
}

/// Queues `p`'s answer to a "choose a color" decision.
pub fn choose_color(t: &mut TestGame, p: PlayerId, c: mtg_engine::types::Color) {
    let i = mtg_engine::types::Color::ALL
        .iter()
        .position(|x| *x == c)
        .unwrap();
    t.answer(
        p,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(i),
    );
}
