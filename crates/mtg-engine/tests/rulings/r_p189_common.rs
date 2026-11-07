//! Shared helpers for the tests of rulings batch P189 (`r_p189_*.rs`): "can't be
//! blocked" abilities, undergrowth-style graveyard counts, "becomes tapped" triggers and
//! triggered mana abilities, and unique counters.

#![allow(dead_code)]

use mtg_engine::events::{Event, MoveCause};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `n` copies of the real card `name` into `p`'s graveyard.
pub fn fill_graveyard(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, name)).collect()
}

/// Exiles every card in `p`'s graveyard (as an effect would).
pub fn exile_graveyard(t: &mut TestGame, p: PlayerId) {
    let cards = t.g.player(p).graveyard.clone();
    for c in cards {
        t.g.move_object(c, Zone::Exile, MoveCause::Effect, Some(p));
    }
    t.g.recompute();
    t.g.flush_events();
}

/// Returns `obj` to its owner's hand (as an effect would).
pub fn bounce(t: &mut TestGame, obj: ObjectId) {
    let owner = t.g.obj(obj).owner;
    t.g.move_object(obj, Zone::Hand(owner), MoveCause::Effect, None);
    t.g.recompute();
    t.g.flush_events();
}

/// Empties `p`'s library (moving the fillers to exile), then puts `top_first` on it.
pub fn set_library(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    let lib = t.g.player(p).library.clone();
    for c in lib {
        t.g.move_object(c, Zone::Exile, MoveCause::Effect, None);
    }
    t.g.flush_events();
    crate::r_s01_common::stack_library(t, p, top_first)
}

/// Number of times `source` had a triggered ability put on the stack this turn.
pub fn triggers_of(t: &TestGame, source: ObjectId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::AbilityTriggeredOnStack { source: s, .. } if *s == source))
        .count()
}

/// The ids on the stack that are triggered abilities from `source`.
pub fn stack_triggers_from(t: &TestGame, source: ObjectId) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            t.g.obj(*id).stack.as_ref().is_some_and(|si| {
                matches!(si.kind, mtg_engine::object::StackKind::Triggered { source: s, .. }
                    if s == source)
            })
        })
        .collect()
}
