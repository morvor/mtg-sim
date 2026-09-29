//! Rulings batch S27 — copying activated abilities as they're activated (Illusionist's
//! Bracers, Rings of Brighthearth): the copy has the value of X chosen for the ability
//! (CR 707.10), and a mana ability, which doesn't use the stack, isn't copied (CR 605.1a,
//! 605.3b).

use crate::r_s01_common::*;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's Silklash Spider ("{X}{G}{G}: This creature deals X damage to each creature with
/// flying.") activates its ability with X = 2 (four lands), P1 has a Serra Angel (4/4
/// flier). Returns the Angel.
fn spider_x2(t: &mut TestGame, spider: ObjectId) -> ObjectId {
    let angel = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    activate_containing(t, P0, spider, "each creature with flying").unwrap();
    assert_eq!(tapped_lands(t, P0), 4);
    angel
}

#[test]
fn illusionist_s_bracers_copies_the_value_of_x() {
    cr!("707.10", "107.3", "603.2");
    ruling!(
        "Illusionist's Bracers",
        "If the ability has {X} in its cost, the value of X is copied."
    );
    supported("Illusionist's Bracers");
    supported("Silklash Spider");
    // "Whenever an ability of equipped creature is activated, if it isn't a mana ability,
    // copy that ability. You may choose new targets for the copy." The copy also deals 2
    // damage: 4 in all kill the Angel.
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Silklash Spider");
    attach_new(&mut t, P0, "Illusionist's Bracers", spider);
    let angel = spider_x2(&mut t, spider);
    t.resolve_all();
    assert!(!t.on_battlefield(angel));
    assert!(t.in_graveyard(P1, "Serra Angel"));
    // Without the Bracers, 2 damage.
    let mut t = TestGame::new(2);
    let spider = t.battlefield(P0, "Silklash Spider");
    let angel = spider_x2(&mut t, spider);
    t.resolve_all();
    assert!(t.on_battlefield(angel));
    assert_eq!(crate::r_s07_common::damage_on(&t, angel), 2);
}

#[test]
fn illusionist_s_bracers_doesn_t_copy_a_mana_ability() {
    cr!("605.1a", "605.3b", "603.2");
    ruling!(
        "Illusionist's Bracers",
        "A mana ability is an ability that (1) isn't a loyalty ability, (2) doesn't target, and (3) could add mana when it resolves."
    );
    supported("Illusionist's Bracers");
    // Llanowar Elves' "{T}: Add {G}." isn't copied: one {G}, nothing on the stack.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Illusionist's Bracers", elves);
    assert!(tap_for_mana(&mut t, P0, elves, "Add {G}"));
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn rings_of_brighthearth_s_copy_uses_the_same_value_of_x() {
    cr!("707.10", "107.3", "603.2");
    ruling!(
        "Rings of Brighthearth",
        "If the ability has {X} in its cost, the copy uses the same value of X."
    );
    supported("Rings of Brighthearth");
    // "Whenever you activate an ability, if it isn't a mana ability, you may pay {2}. If
    // you do, copy that ability. You may choose new targets for the copy."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rings of Brighthearth");
    let spider = t.battlefield(P0, "Silklash Spider");
    let angel = spider_x2(&mut t, spider);
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 6);
    assert!(!t.on_battlefield(angel));
}
