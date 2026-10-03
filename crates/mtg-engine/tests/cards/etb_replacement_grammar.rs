//! Replacement effects that modify how permanents enter the battlefield, compiled by
//! `src/oracle/patterns/etb_replacement_grammar.rs` and the "as ~ enters" instructions of
//! `etb_choices.rs` (CR 614.1c, 614.12, 614.15).

use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn etb_grammar_cards_compile() {
    compiles(&[
        "Captive Audience",
        "Callous Oppressor",
        "Heightened Awareness",
        "Monstrous War-Leech",
        "Urborg Lhurgoyf",
        "Phylactery Lich",
        "Overlaid Terrain",
        "Ixidron",
        "What Must Be Done",
        "Necromantic Summons",
        "Recommission",
        "Heroic Return",
        "Vigor Mortis",
        "Winter Soldier, Reborn Avenger",
        "Zameck Guildmage",
        "Combine Guildmage",
        "Turntimber Symbiosis // Turntimber, Serpentine Wood",
        "Nick Fury, Spymaster",
        "Kari Zev, Skyship Raider",
        "Ral, Monsoon Mage // Ral, Leyline Prodigy",
    ]);
}

#[test]
fn captive_audience_enters_under_an_opponents_control() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(3);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let ca = t.enter(P0, "Captive Audience");
    t.resolve_all();
    let now = t.g.current(ca);
    assert_eq!(t.g.obj(now).controller, P2);
}

#[test]
fn heightened_awareness_discards_your_hand_as_it_enters() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(2);
    t.hand(P0, "Island");
    t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    let h = t.hand(P0, "Heightened Awareness");
    t.cast(P0, h).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Heightened Awareness").len(), 1);
}

#[test]
fn monstrous_war_leech_mills_as_it_enters_only_if_kicked() {
    cr!("614.1c", "702.33d");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    t.lands(P0, "Island", 1);
    let leech = t.hand(P0, "Monstrous War-Leech");
    let lib = t.library_size(P0);
    t.cast(P0, leech).kicked(true).go();
    t.resolve();
    assert_eq!(t.library_size(P0), lib - 4);
    // Not kicked: nothing milled.
    t.lands(P0, "Swamp", 4);
    let leech2 = t.hand(P0, "Monstrous War-Leech");
    t.cast(P0, leech2).kicked(false).go();
    t.resolve();
    assert_eq!(t.library_size(P0), lib - 4);
}

#[test]
fn overlaid_terrain_sacrifices_your_lands_as_it_enters() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 4);
    let o = t.hand(P0, "Overlaid Terrain");
    t.cast(P0, o).go();
    t.resolve();
    assert!(t.named_on_battlefield("Forest").is_empty());
    assert_eq!(t.named_on_battlefield("Overlaid Terrain").len(), 1);
}

#[test]
fn recommission_gives_a_returned_creature_an_additional_counter() {
    cr!("614.1c", "614.15");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    let r = t.hand(P0, "Recommission");
    t.cast(P0, r).target(Entity::Object(bears)).go();
    t.resolve();
    let b = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(b, "+1/+1"), 1);
    // An artifact that isn't a creature gets none.
    let rock = t.graveyard(P0, "Mind Stone");
    t.lands(P0, "Plains", 2);
    let r2 = t.hand(P0, "Recommission");
    t.cast(P0, r2).target(Entity::Object(rock)).go();
    t.resolve();
    let s = t.named_on_battlefield("Mind Stone")[0];
    assert_eq!(t.counters(s, "+1/+1"), 0);
}

#[test]
fn zameck_guildmage_creatures_enter_with_an_additional_counter_this_turn() {
    cr!("614.1c", "122.6");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Zameck Guildmage");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 1);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve();
    let bears = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // An opponent's creature doesn't.
    let theirs = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn phylactery_lich_puts_a_counter_on_an_artifact_as_it_enters() {
    cr!("614.1c", "614.12a");
    let mut t = TestGame::new(2);
    let rock = t.battlefield(P0, "Mind Stone");
    t.answer_choose(P0, &[Entity::Object(rock)]);
    t.enter(P0, "Phylactery Lich");
    t.resolve_all();
    assert_eq!(t.counters(rock, "phylactery"), 1);
    assert_eq!(t.named_on_battlefield("Phylactery Lich").len(), 1);
}
