//! Rulings batch S06 — Laccolith Rig: "Whenever enchanted creature becomes blocked, you may
//! have it deal damage equal to its power to target creature. If you do, the first
//! creature assigns no combat damage this turn." (and the Laccoliths' "[it] assigns no
//! combat damage this turn", CR 510.1a).

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_creature_that_assigns_no_combat_damage_deals_none() {
    cr!("510.1a", "611.2a");
    supported("Laccolith Titan");
    supported("Ophidian");
    // Ophidian (1/3): "Whenever this creature attacks and isn't blocked, you may draw a
    // card. If you do, this creature assigns no combat damage this turn."
    let mut t = TestGame::new(2);
    let snake = t.battlefield(P0, "Ophidian");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    to_blockers(&mut t, &[(snake, Entity::Player(P1))], &[]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P1), 20);
    // Declining: it deals its damage.
    let mut t = TestGame::new(2);
    let snake = t.battlefield(P0, "Ophidian");
    t.answer_yes(P0, false);
    to_blockers(&mut t, &[(snake, Entity::Player(P1))], &[]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn moving_laccolith_rig_after_it_triggers_doesnt_change_the_creature_affected() {
    cr!("603.10", "608.2h", "510.1a");
    ruling!(
        "Laccolith Rig",
        "Moving the enchantment after the ability triggers will not affect which creatures are affected."
    );
    supported("Laccolith Rig");
    // Hill Giant (3/3) enchanted with Laccolith Rig attacks and is blocked by Grizzly
    // Bears; P0 targets P1's Savannah Lions with the trigger.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let rig = attach_new(&mut t, P0, "Laccolith Rig", giant);
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.answer_targets(P0, &[Entity::Object(lions)]);
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(blocker, giant)]);
    assert_eq!(on_stack(&t, "deal damage equal to its power"), 1);
    // The Aura moves to Llanowar Elves (1/1) before the ability resolves.
    assert!(t.g.attach(rig, Entity::Object(elves)));
    t.g.recompute();
    t.answer_yes(P0, true);
    t.resolve_all();
    // Hill Giant dealt the damage (3, killing the 2/1 Lions)...
    assert!(t.in_graveyard(P1, "Savannah Lions"));
    t.advance_to(P0, Step::EndOfCombat);
    // ... and it assigns no combat damage: the blocking Bears survive.
    assert!(t.on_battlefield(blocker));
    assert_eq!(t.obj_now(blocker).damage, 0);
    assert_eq!(t.obj_now(giant).damage, 2);
    // The Elves, now enchanted, weren't affected.
    assert_eq!(t.obj_now(elves).damage, 0);
    // Without the Rig's ability, the Giant would have killed the Bears.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Laccolith Rig", giant);
    let blocker = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P1, "Savannah Lions");
    t.answer_targets(P0, &[Entity::Object(lions)]);
    t.answer_yes(P0, false);
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(blocker, giant)]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(lions));
}
