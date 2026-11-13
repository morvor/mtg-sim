//! Triggers at each of several phases: "At the beginning of each of your main phases"
//! (Frontier Siege, Carpet of Flowers), "At the beginning of each of enchanted player's
//! postcombat main phases" (Shadow of the Second Sun).

use crate::combat2_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> usize {
    t.g.players[p.idx()].mana_pool.mana.len()
}

#[test]
fn frontier_siege_khans_adds_mana_at_each_of_your_main_phases() {
    cr!("505.1", "603.2");
    ruling!(
        "Frontier Siege",
        "The \"Khans\" ability triggers at the beginning of both your precombat main phase and your postcombat main phase."
    );
    supported("Frontier Siege");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.enter(P0, "Frontier Siege");
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2);
    // Not during the opponent's main phases.
    t.advance_to(P1, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 0);
}

#[test]
fn carpet_of_flowers_adds_mana_once_each_turn_at_most() {
    cr!("603.2h", "603.4", "505.1");
    supported("Carpet of Flowers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Carpet of Flowers");
    t.lands(P1, "Island", 2);
    t.advance_to(P1, Step::Upkeep);
    // First main phase: decline.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 0);
    // Second main phase: it triggers again; this time add the mana.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2);
    // Next turn: mana added in the first main phase, so it doesn't trigger in the second.
    t.advance_to(P1, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0), 2);
    t.advance_to(P0, Step::PostcombatMain);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn shadow_of_the_second_sun_gives_an_additional_beginning_phase() {
    cr!("500.8", "505.1");
    ruling!(
        "Shadow of the Second Sun",
        "After the additional beginning phase, the game proceeds to the ending phase"
    );
    supported("Shadow of the Second Sun");
    let mut t = TestGame::new(2);
    let shadow = t.battlefield(P0, "Shadow of the Second Sun");
    assert!(t.g.attach(shadow, Entity::Player(P1)));
    t.advance_to(P1, Step::PostcombatMain);
    let hand = t.hand_size(P1);
    let turn = t.g.turn.number;
    t.resolve_all();
    t.advance_to(P1, Step::End);
    assert_eq!(t.g.turn.number, turn);
    // An untap, upkeep and draw step after the postcombat main phase: one more draw.
    assert_eq!(t.hand_size(P1), hand + 1);
    let log = &t.g.turn.step_log;
    let main = log.iter().position(|s| *s == Step::PostcombatMain).unwrap();
    assert!(log[main..].contains(&Step::Upkeep));
    // P0's turn: not enchanted.
    t.advance_to(P0, Step::End);
    assert_eq!(
        t.g.turn.step_log.iter().filter(|s| **s == Step::Upkeep).count(),
        1
    );
}
