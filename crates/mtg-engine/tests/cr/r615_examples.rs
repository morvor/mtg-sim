//! The worked examples of CR 615 (prevention effects), played out as tests.

use crate::r600_examples_common::*;
use crate::r609_common::modify_target;
use mtg_engine::ability::Modification;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn preventing_damage_in_response_works_but_not_after_the_spell_resolves() {
    cr!("615.1a");
    cr_example!("615.4", "activate an ability that prevents damage in response");
    supported("Samite Healer");
    let mut t = TestGame::new(2);
    let healer = t.battlefield(P0, "Samite Healer");
    let healer2 = t.battlefield(P0, "Samite Healer");
    t.lands(P1, "Mountain", 2);
    t.set_step(P1, Step::PrecombatMain);
    // In response to Lightning Bolt: 1 of its 3 damage is prevented.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.activate(P0, healer, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve(); // the shield
    t.resolve(); // Lightning Bolt
    assert_eq!(t.life(P0), 18);
    // Once the spell has resolved, it's too late to prevent its damage.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 15);
    t.activate(P0, healer2, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 15);
}

#[test]
fn daunting_defender_prevents_one_of_pyroclasms_damage_to_each_of_your_clerics() {
    cr!("615.1a");
    cr_example!("615.10", "Daunting Defender says");
    supported("Daunting Defender");
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    // Daunting Defender and Cleric of the Forward Order (a 2/2) are Clerics; Hill Giant
    // isn't.
    let defender = t.battlefield(P0, "Daunting Defender");
    let cleric = t.battlefield(P0, "Cleric of the Forward Order");
    let giant = t.battlefield(P0, "Hill Giant");
    // An opponent's Cleric isn't "a Cleric creature you control".
    let their_cleric = t.battlefield(P1, "Daunting Defender");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    let pyro = t.hand(P1, "Pyroclasm");
    t.cast(P1, pyro).go();
    t.resolve();
    assert_eq!(t.obj_now(defender).damage, 1);
    assert_eq!(t.obj_now(cleric).damage, 1);
    assert!(t.on_battlefield(cleric));
    assert_eq!(t.obj_now(giant).damage, 2);
    // P1's own Defender protects P1's Clerics, itself included.
    assert_eq!(t.obj_now(their_cleric).damage, 1);
}

#[test]
fn wojek_apothecarys_shields_are_set_when_the_ability_resolves() {
    cr!("615.1a", "611.2c");
    cr_example!("615.11", "Wojek Apothecary has an ability");
    supported("Wojek Apothecary");
    let mut t = TestGame::new(2);
    let apothecary = t.battlefield(P0, "Wojek Apothecary");
    // Serra Angel (white) is targeted; Savannah Lions (white) shares a color with it;
    // Hill Giant (red) doesn't.
    let angel = t.battlefield(P0, "Serra Angel");
    let lions = t.battlefield(P0, "Savannah Lions");
    let giant = t.battlefield(P1, "Hill Giant");
    t.activate(P0, apothecary, 0, &[Entity::Object(angel)])
        .unwrap();
    t.resolve();
    // After resolution: the Lions become red and the Giant white; a white creature
    // enters.
    modify_target(
        &mut t,
        P0,
        lions,
        vec![Modification::SetColors(ColorSet::single(Color::Red))],
    );
    modify_target(
        &mut t,
        P0,
        giant,
        vec![Modification::SetColors(ColorSet::single(Color::White))],
    );
    let late = t.battlefield(P0, "Serra Angel");
    let src = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage_batch(
        vec![
            (src, Entity::Object(angel), 2),
            (src, Entity::Object(lions), 1),
            (src, Entity::Object(giant), 2),
            (src, Entity::Object(late), 2),
        ],
        false,
    );
    t.settle();
    // The target and the creature that shared its color at resolution have shields.
    assert_eq!(t.obj_now(angel).damage, 1);
    assert!(t.on_battlefield(lions));
    assert_eq!(t.obj_now(lions).damage, 0);
    // Changing colors afterward didn't add a shield; the latecomer has none.
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.obj_now(late).damage, 2);
}
