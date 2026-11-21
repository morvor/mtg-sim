//! Shared helpers for the tests of rulings batch P082 (`r_p082_*.rs`): abilities that
//! trigger on blocks ("hate-blocker" cards), block requirements, and a few spells.

#![allow(dead_code)]

use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 attacks P1 with each of `attackers`; P1 declares `blocks` (blocker, attacker). Stops
/// in the declare blockers step with P0 holding priority, block triggers on the stack.
pub fn blocked(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let at: Vec<(ObjectId, Entity)> = attackers
        .iter()
        .map(|a| (*a, Entity::Player(P1)))
        .collect();
    crate::r_s03_common::to_blockers(t, &at, blocks);
}

/// Like [`blocked`], with the real cards `attacker` (P0's) and `blockers` (P1's) put onto
/// the battlefield first (all asserted to compile completely).
pub fn block_with(t: &mut TestGame, attacker: &str, blockers: &[&str]) -> (ObjectId, Vec<ObjectId>) {
    supported(attacker);
    let a = t.battlefield(P0, attacker);
    let bs: Vec<ObjectId> = blockers.iter().map(|b| t.battlefield(P1, b)).collect();
    let blocks: Vec<(ObjectId, ObjectId)> = bs.iter().map(|b| (*b, a)).collect();
    blocked(t, &[a], &blocks);
    (a, bs)
}

/// Triggered abilities on the stack whose source is `source` (followed across zones).
pub fn triggers_from(t: &TestGame, source: ObjectId) -> usize {
    let s = t.g.current(source);
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, mtg_engine::object::StackKind::Triggered { source: src, .. }
                    if *src == s || *src == source)
            })
        })
        .count()
}

/// Whether `attacker` is blocked in the current combat (CR 509.1h).
pub fn is_blocked(t: &TestGame, attacker: ObjectId) -> bool {
    crate::r_p076_common::is_blocked(t, attacker)
}

/// Finishes combat: advances to the end of combat step of P0's turn.
pub fn finish_combat(t: &mut TestGame) {
    t.advance_to(P0, Step::EndOfCombat);
}
