//! Chosen-type sweepers and lords ("all creatures of the chosen type", "creatures that
//! aren't of the chosen type"), and game-history conditions used by ETB abilities and
//! prepared cards (threshold, descend, life gained this turn, creatures died this turn).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{subtype_lists, Color};
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn choose_creature_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = subtype_lists()
        .creature
        .iter()
        .position(|s| s == ty)
        .unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

// ---------------------------------------------------------------------------
// The chosen type / color on groups of objects
// ---------------------------------------------------------------------------

#[test]
fn destroy_all_creatures_not_of_the_chosen_type() {
    cr!("607.2d", "608.2c");
    assert_supported("Kindred Judgment");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let their_elf = t.battlefield(P1, "Llanowar Elves");
    t.lands(P0, "Plains", 7);
    let s = t.hand(P0, "Kindred Judgment");
    choose_creature_type(&mut t, P0, "Elf");
    t.cast(P0, s).go();
    t.resolve();
    assert!(t.on_battlefield(elf));
    assert!(t.on_battlefield(their_elf));
    assert!(!t.on_battlefield(bear));
}

#[test]
fn all_creatures_of_the_chosen_type_get_minus_one() {
    cr!("607.2d", "613.4c");
    assert_supported("Engineered Plague");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P1, "Llanowar Elves");
    let bear = t.battlefield(P1, "Grizzly Bears");
    choose_creature_type(&mut t, P0, "Bear");
    t.enter(P0, "Engineered Plague");
    assert_eq!(t.pt(bear), (1, 1));
    assert_eq!(t.pt(elf), (1, 1));
}

#[test]
fn all_slivers_have_protection_from_the_chosen_color() {
    cr!("607.2d", "702.16b");
    assert_supported("Ward Sliver");
    assert_supported("Muscle Sliver");
    let mut t = TestGame::new(2);
    let other = t.battlefield(P1, "Muscle Sliver");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Option,
        Answer::Index(Color::ALL.iter().position(|c| *c == Color::Red).unwrap()),
    );
    let ward = t.enter(P0, "Ward Sliver");
    let goblin = t.battlefield(P1, "Raging Goblin");
    // Every Sliver, including an opponent's, has protection from the Ward Sliver's color.
    assert!(t.g.protected_from(t.g.current(ward), goblin));
    assert!(t.g.protected_from(t.g.current(other), goblin));
    assert!(!t.g.protected_from(t.g.current(bears), goblin));
}

// ---------------------------------------------------------------------------
// Graveyard counts: threshold, descend (permanent cards)
// ---------------------------------------------------------------------------

#[test]
fn threshold_with_seven_cards_in_graveyard() {
    cr!("207.2c", "604.2");
    assert_supported("Mystic Enforcer");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mystic Enforcer");
    for _ in 0..6 {
        t.graveyard(P0, "Lightning Bolt");
    }
    // Cards in an opponent's graveyard don't count.
    t.graveyard(P1, "Lightning Bolt");
    assert_eq!(t.pt(m), (3, 3));
    t.graveyard(P0, "Forest");
    assert_eq!(t.pt(m), (6, 6));
    assert!(t.obj_now(m).has_keyword(KeywordKind::Flying));
}

#[test]
fn descend_counts_permanent_cards() {
    cr!("110.4a", "207.2c");
    assert_supported("Echo of Dusk");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Echo of Dusk");
    for n in [
        "Lightning Bolt",
        "Shock",
        "Grizzly Bears",
        "Forest",
        "Pacifism",
    ] {
        t.graveyard(P0, n);
    }
    // Three permanent cards (creature, land, enchantment): not yet.
    assert_eq!(t.pt(e), (2, 2));
    t.graveyard(P0, "Sol Ring");
    assert_eq!(t.pt(e), (3, 3));
    assert!(t.obj_now(e).has_keyword(KeywordKind::Lifelink));
}

// ---------------------------------------------------------------------------
// This-turn history conditions
// ---------------------------------------------------------------------------

#[test]
fn draws_if_three_or_more_life_was_gained() {
    cr!("603.4");
    assert_supported("The Gaffer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Gaffer");
    let hand = t.hand_size(P0);
    t.g.gain_life(P0, 2);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), hand, "only 2 life gained");
    // Gaining 3 life during the opponent's turn: "each end step" includes theirs.
    t.g.gain_life(P0, 3);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn becomes_prepared_if_two_creatures_died() {
    cr!("722.3a", "603.4");
    assert_supported("Emeritus of Woe // Demonic Tutor");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Emeritus of Woe // Demonic Tutor");
    assert!(t.obj_now(e).prepared.is_none());
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(e).prepared.is_none(), "no creatures died");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 2);
    let bolt1 = t.hand(P0, "Lightning Bolt");
    let bolt2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt1).target(a).go();
    t.resolve();
    t.cast(P0, bolt2).target(b).go();
    t.resolve();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(e).prepared.is_some());
}
