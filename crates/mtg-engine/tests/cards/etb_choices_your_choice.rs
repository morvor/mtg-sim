//! Values chosen as an effect resolves ("the color of your choice"), locked in for the
//! effect's duration (CR 608.2h).

use mtg_engine::testing::*;
use mtg_engine::types::{Color, ColorSet};
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

#[test]
fn mother_of_runes_grants_protection_from_the_chosen_color() {
    cr!("702.16b", "608.2h", "611.2a");
    assert_supported("Mother of Runes");
    let mut t = TestGame::new(2);
    let mother = t.battlefield(P0, "Mother of Runes");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let merfolk = t.battlefield(P1, "Coral Merfolk");
    choose_color(&mut t, P0, Color::Red);
    t.activate(P0, mother, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert!(t.g.protected_from(t.g.current(bears), goblin));
    assert!(!t.g.protected_from(t.g.current(bears), merfolk));
    // A later choice for another activation doesn't change the first effect.
    let elves = t.battlefield(P0, "Llanowar Elves");
    let m = t.g.current(mother);
    t.g.objects[m.0 as usize].tapped = false;
    choose_color(&mut t, P0, Color::Blue);
    t.activate(P0, mother, 0, &[Entity::Object(elves)]).unwrap();
    t.resolve();
    assert!(t.g.protected_from(t.g.current(bears), goblin));
    assert!(!t.g.protected_from(t.g.current(bears), merfolk));
    assert!(t.g.protected_from(t.g.current(elves), merfolk));
    // Until end of turn.
    t.advance_to(P1, turn::Step::Upkeep);
    assert!(!t.g.protected_from(t.g.current(bears), goblin));
}

#[test]
fn gods_willing_protection_counters_a_red_removal_spell() {
    cr!("702.16b", "608.2b");
    assert_supported("Gods Willing");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.set_step(P1, turn::Step::PrecombatMain);
    t.cast(P1, bolt).target(bears).go();
    // In response, protection from red: the Bolt's only target becomes illegal.
    t.lands(P0, "Plains", 1);
    let gw = t.hand(P0, "Gods Willing");
    choose_color(&mut t, P0, Color::Red);
    t.cast(P0, gw).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
}

#[test]
fn becomes_the_creature_type_of_your_choice() {
    cr!("205.1a", "608.2h", "613.1d");
    assert_supported("Mistform Dreamer");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mistform Dreamer");
    t.lands(P0, "Island", 1);
    let i = types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == "Elf")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.activate(P0, m, 0, &[]).unwrap();
    t.resolve();
    let c = &t.obj_now(m).chars;
    assert!(c.has_subtype("Elf"));
    assert!(
        !c.has_subtype("Illusion"),
        "the new type replaces its creature types"
    );
    t.advance_to(P1, turn::Step::Upkeep);
    assert!(t.obj_now(m).chars.has_subtype("Illusion"));
    assert!(!t.obj_now(m).chars.has_subtype("Elf"));
}

#[test]
fn land_becomes_the_basic_land_type_of_your_choice() {
    cr!("305.7", "608.2h");
    assert_supported("Dream Thrush");
    let mut t = TestGame::new(2);
    let thrush = t.battlefield(P0, "Dream Thrush");
    let mountain = t.battlefield(P1, "Mountain");
    // Plains, Island, Swamp, Mountain, Forest: choose Swamp.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    t.activate(P0, thrush, 0, &[Entity::Object(mountain)])
        .unwrap();
    t.resolve();
    let c = &t.obj_now(mountain).chars;
    assert!(c.has_subtype("Swamp"));
    assert!(!c.has_subtype("Mountain"));
    let now = t.g.current(mountain);
    t.activate(P1, now, 0, &[]).unwrap();
    assert_eq!(
        t.g.players[1]
            .mana_pool
            .count(mtg_engine::mana::ManaType::B),
        1
    );
}

#[test]
fn protection_from_the_card_type_of_your_choice() {
    cr!("702.16b", "608.2h");
    assert_supported("Pippin, Guard of the Citadel");
    let mut t = TestGame::new(2);
    let pippin = t.battlefield(P0, "Pippin, Guard of the Citadel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Card types in CardType::ALL order: artifact, battle, conspiracy, creature, ...
    let i = types::CardType::ALL
        .iter()
        .position(|c| *c == types::CardType::Creature)
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    t.activate(P0, pippin, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    let goblin = t.battlefield(P1, "Raging Goblin");
    let bolt = t.hand(P1, "Lightning Bolt");
    let now = t.g.current(bears);
    assert!(t.g.protected_from(now, goblin));
    assert!(!t.g.protected_from(now, bolt));
}

#[test]
fn becomes_the_color_of_your_choice() {
    cr!("105.3", "613.1e", "608.2h");
    assert_supported("Rainbow Crow");
    let mut t = TestGame::new(2);
    let crow = t.battlefield(P0, "Rainbow Crow");
    t.lands(P0, "Island", 1);
    choose_color(&mut t, P0, Color::Green);
    t.activate(P0, crow, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(crow).chars.colors, ColorSet::single(Color::Green));
    t.advance_to(P1, turn::Step::Upkeep);
    assert_eq!(t.obj_now(crow).chars.colors, ColorSet::single(Color::Blue));
}

// ---------------------------------------------------------------------------
// "choose a creature type. ... of that type" / "~ becomes the chosen color": a value
// chosen earlier in the same ability (CR 607.2d, 608.2c).
// ---------------------------------------------------------------------------

fn choose_creature_type(t: &mut TestGame, p: PlayerId, ty: &str) {
    let i = types::subtype_lists()
        .creature
        .iter()
        .position(|s| s == ty)
        .expect("creature type");
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn permanents_of_that_type_gain_keywords() {
    cr!("607.2d", "608.2c", "611.2c");
    use mtg_engine::keywords::KeywordKind;
    assert_supported("Selfless Safewright");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let enemy_elf = t.battlefield(P1, "Llanowar Elves");
    choose_creature_type(&mut t, P0, "Elf");
    let sw = t.enter(P0, "Selfless Safewright");
    t.resolve_all();
    assert!(t.obj_now(elf).has_keyword(KeywordKind::Hexproof));
    assert!(t.obj_now(elf).has_keyword(KeywordKind::Indestructible));
    assert!(!t.obj_now(bear).has_keyword(KeywordKind::Hexproof));
    assert!(!t.obj_now(enemy_elf).has_keyword(KeywordKind::Hexproof));
    assert!(
        !t.obj_now(sw).has_keyword(KeywordKind::Hexproof),
        "\"other permanents\" excludes the Safewright, an Elf itself"
    );
    // The affected set was locked in as the effect began (CR 611.2c).
    let late = t.battlefield(P0, "Llanowar Elves");
    assert!(!t.obj_now(late).has_keyword(KeywordKind::Hexproof));
    t.advance_to(P1, turn::Step::Upkeep);
    assert!(!t.obj_now(elf).has_keyword(KeywordKind::Hexproof));
}

#[test]
fn artifact_becomes_the_chosen_color() {
    cr!("105.3", "607.2d", "613.1e");
    let c = card("Puca's Eye");
    // The five-colors activation restriction is compiled too (see
    // `cards/misc_game_rules_vivid.rs`).
    assert!(
        c.unsupported_text().is_empty(),
        "{:?}",
        c.unsupported_text()
    );
    let mut t = TestGame::new(2);
    let before = t.hand_size(P0);
    choose_color(&mut t, P0, Color::Red);
    let eye = t.enter(P0, "Puca's Eye");
    assert!(t.obj_now(eye).chars.colors.is_colorless());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before + 1);
    assert_eq!(t.obj_now(eye).chars.colors, ColorSet::single(Color::Red));
    t.advance_to(P1, turn::Step::Upkeep);
    assert_eq!(t.obj_now(eye).chars.colors, ColorSet::single(Color::Red));
}
