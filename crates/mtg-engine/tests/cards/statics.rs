//! Static abilities compiled by `oracle/patterns/statics.rs`: anthems over groups,
//! conditional statics ("as long as", "during your turn"), and "for each" values.

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

// ---------------------------------------------------------------------------
// Anthems over groups
// ---------------------------------------------------------------------------

#[test]
fn all_sliver_creatures_get_a_bonus_on_both_sides() {
    cr!("611.3a", "613.4c");
    compiles("Muscle Sliver");
    let mut t = TestGame::new(2);
    let muscle = t.battlefield(P0, "Muscle Sliver");
    let theirs = t.battlefield(P1, "Metallic Sliver");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "All Sliver creatures" includes itself and opponents' Slivers.
    assert_eq!(t.pt(muscle), (2, 2));
    assert_eq!(t.pt(theirs), (2, 2));
    assert_eq!(t.pt(bears), (2, 2));
    // A second Muscle Sliver pumps every Sliver again.
    let second = t.battlefield(P1, "Muscle Sliver");
    assert_eq!(t.pt(muscle), (3, 3));
    assert_eq!(t.pt(second), (3, 3));
    assert_eq!(t.pt(theirs), (3, 3));
}

#[test]
fn listed_subtypes_share_the_controller_suffix() {
    cr!("611.3a", "613.4c");
    compiles("Master Trinketeer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Master Trinketeer");
    let mine = t.battlefield(P0, "Ornithopter");
    let theirs = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Servos and Thopters you control get +1/+1."
    assert_eq!(t.pt(mine), (1, 3));
    assert_eq!(t.pt(theirs), (0, 2));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn two_whole_groups_joined_by_and() {
    cr!("613.1f");
    compiles("Caterwauling Boggart");
    let mut t = TestGame::new(2);
    let boggart = t.battlefield(P0, "Caterwauling Boggart");
    let goblin = t.battlefield(P0, "Raging Goblin");
    let elemental = t.battlefield(P0, "Air Elemental");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let their_goblin = t.battlefield(P1, "Raging Goblin");
    // "Goblins you control and Elementals you control have menace."
    assert!(has(&t, boggart, KeywordKind::Menace));
    assert!(has(&t, goblin, KeywordKind::Menace));
    assert!(has(&t, elemental, KeywordKind::Menace));
    assert!(!has(&t, bears, KeywordKind::Menace));
    assert!(!has(&t, their_goblin, KeywordKind::Menace));
}

// ---------------------------------------------------------------------------
// Conditional statics
// ---------------------------------------------------------------------------

#[test]
fn during_your_turn_first_strike() {
    cr!("611.3a", "702.7b");
    compiles("Fresh-Faced Recruit");
    // On its controller's turn, the 2/1 Recruit strikes first and survives.
    let mut t = TestGame::new(2);
    let recruit = t.battlefield(P0, "Fresh-Faced Recruit");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(has(&t, recruit, KeywordKind::FirstStrike));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(recruit, Entity::Player(P1))], &[(bears, recruit)]);
    assert!(t.on_battlefield(recruit));
    assert!(!t.on_battlefield(bears));

    // On the opponent's turn it has no first strike: blocking a Bears, both die.
    let mut t = TestGame::new(2);
    let recruit = t.battlefield(P0, "Fresh-Faced Recruit");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!has(&t, recruit, KeywordKind::FirstStrike));
    t.attack(&[(bears, Entity::Player(P0))], &[(recruit, bears)]);
    assert!(!t.on_battlefield(recruit));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn during_turns_other_than_yours() {
    cr!("611.3a");
    compiles("Mesa Lynx");
    let mut t = TestGame::new(2);
    let lynx = t.battlefield(P0, "Mesa Lynx");
    assert_eq!(t.pt(lynx), (2, 1));
    t.set_step(P1, Step::PrecombatMain);
    assert_eq!(t.pt(lynx), (2, 3));
    // It changes back as its controller's turn begins.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.pt(lynx), (2, 1));
}

#[test]
fn threshold_counts_cards_in_your_graveyard() {
    cr!("611.3a", "613.4c");
    compiles("Krosan Beast");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Krosan Beast");
    for _ in 0..6 {
        t.graveyard(P0, "Grizzly Bears");
    }
    // Cards in an opponent's graveyard don't count.
    t.graveyard(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(beast), (1, 1));
    t.graveyard(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(beast), (8, 8));
}

#[test]
fn as_long_as_it_is_attacking() {
    cr!("611.3a", "702.7b", "511.3");
    compiles("Kor Scythemaster");
    let mut t = TestGame::new(2);
    let kor = t.battlefield(P0, "Kor Scythemaster");
    assert!(!has(&t, kor, KeywordKind::FirstStrike));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    // The 3/1 attacker has first strike, so the blocking 2/2 dies first.
    t.attack(&[(kor, Entity::Player(P1))], &[(bears, kor)]);
    assert!(t.on_battlefield(kor));
    assert!(!t.on_battlefield(bears));
    // Still attacking during the end of combat step (CR 511.3), but not afterwards.
    assert!(has(&t, kor, KeywordKind::FirstStrike));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!has(&t, kor, KeywordKind::FirstStrike));
}

#[test]
fn as_long_as_you_control_a_land_type() {
    cr!("611.3a", "613.4c");
    compiles("Loam Lion");
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P0, "Loam Lion");
    // An opponent's Forest doesn't count.
    t.battlefield(P1, "Forest");
    assert_eq!(t.pt(lion), (1, 1));
    t.battlefield(P0, "Forest");
    assert_eq!(t.pt(lion), (2, 3));
}

#[test]
fn as_long_as_an_opponent_has_little_life() {
    cr!("611.3a", "613.1f", "613.4c");
    compiles("Guul Draz Vampire");
    let mut t = TestGame::new(2);
    let vamp = t.battlefield(P0, "Guul Draz Vampire");
    t.lose_life(P1, 9);
    t.settle();
    assert_eq!(t.life(P1), 11);
    assert_eq!(t.pt(vamp), (1, 1));
    assert!(!has(&t, vamp, KeywordKind::Intimidate));
    t.lose_life(P1, 1);
    t.settle();
    assert_eq!(t.pt(vamp), (3, 2));
    assert!(has(&t, vamp, KeywordKind::Intimidate));
    // Gaining life turns it off again.
    t.gain_life(P1, 5);
    t.settle();
    assert_eq!(t.pt(vamp), (1, 1));
}

#[test]
fn hellbent_with_no_cards_in_hand() {
    cr!("611.3a");
    compiles("Demon's Jester");
    let mut t = TestGame::new(2);
    let jester = t.battlefield(P0, "Demon's Jester");
    let card = t.hand(P0, "Grizzly Bears");
    assert_eq!(t.pt(jester), (2, 2));
    t.discard(P0, card, None);
    t.settle();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.pt(jester), (4, 3));
}

#[test]
fn gods_are_creatures_only_with_enough_devotion() {
    cr!("700.5", "205.1a", "613.1d");
    ruling!(
        "Erebos, God of the Dead",
        "loses the type creature and the creature type God"
    );
    compiles("Erebos, God of the Dead");
    let mut t = TestGame::new(2);
    let erebos = t.battlefield(P0, "Erebos, God of the Dead");
    // Devotion to black is 1: not a creature, and not a God either.
    assert!(!t.obj_now(erebos).is_creature());
    assert!(t
        .obj_now(erebos)
        .is(mtg_engine::types::CardType::Enchantment));
    assert!(!t.obj_now(erebos).chars.has_subtype("God"));
    // {B}{B}{B}{B} brings devotion to five.
    t.battlefield(P0, "Phyrexian Obliterator");
    assert!(t.obj_now(erebos).is_creature());
    assert!(t.obj_now(erebos).chars.has_subtype("God"));
    assert_eq!(t.pt(erebos), (5, 7));
    // Black permanents an opponent controls don't count.
    let mut t = TestGame::new(2);
    let erebos = t.battlefield(P0, "Erebos, God of the Dead");
    t.battlefield(P1, "Phyrexian Obliterator");
    assert!(!t.obj_now(erebos).is_creature());
}

// ---------------------------------------------------------------------------
// "for each" and "where X is"
// ---------------------------------------------------------------------------

#[test]
fn gets_plus_one_for_each_artifact_you_control() {
    cr!("611.3a", "613.4c");
    compiles("Nim Lasher");
    let mut t = TestGame::new(2);
    let lasher = t.battlefield(P0, "Nim Lasher");
    assert_eq!(t.pt(lasher), (1, 1));
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P0, "Memnite");
    t.battlefield(P1, "Sol Ring");
    assert_eq!(t.pt(lasher), (3, 1));
}

#[test]
fn gets_plus_two_for_each_aura_attached_to_it() {
    cr!("613.4c");
    compiles("Graceblade Artisan");
    let mut t = TestGame::new(2);
    let artisan = t.battlefield(P0, "Graceblade Artisan");
    let aura = t.battlefield(P0, "Holy Strength");
    t.attach(aura, Entity::Object(artisan));
    t.settle();
    // +2/+2 for the Aura, +1/+2 from Holy Strength itself.
    assert_eq!(t.pt(artisan), (5, 7));
    // An Aura on another creature doesn't count.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Holy Strength");
    t.attach(other, Entity::Object(bears));
    t.settle();
    assert_eq!(t.pt(artisan), (5, 7));
}

#[test]
fn gets_minus_for_each_card_in_hand() {
    cr!("613.4c");
    compiles("Dread Slag");
    let mut t = TestGame::new(2);
    let slag = t.battlefield(P0, "Dread Slag");
    assert_eq!(t.pt(slag), (9, 9));
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    assert_eq!(t.pt(slag), (5, 5));
}

#[test]
fn where_x_is_the_number_of_creature_cards_in_your_graveyard() {
    cr!("613.4c");
    compiles("Wreath of Geists");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wreath = t.battlefield(P0, "Wreath of Geists");
    t.attach(wreath, Entity::Object(bears));
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Sol Ring");
    t.graveyard(P1, "Hill Giant");
    t.settle();
    assert_eq!(t.pt(bears), (3, 3));
}
