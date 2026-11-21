//! Qualities chosen as an effect happens ("of your choice", CR 608.2d), locked into the
//! effect (CR 608.2h), and targets chosen by another player ("target ... of their
//! choice", CR 601.2c).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn creature_type(name: &str) -> usize {
    subtype_lists()
        .creature
        .iter()
        .position(|t| t == name)
        .expect("a creature type")
}

fn color_index(c: Color) -> usize {
    Color::ALL.iter().position(|x| *x == c).unwrap()
}

#[test]
fn tribal_unity_creatures_of_the_creature_type_of_your_choice() {
    cr!("608.2d", "205.3m");
    compiles("Tribal Unity");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer(P0, DecisionKind::Option, Answer::Index(creature_type("Bear")));
    let spell = t.hand(P0, "Tribal Unity");
    t.cast(P0, spell).x(2).go();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(their_bears), (4, 4));
    assert_eq!(t.pt(elves), (1, 1));
}

#[test]
fn blind_seer_target_spell_becomes_the_color_of_your_choice() {
    cr!("105.3", "608.2h");
    compiles("Blind Seer");
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Blind Seer");
    t.lands(P0, "Island", 3);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.cast(P1, bolt).target(P0).go();
    t.answer(P0, DecisionKind::Option, Answer::Index(color_index(Color::Blue)));
    t.activate(P0, seer, 0, &[Entity::Object(spell)])
        .expect("activates");
    t.resolve();
    let colors = t.obj_now(spell).chars.colors;
    assert!(colors.contains(Color::Blue));
    assert!(!colors.contains(Color::Red));
}

#[test]
fn wild_mongrel_gets_bigger_and_becomes_the_color_of_your_choice() {
    cr!("105.3");
    compiles("Wild Mongrel");
    let mut t = TestGame::new(2);
    let mongrel = t.battlefield(P0, "Wild Mongrel");
    t.hand(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(color_index(Color::Red)));
    t.activate(P0, mongrel, 0, &[]).expect("activates");
    t.resolve();
    assert_eq!(t.pt(mongrel), (3, 3));
    let colors = t.obj_now(mongrel).chars.colors;
    assert!(colors.contains(Color::Red) && !colors.contains(Color::Green));
    // A second activation with another color: the later effect wins; the first color
    // was locked in, not changed (CR 608.2h).
    t.hand(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(color_index(Color::Blue)));
    t.activate(P0, mongrel, 0, &[]).expect("activates");
    t.resolve();
    assert_eq!(t.pt(mongrel), (4, 4));
    let colors = t.obj_now(mongrel).chars.colors;
    assert!(colors.contains(Color::Blue) && !colors.contains(Color::Red));
}

#[test]
fn giver_of_runes_protection_from_colorless_or_a_color() {
    cr!("702.16a", "105.2c");
    compiles("Giver of Runes");
    let mut t = TestGame::new(2);
    let giver = t.battlefield(P0, "Giver of Runes");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Option 0 is colorless.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, giver, 0, &[Entity::Object(bears)]).expect("activates");
    t.resolve();
    let o = t.obj_now(bears);
    assert!(o.has_keyword(KeywordKind::Protection));
    let ornithopter = t.battlefield(P1, "Ornithopter");
    assert!(t.g.protected_from(bears, ornithopter));
    // A color instead.
    let mut t = TestGame::new(2);
    let giver = t.battlefield(P0, "Giver of Runes");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(P0, DecisionKind::Option, Answer::Index(1 + color_index(Color::Red)));
    t.activate(P0, giver, 0, &[Entity::Object(bears)]).expect("activates");
    t.resolve();
    let goblin = t.battlefield(P1, "Raging Goblin");
    let thopter = t.battlefield(P1, "Ornithopter");
    assert!(t.g.protected_from(bears, goblin));
    assert!(!t.g.protected_from(bears, thopter));
}

#[test]
fn navigators_compass_basic_land_type_of_your_choice_in_addition() {
    cr!("305.7", "205.1b");
    compiles("Navigator's Compass");
    let mut t = TestGame::new(2);
    let compass = t.battlefield(P0, "Navigator's Compass");
    let forest = t.battlefield(P0, "Forest");
    // Plains, Island, Swamp, Mountain, Forest.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, compass, 0, &[Entity::Object(forest)]).expect("activates");
    t.resolve();
    let subs = &t.obj_now(forest).chars.subtypes;
    assert!(subs.iter().any(|s| s == "Swamp"));
    assert!(subs.iter().any(|s| s == "Forest"));
}

#[test]
fn magus_of_the_abyss_that_player_chooses_the_target() {
    cr!("601.2c");
    compiles("Magus of the Abyss");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Magus of the Abyss");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    t.battlefield(P1, "Ornithopter");
    t.answer_targets(P1, &[Entity::Object(b)]);
    t.set_step(P0, Step::End);
    let from = t.asked().len();
    t.advance_to(P1, Step::Draw);
    let asked: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P1], "the upkeep's player chooses the target");
    assert!(t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn harsh_mercy_destroys_creatures_not_of_a_type_any_player_chose() {
    cr!("101.4", "608.2d");
    compiles("Harsh Mercy");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.answer(P0, DecisionKind::Option, Answer::Index(creature_type("Bear")));
    t.answer(P1, DecisionKind::Option, Answer::Index(creature_type("Elf")));
    let spell = t.hand(P0, "Harsh Mercy");
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.on_battlefield(bears));
    assert!(t.on_battlefield(elves));
    assert!(!t.on_battlefield(goblin));
}

#[test]
fn patriarchs_bidding_returns_creatures_of_every_chosen_type() {
    cr!("101.4");
    compiles("Patriarch's Bidding");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Raging Goblin");
    t.graveyard(P1, "Llanowar Elves");
    t.graveyard(P1, "Gray Ogre");
    t.answer(P0, DecisionKind::Option, Answer::Index(creature_type("Bear")));
    t.answer(P1, DecisionKind::Option, Answer::Index(creature_type("Elf")));
    let spell = t.hand(P0, "Patriarch's Bidding");
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.in_graveyard(P0, "Raging Goblin"));
    assert!(t.in_graveyard(P1, "Gray Ogre"));
}
