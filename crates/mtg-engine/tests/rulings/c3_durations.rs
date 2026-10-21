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
