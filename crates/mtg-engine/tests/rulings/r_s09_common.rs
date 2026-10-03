//! Shared helpers for the tests of rulings batch S09 (`r_s09_*.rs`): gift, goad,
//! gravestorm, harmonize, haste, heroic, hidden agenda, hideaway, horsemanship, impending,
//! imprint, improvise, increment. (The helpers of batches S01–S07 are used too.)

#![allow(dead_code)]

use mtg_engine::combat::{attack_declaration_legal, attack_options};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

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

/// Moves to `p`'s beginning of combat step, starting a new combat phase (so that `p` is
/// the attacking player).
pub fn to_combat(t: &mut TestGame, p: PlayerId) {
    t.g.combat = None;
    t.set_step(p, Step::BeginningOfCombat);
}

/// Whether the declaration of attackers `decl` would be legal for the attacking player
/// now (CR 508.1c–d). Call it in the beginning of combat step.
pub fn legal_attack(t: &mut TestGame, decl: &[(ObjectId, Entity)]) -> bool {
    t.g.recompute();
    let options = attack_options(&t.g);
    attack_declaration_legal(&t.g, &options, decl)
}

/// From `p`'s beginning of combat step (moved to if needed), `p` declares `decl` as
/// attackers (an illegal declaration is replaced by a legal one, CR 508.1), and the game
/// stops in the declare attackers step. Returns the attacks.
pub fn declare(
    t: &mut TestGame,
    p: PlayerId,
    decl: &[(ObjectId, Entity)],
) -> Vec<(ObjectId, Entity)> {
    use mtg_engine::decision::Answer;
    if !(t.g.turn.active == p && t.g.turn.step == Step::BeginningOfCombat) {
        to_combat(t, p);
    }
    t.answer(p, DecisionKind::Attackers, Answer::Attackers(decl.to_vec()));
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers && g.turn.stage == Stage::Priority)
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "attackers not declared");
    attacks_now(t)
}

/// Runs the game until the next declare attackers step of `p`'s turn in which `p` has
/// priority (answering the declaration with `decl`), and returns the attacks.
pub fn declare_in_next_combat(
    t: &mut TestGame,
    p: PlayerId,
    decl: &[(ObjectId, Entity)],
) -> Vec<(ObjectId, Entity)> {
    use mtg_engine::decision::Answer;
    t.answer(p, DecisionKind::Attackers, Answer::Attackers(decl.to_vec()));
    let ok = t.g.run_until(10_000, |g| {
        g.turn.active == p
            && g.turn.step == Step::DeclareAttackers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(p)
    });
    assert!(ok, "no declare attackers step of {p}'s turn");
    attacks_now(t)
}
