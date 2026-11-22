//! Shared helpers for the tests of rulings batch P019 (`r_p019_*.rs`): copying activated
//! and triggered abilities (CR 707.10), abilities that trigger an additional time (CR
//! 603.2d), "reveal cards until ..." effects that put a card onto the battlefield or let
//! it be cast, control-changing combat abilities, converge and convoke. (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The abilities on the stack (any source) whose text contains `needle`, bottom first.
pub fn abilities_with_text(t: &TestGame, needle: &str) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            matches!(
                t.g.obj(*id).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Activated { ability, .. } | StackKind::Triggered { ability, .. })
                    if ability.text.contains(needle)
            )
        })
        .collect()
}

/// How many decisions matching `f` were asked since index `from` of the decision log.
pub fn asked_count(t: &TestGame, from: usize, f: fn(&Decision) -> bool) -> usize {
    t.asked()[from..].iter().filter(|(_, d)| f(d)).count()
}

/// Whether a ChooseModes or ChooseX decision was asked since `from`.
pub fn asked_modes_or_x(t: &TestGame, from: usize) -> bool {
    asked_count(t, from, |d| {
        matches!(d, Decision::ChooseModes { .. } | Decision::ChooseX { .. })
    }) > 0
}

/// The number of YesNo decisions asked since `from`.
pub fn yes_nos(t: &TestGame, from: usize) -> usize {
    asked_count(t, from, |d| matches!(d, Decision::YesNo { .. }))
}

/// Number of cards of the named card in `p`'s library.
pub fn in_library(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.players[p.idx()]
        .library
        .iter()
        .filter(|id| t.g.obj(**id).chars.name == name)
        .count()
}
