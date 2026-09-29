//! Rulings batch S27 — copying activated abilities as they're activated (Illusionist's
//! Bracers, Rings of Brighthearth): the copy has the value of X chosen for the ability
//! (CR 707.10), and a mana ability, which doesn't use the stack, isn't copied (CR 605.1a,
//! 605.3b).

use crate::r_s01_common::*;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s25_common::change_copy_targets;
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

#[test]
fn illusionist_s_bracers_on_an_opponent_s_creature_gives_you_the_copy() {
    cr!("707.10", "603.3a", "603.2");
    ruling!(
        "Illusionist's Bracers",
        "If Illusionist's Bracers somehow becomes equipped to a creature an opponent controls, and an activated ability (that isn't a mana ability) of that creature is activated, you will control the copy of that ability."
    );
    supported("Illusionist's Bracers");
    // P0's Bracers equip P1's Prodigal Pyromancer ("{T}: This creature deals 1 damage to
    // any target."). P1 pings P0; P0 controls the copy and points it at P1.
    let mut t = TestGame::new(2);
    let pyromancer = t.battlefield(P1, "Prodigal Pyromancer");
    attach_new(&mut t, P0, "Illusionist's Bracers", pyromancer);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    activate_containing(&mut t, P1, pyromancer, "damage").unwrap();
    t.settle();
    // The Bracers' trigger resolves: the copy is P0's.
    t.resolve();
    let copy = *t.g.stack.last().expect("the copy");
    assert_eq!(t.obj(copy).controller, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 19);
}

#[test]
fn illusionist_s_bracers_doesn_t_copy_an_ability_whose_cost_sacrificed_either() {
    cr!("602.2b", "601.2i", "603.2", "707.10");
    ruling!(
        "Illusionist's Bracers",
        "If the cost of an activated ability requires Illusionist's Bracers or the equipped creature to be sacrificed, the ability won't be copied."
    );
    supported("Illusionist's Bracers");
    supported("Mogg Fanatic");
    supported("Atog");
    // Mogg Fanatic ("Sacrifice this creature: It deals 1 damage to any target.") is
    // sacrificed to pay the cost: when the ability becomes activated, the Bracers equip
    // nothing. 1 damage, not 2.
    let mut t = TestGame::new(2);
    let fanatic = t.battlefield(P0, "Mogg Fanatic");
    attach_new(&mut t, P0, "Illusionist's Bracers", fanatic);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, fanatic, "Sacrifice").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Atog ("Sacrifice an artifact: This creature gets +2/+2 until end of turn.") equipped
    // with the Bracers sacrifices them: +2/+2 once.
    let mut t = TestGame::new(2);
    let atog = t.battlefield(P0, "Atog");
    let bracers = attach_new(&mut t, P0, "Illusionist's Bracers", atog);
    t.answer_choose(P0, &[Entity::Object(bracers)]);
    activate_containing(&mut t, P0, atog, "Sacrifice").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(atog), (3, 4));
    // Sacrificing another artifact, the ability is copied: +4/+4.
    let mut t = TestGame::new(2);
    let atog = t.battlefield(P0, "Atog");
    attach_new(&mut t, P0, "Illusionist's Bracers", atog);
    let ornithopter = t.battlefield(P0, "Ornithopter");
    t.answer_choose(P0, &[Entity::Object(ornithopter)]);
    activate_containing(&mut t, P0, atog, "Sacrifice").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(atog), (5, 6));
}

#[test]
fn rings_of_brighthearth_doesn_t_copy_an_ability_whose_cost_sacrificed_it() {
    cr!("602.2b", "601.2i", "603.2");
    ruling!(
        "Rings of Brighthearth",
        "If paying the activation cost of the ability includes sacrificing Rings of Brighthearth, the ability won't be copied."
    );
    supported("Rings of Brighthearth");
    supported("Atog");
    let mut t = TestGame::new(2);
    let rings = t.battlefield(P0, "Rings of Brighthearth");
    let atog = t.battlefield(P0, "Atog");
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[Entity::Object(rings)]);
    t.answer_yes(P0, true);
    activate_containing(&mut t, P0, atog, "Sacrifice").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(atog), (3, 4));
    assert_eq!(tapped_lands(&t, P0), 0);
}

#[test]
fn rings_of_brighthearth_copies_an_ability_countered_in_response() {
    cr!("707.10", "608.2h", "701.6a");
    ruling!(
        "Rings of Brighthearth",
        "They resolve even if that ability is countered."
    );
    supported("Rings of Brighthearth");
    supported("Prodigal Pyromancer");
    // P0's Prodigal Pyromancer pings P1; with Rings' trigger on the stack, P1 counters
    // the original with Disallow. The trigger still resolves and copies it (as it last
    // existed on the stack): P1 takes 1 damage from the copy.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rings of Brighthearth");
    let pyromancer = t.battlefield(P0, "Prodigal Pyromancer");
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ping = activate_containing(&mut t, P0, pyromancer, "damage")
        .unwrap()
        .expect("on the stack");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.lands(P1, "Island", 3);
    let disallow = t.hand(P1, "Disallow");
    t.cast(P1, disallow).target(ping).go();
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(tapped_lands(&t, P0), 2);
    assert_eq!(t.life(P1), 19);
}
