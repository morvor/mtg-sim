//! Destroying creatures in combat: "both creatures", "that Wall at end of combat", "it and
//! all creatures it blocked this turn", "target creature ~ is blocking", "it and ~ at end
//! of combat" (CR 506, 509, 603.7).

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 attacks with `attacker`; P1 blocks with `blocker`; stops in the declare blockers step.
fn block(t: &mut TestGame, attacker: ObjectId, blocker: ObjectId) {
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P1))]),
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(blocker, attacker)]));
    t.advance_to(P0, Step::DeclareBlockers);
}

#[test]
fn alaborn_zealot_destroys_both_creatures() {
    cr!("509.1", "603.2");
    assert_supported("Alaborn Zealot");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Craw Wurm");
    let zealot = t.battlefield(P1, "Alaborn Zealot");
    block(&mut t, attacker, zealot);
    t.resolve_all();
    assert!(!t.on_battlefield(attacker));
    assert!(!t.on_battlefield(zealot));
}

#[test]
fn defiant_vanguard_destroys_itself_and_what_it_blocked_at_end_of_combat() {
    cr!("603.7");
    assert_supported("Defiant Vanguard");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Craw Wurm");
    let other = t.battlefield(P0, "Craw Wurm");
    let dv = t.battlefield(P1, "Defiant Vanguard");
    block(&mut t, attacker, dv);
    t.resolve_all();
    assert!(t.on_battlefield(attacker));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!t.on_battlefield(attacker));
    assert!(t.on_battlefield(other));
}

#[test]
fn battering_ram_destroys_the_blocking_wall_at_end_of_combat() {
    cr!("603.7");
    assert_supported("Battering Ram");
    let mut t = TestGame::new(2);
    let ram = t.battlefield(P0, "Battering Ram");
    let wall = t.battlefield(P1, "Wall of Stone");
    block(&mut t, ram, wall);
    t.resolve_all();
    assert!(t.on_battlefield(wall));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!t.on_battlefield(wall));
}

#[test]
fn wall_of_corpses_destroys_the_creature_it_blocks() {
    cr!("509.1", "608.2b");
    assert_supported("Wall of Corpses");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Craw Wurm");
    let wall = t.battlefield(P1, "Wall of Corpses");
    block(&mut t, attacker, wall);
    t.lands(P1, "Swamp", 1);
    t.activate(P1, wall, 0, &[Entity::Object(attacker)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(attacker));
}

#[test]
fn goblin_sappers_destroys_both_at_end_of_combat() {
    cr!("603.7");
    assert_supported("Goblin Sappers");
    let mut t = TestGame::new(2);
    let sappers = t.battlefield(P0, "Goblin Sappers");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, sappers, 0, &[Entity::Object(wurm)]).unwrap();
    t.resolve();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(wurm, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 14);
    assert!(!t.on_battlefield(wurm));
    assert!(!t.on_battlefield(sappers));
}
