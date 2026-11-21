//! Fatespinner (hand-written, `src/cards/fatespinner.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn opponent_chooses(choice: usize) -> TestGame {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fatespinner");
    t.set_step(P0, Step::End);
    t.answer(P1, DecisionKind::Option, Answer::Index(choice));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P1, Step::End);
    t
}

#[test]
fn skipping_the_combat_phase() {
    cr!("614.1b", "614.10");
    let t = opponent_chooses(2);
    let log = &t.g.turn.step_log;
    assert!(!log.iter().any(|s| s.is_combat()));
    assert!(log.contains(&Step::Draw));
    assert!(log.contains(&Step::PrecombatMain));
}

#[test]
fn skipping_the_main_phase_skips_both() {
    cr!("614.1b", "614.10");
    ruling!("Fatespinner", "your opponent skips them all");
    let t = opponent_chooses(1);
    let log = &t.g.turn.step_log;
    assert!(!log.iter().any(|s| s.is_main()));
    assert!(log.contains(&Step::BeginningOfCombat));
}

#[test]
fn skipping_the_draw_step() {
    cr!("614.1b");
    let t = opponent_chooses(0);
    assert!(!t.g.turn.step_log.contains(&Step::Draw));
    assert!(t.g.turn.step_log.contains(&Step::PrecombatMain));
}
