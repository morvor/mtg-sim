//! Conditional statics and "for each" amounts: opponents' hands, graveyards and poison
//! counters, the most common color, devotion CDAs, statics that function from the
//! graveyard, and "has flash as long as".

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
// Conditions
// ---------------------------------------------------------------------------

#[test]
fn opponent_with_no_cards_in_hand() {
    cr!("611.3a", "613.4c");
    compiles("Guul Draz Specter");
    let mut t = TestGame::new(2);
    let specter = t.battlefield(P0, "Guul Draz Specter");
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.pt(specter), (5, 5));
    t.hand(P1, "Island");
    t.settle();
    assert_eq!(t.pt(specter), (2, 2));
}

#[test]
fn no_opponent_controls_a_creature() {
    cr!("611.3a");
    compiles("Vexing Beetle");
    let mut t = TestGame::new(2);
    let beetle = t.battlefield(P0, "Vexing Beetle");
    t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(beetle), (6, 6));
    t.battlefield(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(beetle), (3, 3));
}

#[test]
fn another_merfolk_or_an_island() {
    cr!("611.3a");
    compiles("Kumena's Speaker");
    let mut t = TestGame::new(2);
    let speaker = t.battlefield(P0, "Kumena's Speaker");
    // It's a Merfolk itself: that doesn't count.
    assert_eq!(t.pt(speaker), (1, 1));
    t.battlefield(P1, "Island");
    t.settle();
    assert_eq!(t.pt(speaker), (1, 1));
    t.battlefield(P0, "Island");
    t.settle();
    assert_eq!(t.pt(speaker), (2, 2));
}

#[test]
fn has_infect_while_an_opponent_is_poisoned() {
    cr!("611.3a", "122.1f");
    compiles("Viridian Betrayers");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Viridian Betrayers");
    assert!(!has(&t, b, KeywordKind::Infect));
    t.g.add_counters(Entity::Player(P0), "poison", 1, None);
    t.settle();
    assert!(!has(&t, b, KeywordKind::Infect));
    t.g.add_counters(Entity::Player(P1), "poison", 1, None);
    t.settle();
    assert!(has(&t, b, KeywordKind::Infect));
}

#[test]
fn djinn_shrinks_while_its_color_is_most_common_or_tied() {
    cr!("611.3a");
    compiles("Zanam Djinn");
    let mut t = TestGame::new(2);
    let djinn = t.battlefield(P0, "Zanam Djinn");
    assert_eq!(t.pt(djinn), (3, 4));
    // Tied with green: still most common.
    t.battlefield(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(djinn), (3, 4));
    t.battlefield(P1, "Llanowar Elves");
    t.settle();
    assert_eq!(t.pt(djinn), (5, 6));
}

#[test]
fn a_kind_of_card_in_your_graveyard() {
    cr!("611.3a");
    compiles("Vengeful Firebrand");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Vengeful Firebrand");
    t.graveyard(P1, "Goblin Piker");
    t.settle();
    assert!(!has(&t, f, KeywordKind::Haste));
    // Goblin Piker is a Goblin Warrior.
    t.graveyard(P0, "Goblin Piker");
    t.settle();
    assert!(has(&t, f, KeywordKind::Haste));
}

// ---------------------------------------------------------------------------
// Amounts
// ---------------------------------------------------------------------------

#[test]
fn for_each_other_creature_named_this() {
    cr!("613.4c", "201.2a");
    compiles("Timberpack Wolf");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Timberpack Wolf");
    assert_eq!(t.pt(a), (2, 2));
    let b = t.battlefield(P0, "Timberpack Wolf");
    t.battlefield(P1, "Timberpack Wolf");
    t.settle();
    assert_eq!(t.pt(a), (3, 3));
    assert_eq!(t.pt(b), (3, 3));
}

#[test]
fn for_each_other_creature_with_a_counter() {
    cr!("613.4c");
    compiles("High Sentinels of Arashin");
    let mut t = TestGame::new(2);
    let hs = t.battlefield(P0, "High Sentinels of Arashin");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(hs), "+1/+1", 1, None);
    t.settle();
    // Its own counter counts for its P/T, not for "each other creature".
    assert_eq!(t.pt(hs), (4, 5));
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    t.settle();
    assert_eq!(t.pt(hs), (5, 6));
}

#[test]
fn for_each_creature_and_each_creature_card_in_your_graveyard() {
    cr!("613.4c");
    compiles("Moon-Vigil Adherents");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Moon-Vigil Adherents");
    t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Island");
    t.graveyard(P1, "Hill Giant");
    t.settle();
    // Two creatures (itself included) plus one creature card in your graveyard.
    assert_eq!(t.pt(m), (3, 3));
}

#[test]
fn for_each_poison_counter_your_opponents_have() {
    cr!("613.4c", "122.1f");
    compiles("Mycosynth Fiend");
    let mut t = TestGame::new(3);
    let f = t.battlefield(P0, "Mycosynth Fiend");
    t.g.add_counters(Entity::Player(P1), "poison", 2, None);
    t.g.add_counters(Entity::Player(P2), "poison", 1, None);
    t.g.add_counters(Entity::Player(P0), "poison", 5, None);
    t.settle();
    assert_eq!(t.pt(f), (5, 5));
}

#[test]
fn for_each_card_in_your_opponents_hands_and_graveyards() {
    cr!("613.4c");
    compiles("Enemy of Enlightenment");
    compiles("Wight of Precinct Six");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Enemy of Enlightenment");
    let w = t.battlefield(P0, "Wight of Precinct Six");
    t.hand(P1, "Island");
    t.hand(P1, "Island");
    t.hand(P0, "Island");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Island");
    t.graveyard(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(e), (3, 3));
    assert_eq!(t.pt(w), (2, 2));
}

#[test]
fn power_equal_to_devotion() {
    cr!("604.3", "700.5");
    compiles("Renata, Called to the Hunt");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Renata, Called to the Hunt");
    // {2}{G}{G}: devotion to green 2.
    assert_eq!(t.pt(r), (2, 3));
    t.battlefield(P0, "Llanowar Elves");
    t.settle();
    assert_eq!(t.pt(r), (3, 3));
}

#[test]
fn plague_rats_count_each_other() {
    cr!("604.3", "201.2a");
    compiles("Plague Rats");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Plague Rats");
    t.battlefield(P1, "Plague Rats");
    t.settle();
    assert_eq!(t.pt(a), (2, 2));
}

// ---------------------------------------------------------------------------
// Where abilities function
// ---------------------------------------------------------------------------

#[test]
fn incarnation_works_from_the_graveyard() {
    cr!("113.6", "611.3a");
    compiles("Wonder");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wonder = t.battlefield(P0, "Wonder");
    // On the battlefield it does nothing for others.
    assert!(!has(&t, bears, KeywordKind::Flying));
    t.g.destroy(wonder, None);
    t.settle();
    assert!(!has(&t, bears, KeywordKind::Flying));
    t.battlefield(P0, "Island");
    t.settle();
    assert!(has(&t, bears, KeywordKind::Flying));
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(!has(&t, theirs, KeywordKind::Flying));
}

#[test]
fn has_flash_as_long_as_you_control_a_kind() {
    cr!("601.3d", "702.8a");
    compiles("Colossal Rattlewurm");
    compiles("Crashing Tide");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Island", 3);
    let wurm = t.hand(P0, "Colossal Rattlewurm");
    let tide = t.hand(P0, "Crashing Tide");
    let target = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, wurm).try_go().is_err());
    assert!(t.cast(P0, tide).target(target).try_go().is_err());
    t.battlefield(P0, "Desert");
    t.battlefield(P0, "Kumena's Speaker");
    t.g.recompute();
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, wurm).try_go().is_ok());
    t.resolve_all();
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, tide).target(target).try_go().is_ok());
}

#[test]
fn opponents_below_half_their_starting_life() {
    cr!("611.3a", "613.4c");
    compiles("Anya, Merciless Angel");
    let mut t = TestGame::new(3);
    let anya = t.battlefield(P0, "Anya, Merciless Angel");
    assert_eq!(t.pt(anya), (4, 4));
    assert!(!has(&t, anya, KeywordKind::Indestructible));
    // Exactly half isn't less than half.
    t.g.lose_life(P1, 10);
    t.settle();
    assert_eq!(t.pt(anya), (4, 4));
    assert!(!has(&t, anya, KeywordKind::Indestructible));
    t.g.lose_life(P1, 1);
    t.settle();
    assert_eq!(t.pt(anya), (7, 7));
    assert!(has(&t, anya, KeywordKind::Indestructible));
    // Your own life total doesn't count.
    t.g.lose_life(P0, 15);
    t.settle();
    assert_eq!(t.pt(anya), (7, 7));
    t.g.lose_life(P2, 11);
    t.settle();
    assert_eq!(t.pt(anya), (10, 10));
}

#[test]
fn half_an_odd_starting_life_total_is_a_fraction() {
    ruling!("Anya, Merciless Angel", "half that number is 20");
    cr!("611.3a", "103.4");
    let config = mtg_engine::game::GameConfig {
        starting_life: 41,
        ..Default::default()
    };
    let mut t = TestGame::with_config(2, config);
    let anya = t.battlefield(P0, "Anya, Merciless Angel");
    // 41 → 21: not below 20½.
    t.g.lose_life(P1, 20);
    t.settle();
    assert!(!has(&t, anya, KeywordKind::Indestructible));
    assert_eq!(t.pt(anya), (4, 4));
    // 20 is below 20½.
    t.g.lose_life(P1, 1);
    t.settle();
    assert!(has(&t, anya, KeywordKind::Indestructible));
    assert_eq!(t.pt(anya), (7, 7));
}
