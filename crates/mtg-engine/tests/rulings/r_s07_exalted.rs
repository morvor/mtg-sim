//! Rulings batch S07 — exalted (CR 702.83): "Whenever a creature you control attacks
//! alone, that creature gets +1/+1 until end of turn."

use crate::r_s01_common::*;
use crate::r_s05_common::*;
use mtg_engine::ability::*;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_creature_attacks_alone_only_if_it_was_the_only_one_declared() {
    cr!("702.83a", "702.83b", "506.5");
    ruling!(
        "Ignoble Hierarch",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step (including creatures controlled by your teammates, if applicable). For example, exalted won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    supported("Ignoble Hierarch");
    // Ignoble Hierarch: 0/1, exalted.
    // Two attackers, then one is removed from combat: exalted never triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ignoble Hierarch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (elves, Entity::Player(P1))],
    );
    assert_eq!(triggers_on_stack(&t, "Exalted"), 0);
    run_from(
        &mut t,
        P0,
        None,
        Effect::RemoveFromCombat {
            what: Sel::Target(0),
        },
        &[Entity::Object(elves)],
    );
    assert!(!t.g.is_attacking(elves));
    assert_eq!(triggers_on_stack(&t, "Exalted"), 0);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
    // One creature declared as the attacker: exalted triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ignoble Hierarch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "Exalted"), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    // Two-Headed Giant: a creature attacking along with a teammate's creature doesn't
    // attack alone; one attacking alone for the whole team does.
    let config = || GameConfig {
        variant: Variant::TwoHeadedGiant,
        teams: Some(vec![0, 0, 1, 1]),
        ..Default::default()
    };
    let mut t = TestGame::with_config(4, config());
    t.battlefield(P0, "Ignoble Hierarch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P2)), (theirs, Entity::Player(P2))],
    );
    assert!(t.g.is_attacking(theirs));
    assert_eq!(triggers_on_stack(&t, "Exalted"), 0);
    let mut t = TestGame::with_config(4, config());
    t.battlefield(P0, "Ignoble Hierarch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    assert_eq!(triggers_on_stack(&t, "Exalted"), 1);
}
