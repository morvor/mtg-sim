//! Rulings batch S21 — "Cast this spell only during combat before/after blockers are
//! declared" in a turn with several combat phases: the point is that of the current combat
//! phase (CR 506.8b–c).

use crate::r_s01_common::*;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Where the game stops in a turn with two combat phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Point {
    BeginningOfCombat(u32),
    DeclareAttackers(u32),
    DeclareBlockers(u32),
    EndOfCombat(u32),
    PostcombatMain,
}

/// P0 (with a vigilant Serra Angel attacking P1, who has no creatures) goes through two
/// combat phases; at each point, `check` is called (with the game stopped there).
fn two_combats(t: &mut TestGame, mut check: impl FnMut(&mut TestGame, Point)) {
    let angel = t.battlefield(P0, "Serra Angel");
    let attack = |t: &mut TestGame| {
        t.answer(
            P0,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(angel, Entity::Player(P1))]),
        );
    };
    to_beginning_of_combat(t, P0);
    let first = t.g.turn.combat_phases;
    for n in 1..=2 {
        assert_eq!(t.g.turn.combat_phases, first + n - 1);
        check(t, Point::BeginningOfCombat(n));
        attack(t);
        go_to(t, Step::DeclareAttackers);
        assert!(t.g.is_attacking(angel));
        check(t, Point::DeclareAttackers(n));
        go_to(t, Step::DeclareBlockers);
        check(t, Point::DeclareBlockers(n));
        if n == 1 {
            // An additional combat phase follows this one.
            t.g.add_extra_combat(false);
        }
        go_to(t, Step::EndOfCombat);
        check(t, Point::EndOfCombat(n));
        if n == 1 {
            go_to(t, Step::BeginningOfCombat);
        }
    }
    go_to(t, Step::PostcombatMain);
    check(t, Point::PostcombatMain);
    assert_eq!(t.life(P1), 12);
}

#[test]
fn after_blockers_are_declared_means_in_the_current_combat_phase() {
    cr!("506.8b", "506.8c");
    ruling!(
        "Chaotic Strike",
        "If a turn has multiple combat phases, this spell can be cast during any of them as long as it’s after the beginning of that phase’s Declare Blockers Step."
    );
    supported("Chaotic Strike");
    let mut t = TestGame::new(2);
    // "Cast this spell only during combat after blockers are declared."
    let strike = t.hand(P0, "Chaotic Strike");
    t.lands(P0, "Mountain", 2);
    let mut seen = vec![];
    two_combats(&mut t, |t, at| seen.push((at, castable(t, P0, strike))));
    use Point::*;
    assert_eq!(
        seen,
        vec![
            (BeginningOfCombat(1), false),
            (DeclareAttackers(1), false),
            (DeclareBlockers(1), true),
            (EndOfCombat(1), true),
            // The second combat phase: blockers were declared in the first one, but not
            // yet in this one.
            (BeginningOfCombat(2), false),
            (DeclareAttackers(2), false),
            (DeclareBlockers(2), true),
            (EndOfCombat(2), true),
            (PostcombatMain, false),
        ]
    );
}

fn before_blockers(name: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    // "Cast this spell only during combat before blockers are declared."
    let spell = t.hand(P0, name);
    t.lands(P0, "Mountain", 2);
    let mut seen = vec![];
    two_combats(&mut t, |t, at| seen.push((at, castable(t, P0, spell))));
    use Point::*;
    assert_eq!(
        seen,
        vec![
            (BeginningOfCombat(1), true),
            (DeclareAttackers(1), true),
            (DeclareBlockers(1), false),
            (EndOfCombat(1), false),
            // The second combat phase: before its declare blockers step, again.
            (BeginningOfCombat(2), true),
            (DeclareAttackers(2), true),
            (DeclareBlockers(2), false),
            (EndOfCombat(2), false),
            (PostcombatMain, false),
        ],
        "{name}"
    );
}

#[test]
fn before_blockers_are_declared_means_in_the_current_combat_phase_gorilla_war_cry() {
    cr!("506.8b", "506.8c");
    ruling!(
        "Gorilla War Cry",
        "If a turn has multiple combat phases, this spell can be cast during any of them as long as it's before the beginning of that phase's Declare Blockers Step."
    );
    before_blockers("Gorilla War Cry");
}

#[test]
fn before_blockers_are_declared_means_in_the_current_combat_phase_panic() {
    cr!("506.8b", "506.8c");
    ruling!(
        "Panic",
        "If a turn has multiple combat phases, this spell can be cast during any of them as long as it’s before the beginning of that phase’s Declare Blockers Step."
    );
    before_blockers("Panic");
    // Cast in the second combat's declare attackers step, it works.
    let mut t = TestGame::new(2);
    let panic = t.hand(P0, "Panic");
    t.lands(P0, "Mountain", 1);
    let wall = t.battlefield(P1, "Wall of Wood");
    let mut cast = false;
    two_combats(&mut t, |t, at| {
        if at == Point::DeclareAttackers(2) {
            // "Target creature can't block this turn."
            t.cast(P0, panic).target(wall).go();
            t.resolve_all();
            cast = true;
            assert!(t.in_graveyard(P0, "Panic"));
        }
    });
    assert!(cast);
}

#[test]
fn after_blockers_are_declared_never_comes_in_a_combat_without_attackers() {
    cr!("506.8f", "508.8");
    supported("Chaotic Strike");
    // No creature attacks: the declare blockers step is skipped, and so Chaotic Strike
    // ("Cast this spell only during combat after blockers are declared") can't be cast
    // during that combat phase.
    let mut t = TestGame::new(2);
    let strike = t.hand(P0, "Chaotic Strike");
    t.lands(P0, "Mountain", 2);
    t.battlefield(P0, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    go_to(&mut t, Step::EndOfCombat);
    assert!(t.g.combat.as_ref().is_none_or(|c| !c.blockers_declared));
    assert!(!castable(&mut t, P0, strike));
}
