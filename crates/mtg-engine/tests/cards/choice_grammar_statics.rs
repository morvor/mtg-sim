//! Statics giving objects the type or color chosen as the permanent entered (CR 607.2d),
//! wherever those objects are (CR 611.3c: a static ability's effect applies to the
//! objects it describes, also outside the battlefield).

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
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

fn subtypes(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.obj_now(id).chars.subtypes.iter().map(|s| s.to_string()).collect()
}

#[test]
fn conspiracy_creatures_are_only_the_chosen_type_everywhere() {
    cr!("607.2d", "205.1a", "611.3a");
    compiles("Conspiracy");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Llanowar Elves");
    let in_gy = t.graveyard(P0, "Hill Giant");
    let in_library = t.library_top(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let their_card = t.hand(P1, "Hill Giant");
    let land = t.hand(P0, "Forest");
    t.answer(P0, DecisionKind::Option, Answer::Index(creature_type("Goblin")));
    t.enter(P0, "Conspiracy");
    for id in [bears, in_hand, in_gy, in_library] {
        assert_eq!(subtypes(&mut t, id), vec!["Goblin".to_string()]);
    }
    assert_eq!(subtypes(&mut t, theirs), vec!["Bear".to_string()]);
    assert_eq!(subtypes(&mut t, their_card), vec!["Giant".to_string()]);
    assert_eq!(subtypes(&mut t, land), vec!["Forest".to_string()]);
}

#[test]
fn rukarumel_nontoken_creatures_have_the_chosen_type_in_addition() {
    cr!("607.2d", "611.3a");
    compiles("Rukarumel, Biologist");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let in_gy = t.graveyard(P0, "Hill Giant");
    t.answer(P0, DecisionKind::Option, Answer::Index(creature_type("Sliver")));
    let ruka = t.enter(P0, "Rukarumel, Biologist");
    let mut b = subtypes(&mut t, bears);
    b.sort();
    assert_eq!(b, vec!["Bear".to_string(), "Sliver".to_string()]);
    assert!(subtypes(&mut t, in_gy).contains(&"Sliver".to_string()));
    assert!(subtypes(&mut t, ruka).contains(&"Sliver".to_string()));
}

#[test]
fn ashes_of_the_fallen_creature_cards_in_your_graveyard() {
    cr!("607.2d", "611.3a");
    compiles("Ashes of the Fallen");
    let mut t = TestGame::new(2);
    let in_gy = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let their_gy = t.graveyard(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Option, Answer::Index(creature_type("Zombie")));
    t.enter(P0, "Ashes of the Fallen");
    let mut g = subtypes(&mut t, in_gy);
    g.sort();
    assert_eq!(g, vec!["Giant".to_string(), "Zombie".to_string()]);
    assert_eq!(subtypes(&mut t, bears), vec!["Bear".to_string()]);
    assert_eq!(subtypes(&mut t, their_gy), vec!["Giant".to_string()]);
}

#[test]
fn painters_servant_everything_is_the_chosen_color_too() {
    cr!("607.2d", "105.3", "611.3a");
    compiles("Painter's Servant");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.hand(P1, "Lightning Bolt");
    let gy = t.graveyard(P0, "Hill Giant");
    let lib = t.library_top(P1, "Llanowar Elves");
    let i = Color::ALL.iter().position(|c| *c == Color::Blue).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let servant = t.enter(P0, "Painter's Servant");
    t.g.recompute();
    for id in [bears, card, gy, lib, servant] {
        assert!(t.obj_now(id).chars.colors.contains(Color::Blue), "{id:?}");
    }
    assert!(t.obj_now(bears).chars.colors.contains(Color::Green));
}

#[test]
fn serras_emissary_you_and_your_creatures_have_protection_from_the_chosen_card_type() {
    cr!("607.2d", "702.16b", "702.16k");
    compiles("Serra's Emissary");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let i = CardType::ALL.iter().position(|c| *c == CardType::Instant).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.enter(P0, "Serra's Emissary");
    t.lands(P1, "Mountain", 2);
    t.g.turn.priority = Some(P1);
    let bolt = t.hand(P1, "Lightning Bolt");
    // Neither the creature nor its controller can be targeted: the Bolt goes elsewhere
    // (the only legal target left is P1).
    t.cast(P1, bolt).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears), "{}", t.dump_log());
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
    // Not protected from other card types.
    let shock_src = t.hand(P1, "Lava Axe");
    t.lands(P1, "Mountain", 5);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.cast(P1, shock_src).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 15);
}
