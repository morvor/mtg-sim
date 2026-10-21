//! Durations (cards3-triggers-durations): "for as long as" before the instructions
//! (CR 611.2b), next-turn durations, player-level durations.

use crate::r_s01_common::supported;
use crate::r_s02_common::{can_attack, destroy};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn immovable_rod_for_as_long_as_it_remains_tapped() {
    cr!("611.2b");
    supported("Immovable Rod");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let rod = t.battlefield(P0, "Immovable Rod");
    t.lands(P0, "Plains", 4);
    let angel = t.battlefield(P1, "Serra Angel");
    t.g.turn.priority = Some(P0);
    t.activate(P0, rod, 0, &[Entity::Object(angel)]).expect("activate");
    t.resolve_all();
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Flying));
    t.set_step(P1, Step::DeclareAttackers);
    assert!(!can_attack(&mut t, angel));
    // The Rod untaps: the effect ends, and stays ended if it's tapped again.
    t.g.untap(rod);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    t.g.obj_mut(rod).tapped = true;
    t.g.recompute();
    assert!(t.obj_now(angel).has_keyword(KeywordKind::Flying));
}

#[test]
fn opportunistic_dragon_steals_for_as_long_as_it_remains() {
    cr!("611.2b", "613.1b");
    ruling!(
        "Opportunistic Dragon",
        "Opportunistic Dragon's effect doesn't expire if another player gains control of it or if it loses all abilities."
    );
    supported("Opportunistic Dragon");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    let dragon = t.enter(P0, "Opportunistic Dragon");
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(thopter).controller, P0);
    assert!(!t.obj_now(thopter).has_keyword(KeywordKind::Flying));
    t.set_step(P0, Step::DeclareAttackers);
    assert!(!can_attack(&mut t, thopter));
    // The Dragon leaves: the opponent gets it back, with its abilities.
    destroy(&mut t, dragon);
    t.resolve_all();
    assert_eq!(t.obj_now(thopter).controller, P1);
    assert!(t.obj_now(thopter).has_keyword(KeywordKind::Flying));
}

#[test]
fn second_doctor_opponents_who_draw_cant_attack_you_next_turn() {
    cr!("611.2a", "611.2c", "508.1c");
    interp!("next-turn-effects-start-with-the-turn");
    supported("The Second Doctor");
    for draws in [true, false] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "The Second Doctor");
        for p in [P0, P1, P2] {
            t.library_top(p, "Island");
            t.library_top(p, "Island");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P0, false);
        t.answer_yes(P1, draws);
        t.answer_yes(P2, false);
        t.set_step(P0, Step::PostcombatMain);
        t.advance_to(P1, Step::DeclareAttackers);
        let targets: Vec<Entity> = mtg_engine::combat::attack_options(&t.g)
            .into_iter()
            .filter(|(c, _)| *c == bears)
            .flat_map(|(_, e)| e)
            .collect();
        // It can still attack P2; P0 only if P1 didn't draw.
        assert!(targets.contains(&Entity::Player(P2)));
        assert_eq!(targets.contains(&Entity::Player(P0)), !draws, "drew: {draws}");
    }
}

#[test]
fn willie_lumpkin_creatures_entering_later_cant_attack_either() {
    cr!("611.2c", "508.1c");
    interp!("next-turn-effects-start-with-the-turn");
    supported("Willie Lumpkin, Postman");
    let mut t = TestGame::new(2);
    let willie = t.battlefield(P0, "Willie Lumpkin, Postman");
    for p in [P0, P1] {
        t.library_top(p, "Island");
        t.library_top(p, "Island");
    }
    t.set_step(P0, Step::PrecombatMain);
    t.answer_yes(P1, true);
    t.attack(&[(willie, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 1);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::PrecombatMain);
    // A haste creature that entered during that turn can't attack P0 either.
    let hasty = t.battlefield(P1, "Raging Goblin");
    t.set_step(P1, Step::DeclareAttackers);
    assert!(!can_attack(&mut t, hasty));
    // The turn after, it can.
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::DeclareAttackers);
    assert!(can_attack(&mut t, hasty));
}
