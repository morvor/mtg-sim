//! CR 719: Case cards.

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, Layout};
use mtg_engine::cases;
use mtg_engine::events::MoveCause;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Case of the Crimson Pulse ({2}{R}): "When this Case enters, discard a card, then draw
/// two cards. To solve — You have no cards in hand. Solved — At the beginning of your
/// upkeep, discard your hand, then draw two cards."
const PULSE: &str = "Case of the Crimson Pulse";

fn empty_hand(t: &mut TestGame, p: PlayerId) {
    for c in t.g.player(p).hand.clone() {
        t.g.move_object(c, Zone::Graveyard(p), MoveCause::Effect, None);
    }
}

/// Advances from P0's second main phase through their end step (resolving "to solve").
fn through_end_step(t: &mut TestGame) {
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
}

#[test]
fn a_case_has_to_solve_and_solved_abilities() {
    cr!("719.3");
    ruling!(
        "Case of the Crimson Pulse",
        "Each Case has two special keyword abilities: to solve and solved"
    );
    supported(PULSE);
    let def = card(PULSE);
    assert_eq!(def.layout, Layout::Case);
    let abilities = &def.front().chars.abilities;
    // "To solve": a triggered ability at the beginning of your end step.
    assert!(abilities.iter().any(|a| matches!(&a.kind,
        AbilityKind::Triggered(t) if matches!(t.trigger, TriggerCond::BeginningOf { step: TriggerStep::End, .. })
            && matches!(t.body.effect, Effect::Custom(ref n) if n == cases::SOLVE))));
    // "Solved": an ability that functions only while it's solved.
    assert!(abilities.iter().any(|a| matches!(&a.kind,
        AbilityKind::Static(s) if matches!(s.condition, Some(Condition::Custom(ref n)) if n == cases::SOLVED))));
}

#[test]
fn to_solve_at_the_beginning_of_your_end_step() {
    cr!("719.3a");
    ruling!(
        "Case of the Crimson Pulse",
        "If the condition isn't true when the ability resolves, the Case won't become solved"
    );
    // With cards in hand at the end step: not solved.
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, PULSE);
    t.hand(P0, "Island");
    through_end_step(&mut t);
    assert!(!cases::is_solved(&t.g, case));
    // With no cards in hand: solved.
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, PULSE);
    empty_hand(&mut t, P0);
    through_end_step(&mut t);
    assert!(cases::is_solved(&t.g, case));
    // Only at the beginning of its controller's end step.
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, PULSE);
    empty_hand(&mut t, P0);
    t.set_step(P1, Step::PostcombatMain);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!cases::is_solved(&t.g, case));
    // The condition is checked again as it resolves.
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, PULSE);
    empty_hand(&mut t, P0);
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.hand(P0, "Island");
    t.resolve_all();
    assert!(!cases::is_solved(&t.g, case));
}

#[test]
fn solved_is_a_designation_that_stays_until_it_leaves_and_isnt_copiable() {
    cr!("719.3b");
    ruling!(
        "Case of the Crimson Pulse",
        "A permanent that becomes a copy of a solved Case is not solved"
    );
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, PULSE);
    assert!(cases::solve(&mut t.g, case));
    assert!(!cases::solve(&mut t.g, case), "already solved");
    // It stays solved though the condition stops being true.
    t.hand(P0, "Island");
    through_end_step(&mut t);
    assert!(cases::is_solved(&t.g, case));
    // A copy isn't solved.
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(case)]);
    let copy = t.enter(P0, "Copy Enchantment");
    t.resolve_all();
    assert_eq!(t.obj(copy).chars.name, PULSE);
    assert!(!cases::is_solved(&t.g, copy));
    // Leaving the battlefield, it loses the designation.
    let back =
        t.g.move_object(case, Zone::Hand(P0), MoveCause::Effect, None)
            .unwrap();
    let again =
        t.g.move_object(back, Zone::Battlefield, MoveCause::Effect, None)
            .unwrap();
    assert!(!cases::is_solved(&t.g, again));
}

#[test]
fn a_solved_ability_functions_only_while_the_case_is_solved() {
    cr!("719.3c");
    ruling!(
        "Case of the Crimson Pulse",
        "This ability triggers only if this Case is solved"
    );
    // Unsolved: its upkeep ability doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, PULSE);
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // Solved: "At the beginning of your upkeep, discard your hand, then draw two cards."
    let mut t = TestGame::new(2);
    let case = t.battlefield(P0, PULSE);
    cases::solve(&mut t.g, case);
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    assert_eq!(t.graveyard_size(P0), 3);
}
