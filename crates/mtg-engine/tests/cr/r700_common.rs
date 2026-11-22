//! Shared helpers for the CR 700 tests (general rules for keyword actions and game terms).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::eval::Ctx;
use mtg_engine::object::ChosenMode;
use mtg_engine::testing::*;
use mtg_engine::types::*;

pub use crate::r703_common::{bear, oracle_card, supported};

/// Performs `effect` as if a resolving ability controlled by `p` (with source `source`)
/// did, then checks for triggers.
pub fn run(t: &mut TestGame, p: PlayerId, source: Option<ObjectId>, effect: Effect) {
    let mut ctx = Ctx::new(source, p);
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// The modes and targets chosen for a spell or ability on the stack.
pub fn chosen(t: &TestGame, id: ObjectId) -> Vec<ChosenMode> {
    t.g.obj(id)
        .stack
        .as_deref()
        .map(|si| si.chosen.clone())
        .unwrap_or_default()
}

/// The chosen mode indices of a stack object, in order.
pub fn modes_of(t: &TestGame, id: ObjectId) -> Vec<usize> {
    chosen(t, id).iter().filter_map(|c| c.mode).collect()
}

/// Every target chosen for a stack object.
pub fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    chosen(t, id)
        .iter()
        .flat_map(|m| m.targets.iter().flatten().copied())
        .collect()
}

/// The top object of the stack.
pub fn top(t: &TestGame) -> ObjectId {
    *t.g.stack.last().expect("empty stack")
}

/// Number of decisions of a kind asked of `p` so far.
pub fn times_asked(t: &TestGame, p: PlayerId, pred: impl Fn(&Decision) -> bool) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && pred(d))
        .count()
}
