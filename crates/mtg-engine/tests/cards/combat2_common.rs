//! Shared helpers for the combat grammar III tests (`combat2_*.rs`).

#![allow(dead_code)]

use mtg_engine::combat::{attack_declaration_legal, attack_options};
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// Asserts that the card's oracle text compiled completely.
pub fn supported(name: &str) {
    let c = mtg_engine::card(name);
    for f in &c.faces {
        assert!(f.unsupported.is_empty(), "{name}: {:?}", f.unsupported);
    }
}

/// The attackers and what they attack in the current combat.
pub fn attacks_now(t: &TestGame) -> Vec<(ObjectId, Entity)> {
    t.g.combat
        .as_ref()
        .map(|c| {
            c.attackers
                .iter()
                .filter_map(|a| a.target.map(|x| (a.id, x)))
                .collect()
        })
        .unwrap_or_default()
}

/// Moves to `p`'s beginning of combat step, starting a new combat phase.
pub fn to_combat(t: &mut TestGame, p: PlayerId) {
    t.g.combat = None;
    t.set_step(p, Step::BeginningOfCombat);
}

/// Whether the declaration of attackers `decl` would be legal for the attacking player
/// now (CR 508.1c-d). Call it in the beginning of combat step.
pub fn legal_attack(t: &mut TestGame, decl: &[(ObjectId, Entity)]) -> bool {
    t.g.recompute();
    let options = attack_options(&t.g);
    attack_declaration_legal(&t.g, &options, decl)
}

/// Whether the declaration of blockers `decl` would be legal now (CR 509.1b-c). Call it
/// in the declare attackers step.
pub fn legal_blocks(t: &mut TestGame, decl: &[(ObjectId, ObjectId)]) -> bool {
    t.g.recompute();
    let defenders =
        t.g.combat
            .as_ref()
            .map(|c| c.defending_players.clone())
            .unwrap_or_default();
    let options = mtg_engine::combat::block_options(&t.g, &defenders);
    mtg_engine::combat::block_declaration_legal(&t.g, &options, decl)
}

/// From `p`'s beginning of combat step, `p` declares `decl` as attackers; the game stops
/// in the declare attackers step with the active player holding priority. Returns the
/// attacks. Assert `legal_attack` first: the harness replaces an illegal declaration.
pub fn declare(
    t: &mut TestGame,
    p: PlayerId,
    decl: &[(ObjectId, Entity)],
) -> Vec<(ObjectId, Entity)> {
    if !(t.g.turn.active == p && t.g.turn.step == Step::BeginningOfCombat) {
        to_combat(t, p);
    }
    t.answer(p, DecisionKind::Attackers, Answer::Attackers(decl.to_vec()));
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(g.turn.active))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "attackers not declared");
    attacks_now(t)
}

/// From the declare attackers step, the defending player `dp` declares `blocks`; the game
/// stops in the declare blockers step with the active player holding priority.
pub fn declare_blocks(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareBlockers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(g.turn.active))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "blockers not declared");
}

/// Runs the game to the end of combat step of the current turn.
pub fn to_end_of_combat(t: &mut TestGame) {
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::EndOfCombat);
}

/// The current object of `id` (following zone changes is not done: same id).
pub fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.g.obj(id).controller
}
