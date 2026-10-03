//! What a permanent enters with (CR 614.1c): abilities ("it enters with ... and with
//! haste"), several kinds of counters, a choice of counters, characteristics set by an
//! "as this enters" ability (CR 707.2), and characteristic-changing statics using the
//! choice made as it entered (CR 607.2d).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{subtype_lists, Color, ColorSet};
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn choose_color(t: &mut TestGame, p: PlayerId, c: Color) {
    let i = Color::ALL.iter().position(|x| *x == c).unwrap();
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

fn choose_subtype(t: &mut TestGame, p: PlayerId, list: &[String], ty: &str) {
    let i = list.iter().position(|s| s == ty).expect("subtype");
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

// ---------------------------------------------------------------------------
// "If this creature was kicked, it enters with N +1/+1 counters on it and with haste."
// ---------------------------------------------------------------------------

#[test]
fn kicked_creature_enters_with_counters_and_haste() {
    cr!("614.1c", "702.33d");
    ruling!(
        "Pouncing Wurm",
        "If the Pouncing Wurm is kicked, Pouncing Wurm has haste permanently."
    );
    assert_supported("Pouncing Wurm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let w = t.hand(P0, "Pouncing Wurm");
    t.cast(P0, w).kicked(true).go();
    t.resolve();
    assert_eq!(t.counters(w, "+1/+1"), 3);
    assert_eq!(t.pt(w), (6, 6));
    assert!(t.obj_now(w).has_keyword(KeywordKind::Haste));
    // It keeps haste for as long as it's on the battlefield.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(w).has_keyword(KeywordKind::Haste));
}

#[test]
fn unkicked_creature_enters_without_haste() {
    cr!("614.1c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let w = t.hand(P0, "Pouncing Wurm");
    t.cast(P0, w).kicked(false).go();
    t.resolve();
    assert_eq!(t.counters(w, "+1/+1"), 0);
    assert!(!t.obj_now(w).has_keyword(KeywordKind::Haste));
}

#[test]
fn kicked_creature_enters_with_two_kinds_of_counters_and_haste() {
    cr!("614.1c", "122.1b", "702.33d");
    assert_supported("Voidpouncer");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    t.lands(P0, "Wastes", 1);
    let v = t.hand(P0, "Voidpouncer");
    t.cast(P0, v).kicked(true).go();
    t.resolve();
    assert_eq!(t.counters(v, "+1/+1"), 2);
    assert_eq!(t.counters(v, "trample"), 1);
    assert_eq!(t.pt(v), (5, 3));
    assert!(t.obj_now(v).has_keyword(KeywordKind::Trample));
    assert!(t.obj_now(v).has_keyword(KeywordKind::Haste));
}

#[test]
fn enters_with_counters_of_two_kinds_if_condition_holds() {
    cr!("614.1c", "122.1b");
    assert_supported("Dust Animus");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let a = t.enter(P0, "Dust Animus");
    assert_eq!(t.counters(a, "+1/+1"), 2);
    assert_eq!(t.counters(a, "lifelink"), 1);
    assert!(t.obj_now(a).has_keyword(KeywordKind::Lifelink));
    // With only four untapped lands, no counters.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let a = t.enter(P0, "Dust Animus");
    assert_eq!(t.counters(a, "+1/+1"), 0);
    assert_eq!(t.counters(a, "lifelink"), 0);
}

// ---------------------------------------------------------------------------
// Two replacement effects on the same event; "an additional X counters"
// ---------------------------------------------------------------------------

#[test]
fn additional_counters_if_x_is_large() {
    cr!("614.1c", "107.3m");
    assert_supported("Apocalypse Hydra");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 4);
    let h = t.hand(P0, "Apocalypse Hydra");
    t.cast(P0, h).x(5).go();
    t.resolve();
    assert_eq!(t.counters(h, "+1/+1"), 10);

    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 3);
    let h = t.hand(P0, "Apocalypse Hydra");
    t.cast(P0, h).x(4).go();
    t.resolve();
    assert_eq!(t.counters(h, "+1/+1"), 4);
}

#[test]
fn counters_for_each_creature_the_chosen_opponent_controls() {
    cr!("614.12a", "607.2d", "614.1c");
    assert_supported("Canker Abomination");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P2, "Llanowar Elves");
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let c = t.enter(P0, "Canker Abomination");
    assert_eq!(t.obj_now(c).choices.player, Some(P2));
    assert_eq!(t.counters(c, "-1/-1"), 2);
    assert_eq!(t.pt(c), (4, 4));
}

#[test]
fn counters_for_each_of_two_kinds_of_creatures() {
    cr!("614.1c", "122.6");
    assert_supported("Ulasht, the Hate Seed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging Goblin");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Raging Goblin");
    let u = t.enter(P0, "Ulasht, the Hate Seed");
    // One other red creature and two other green creatures you control.
    assert_eq!(t.counters(u, "+1/+1"), 3);
}

// ---------------------------------------------------------------------------
// A choice of counters
// ---------------------------------------------------------------------------

#[test]
fn choice_of_a_two_word_keyword_counter() {
    cr!("614.1c", "614.12a", "122.1b");
    assert_supported("Helica Glider");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let g = t.enter(P0, "Helica Glider");
    assert_eq!(t.counters(g, "first strike"), 1);
    assert_eq!(t.counters(g, "flying"), 0);
    assert!(t.obj_now(g).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn choice_of_two_different_counters() {
    cr!("614.1c", "614.12a", "122.1b");
    assert_supported("Grimdancer");
    let mut t = TestGame::new(2);
    // Options: menace+deathtouch, menace+lifelink, deathtouch+lifelink.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let g = t.enter(P0, "Grimdancer");
    assert_eq!(t.counters(g, "menace"), 0);
    assert_eq!(t.counters(g, "deathtouch"), 1);
    assert_eq!(t.counters(g, "lifelink"), 1);
    assert!(t.obj_now(g).has_keyword(KeywordKind::Deathtouch));
    assert!(t.obj_now(g).has_keyword(KeywordKind::Lifelink));
}

// ---------------------------------------------------------------------------
// "As this creature enters, it becomes your choice of ..." (CR 707.2)
// ---------------------------------------------------------------------------

#[test]
fn becomes_the_chosen_creature_as_it_enters() {
    cr!("614.1c", "614.12a");
    ruling!(
        "Primal Plasma",
        "While not on the battlefield, Primal Plasma is a 0/0 creature card"
    );
    assert_supported("Primal Plasma");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Primal Plasma");
    assert!(!t.obj_now(in_hand).has_keyword(KeywordKind::Flying));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let p = t.enter(P0, "Primal Plasma");
    t.settle();
    assert!(t.on_battlefield(p));
    assert_eq!(t.pt(p), (2, 2));
    assert!(t.obj_now(p).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(p).has_keyword(KeywordKind::Defender));
}

#[test]
fn a_copy_gets_the_chosen_characteristics_and_makes_its_own_choice() {
    cr!("707.2", "614.12");
    ruling!(
        "Primal Plasma",
        "its power and toughness are determined by the copy's own enters-the-battlefield replacement effect"
    );
    let mut t = TestGame::new(2);
    // Primal Plasma as a 2/2 with flying.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    let p = t.enter(P0, "Primal Plasma");
    // Clone copies it (flying is part of the copiable values) and chooses 1/6 defender.
    t.answer_choose(P0, &[Entity::Object(p)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let c = t.enter(P0, "Clone");
    t.settle();
    assert_eq!(t.obj_now(c).chars.name.as_str(), "Primal Plasma");
    assert_eq!(t.pt(c), (1, 6));
    assert!(t.obj_now(c).has_keyword(KeywordKind::Flying));
    assert!(t.obj_now(c).has_keyword(KeywordKind::Defender));
    // The original is unchanged.
    assert_eq!(t.pt(p), (2, 2));
    assert!(!t.obj_now(p).has_keyword(KeywordKind::Defender));
}

#[test]
fn corrupted_shapeshifter_modes() {
    cr!("614.1c", "707.2");
    assert_supported("Corrupted Shapeshifter");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let s = t.enter(P0, "Corrupted Shapeshifter");
    assert_eq!(t.pt(s), (0, 12));
    assert!(t.obj_now(s).has_keyword(KeywordKind::Defender));
}

// ---------------------------------------------------------------------------
// "[Permanents] are the chosen color / type" (layers 4 and 5)
// ---------------------------------------------------------------------------

#[test]
fn creature_is_the_chosen_color() {
    cr!("105.3", "607.2d", "613.1e");
    assert_supported("Alloy Golem");
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, Color::Red);
    let g = t.enter(P0, "Alloy Golem");
    assert_eq!(t.obj_now(g).chars.colors, ColorSet::single(Color::Red));
    assert!(t.obj_now(g).chars.is(types::CardType::Artifact));
}

#[test]
fn all_nonland_permanents_are_the_chosen_color() {
    cr!("105.3", "607.2d", "613.1e");
    assert_supported("Shifting Sky");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    choose_color(&mut t, P0, Color::Blue);
    let sky = t.enter(P0, "Shifting Sky");
    assert_eq!(t.obj_now(bears).chars.colors, ColorSet::single(Color::Blue));
    assert_eq!(t.obj_now(sky).chars.colors, ColorSet::single(Color::Blue));
    assert!(t.obj_now(forest).chars.colors.is_colorless());
    // A creature cast later is also blue while on the battlefield.
    let later = t.battlefield(P1, "Raging Goblin");
    assert_eq!(t.obj_now(later).chars.colors, ColorSet::single(Color::Blue));
}

#[test]
fn creatures_you_control_are_the_chosen_type_too() {
    cr!("205.1b", "607.2d", "613.1d");
    assert_supported("Xenograft");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let lists = subtype_lists();
    choose_subtype(&mut t, P0, &lists.creature, "Elf");
    t.enter(P0, "Xenograft");
    assert!(t.obj_now(mine).chars.has_subtype("Elf"));
    assert!(t.obj_now(mine).chars.has_subtype("Bear"));
    assert!(!t.obj_now(theirs).chars.has_subtype("Elf"));
}

#[test]
fn lands_you_control_gain_the_chosen_basic_land_type() {
    cr!("305.6", "305.7", "607.2d");
    assert_supported("Realmwright");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let forest = t.battlefield(P0, "Forest");
    choose_subtype(
        &mut t,
        P0,
        &["Plains", "Island", "Swamp", "Mountain", "Forest"].map(String::from),
        "Swamp",
    );
    t.enter(P0, "Realmwright");
    assert!(t.obj_now(forest).chars.has_subtype("Swamp"));
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
    // The Forest can now also tap for {B} (CR 305.6).
    let now = t.g.current(forest);
    t.activate(P0, now, 1, &[]).unwrap();
    assert_eq!(
        t.g.players[0]
            .mana_pool
            .count(mtg_engine::mana::ManaType::B),
        1
    );
}

// ---------------------------------------------------------------------------
// "the chosen number"
// ---------------------------------------------------------------------------

#[test]
fn spells_with_the_chosen_mana_value_cant_be_cast() {
    cr!("607.2d", "601.2", "202.3");
    assert_supported("Sanctum Prelate");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Number, Answer::Number(1));
    let p = t.enter(P0, "Sanctum Prelate");
    assert_eq!(t.obj_now(p).choices.number, Some(1));
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 3);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spear = t.hand(P1, "Searing Spear");
    let goblin = t.hand(P1, "Raging Goblin");
    assert!(t.cast(P1, bolt).target(P0).try_go().is_err());
    // A creature spell with that mana value isn't affected.
    assert!(t.cast(P1, goblin).try_go().is_ok());
    t.resolve();
    assert!(t.cast(P1, spear).target(P0).try_go().is_ok());
}

#[test]
fn destroy_creatures_with_power_at_least_the_chosen_number() {
    cr!("607.2d", "608.2c");
    assert_supported("Expel the Interlopers");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 5);
    let s = t.hand(P0, "Expel the Interlopers");
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.cast(P0, s).go();
    t.resolve();
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(bears));
}
