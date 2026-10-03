//! Damage divided among any number of targets with an amount read from an object
//! (CR 601.2d, 603.3d), and damage dealt by each object of a group named before (CR 120.2).

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn living_inferno_divides_its_power_and_takes_damage_from_each_of_those_creatures() {
    cr!("601.2d", "120.2");
    compiles("Living Inferno");
    let mut t = TestGame::new(2);
    let inferno = t.battlefield(P0, "Living Inferno");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(elves)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![7, 1]));
    t.activate(P0, inferno, 0, &[]).expect("activates");
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(elves));
    // 2 from the Bears and 1 from the Elves.
    assert_eq!(t.obj_now(inferno).damage, 3);
}

#[test]
fn orca_siege_demon_divides_damage_equal_to_its_last_known_power() {
    cr!("601.2d", "603.10a");
    compiles("Orca, Siege Demon");
    let mut t = TestGame::new(2);
    let orca = t.battlefield(P0, "Orca, Siege Demon");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 3]));
    t.g.destroy(orca, None);
    t.settle();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn shatterskull_smashing_divides_twice_x_when_x_is_6_or_more() {
    cr!("601.2d", "601.2b");
    let name = "Shatterskull Smashing // Shatterskull, the Hammer Pass";
    compiles(name);
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 8);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Colossal Dreadmaw");
    let s = t.hand(P0, name);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![6, 6]));
    t.cast(P0, s)
        .x(6)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn shatterskull_smashing_divides_x_when_x_is_less_than_6() {
    cr!("601.2d");
    let name = "Shatterskull Smashing // Shatterskull, the Hammer Pass";
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    let a = t.battlefield(P1, "Colossal Dreadmaw");
    let b = t.battlefield(P1, "Grizzly Bears");
    let s = t.hand(P0, name);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 2]));
    t.cast(P0, s)
        .x(5)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert!(t.on_battlefield(a));
    assert_eq!(t.obj_now(a).damage, 3);
    assert!(!t.on_battlefield(b));
}
