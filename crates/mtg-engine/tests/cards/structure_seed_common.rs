//! Helpers for the structure-coverage seed tests (`structure_seed_*.rs`): tests of the
//! ability structures shared by the most cards that no other test exercised (see
//! `docs/STRUCTURE_COVERAGE.md`).

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// Asserts that a real card's oracle text is fully understood by the compiler.
pub fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// Queues the active player's attack declaration.
pub fn declare(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
}

/// Queues a defending player's block declaration.
pub fn block(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
}

/// Advances until `step` of the current turn begins and the active player has priority.
pub fn go_to(t: &mut TestGame, step: Step) {
    let ap = t.g.turn.active;
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == step && g.turn.stage == Stage::Priority && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(
        ok && t.g.turn.number == turn,
        "did not reach {step:?} this turn"
    );
}

/// The numbers of cards P0 was asked to surveil (one entry per surveil decision).
pub fn surveils(t: &TestGame) -> Vec<usize> {
    t.asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::Surveil { cards } if *p == P0 => Some(cards.len()),
            _ => None,
        })
        .collect()
}

/// The numbers of cards P0 was asked to scry.
pub fn scries(t: &TestGame) -> Vec<usize> {
    t.asked()
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::Scry { cards } if *p == P0 => Some(cards.len()),
            _ => None,
        })
        .collect()
}

/// The number of permanents named `name` controlled by `p`.
pub fn count_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj_now(*id).controller == p)
        .count()
}

/// The number of permanents with subtype `sub` controlled by `p`.
pub fn count_subtype(t: &TestGame, p: PlayerId, sub: &str) -> usize {
    subtype_ids(t, p, sub).len()
}

/// The permanents with subtype `sub` controlled by `p`.
pub fn subtype_ids(t: &TestGame, p: PlayerId, sub: &str) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|id| t.obj_now(*id).controller == p && t.obj_now(*id).chars.has_subtype(sub))
        .collect()
}
