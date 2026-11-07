//! Shared helpers for the tests of CR 702.18–702.26 (shroud, trample, vigilance, ward,
//! banding, rampage, cumulative upkeep, flanking, phasing).

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// Declares blocks for the defending player and stops at the first priority of the
/// declare blockers step (block triggers are on the stack, not yet resolved).
pub fn declare_blocks(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    let ap = t.g.turn.active;
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareBlockers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "blockers not declared");
    // Triggered abilities are put on the stack before the player receives priority.
    t.settle();
}

/// Number of triggered abilities on the stack whose text is `text` (a keyword's name).
pub fn triggers_on_stack(t: &TestGame, text: &str) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, object::StackKind::Triggered { ability, .. }
                    if ability.text.as_str() == text)
            })
        })
        .count()
}

/// Answers the next combat damage assignment of `p` with `amounts` (in the order of the
/// decision's recipients).
pub fn assign_damage(t: &mut TestGame, p: PlayerId, amounts: &[i64]) {
    t.answer(p, DecisionKind::Damage, Answer::Numbers(amounts.to_vec()));
}

/// The combat damage assignment decisions asked so far: (player, creature, recipients).
pub fn damage_decisions(t: &TestGame) -> Vec<(PlayerId, ObjectId, Vec<Entity>)> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::AssignCombatDamage {
                creature,
                recipients,
                ..
            } => Some((p, creature, recipients)),
            _ => None,
        })
        .collect()
}
