//! Damage and removal amounts that depend on values: the sacrificed creature's power
//! (Fling), "-X/-X where X is ...", "for each" (patterns in
//! `src/oracle/patterns/damage_removal_*.rs` and the shared value grammar).

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn sacrificed_creature_cards_compile() {
    assert_compiles(&[
        "Fling",
        "Thud",
        "Rite of Consumption",
        "Bushmeat Poacher",
    ]);
}

#[test]
fn fling_uses_the_last_known_power_of_the_sacrificed_creature() {
    cr!("118.8", "608.2h");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 2);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, fling).target(P1).go();
    assert!(!t.on_battlefield(bears), "sacrificed as a cost");
    t.resolve();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn a_copy_of_fling_deals_the_same_damage() {
    cr!("707.10");
    ruling!(
        "Melek, Izzet Paragon",
        "if a player sacrifices a 3/3 creature to cast Fling, and you copy it"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let fling = t.cast(P0, fling).target(P1).go();
    // A copy (e.g. from Melek) isn't cast: no creature is sacrificed for it, but it uses
    // the original's sacrificed creature.
    mtg_engine::copy::copy_spell(&mut t.g, fling, P0, false).expect("copied");
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "2 from the copy, 2 from the original");
}

#[test]
fn bushmeat_poacher_gains_life_equal_to_the_sacrificed_creatures_toughness() {
    cr!("118.8", "602.2b");
    let mut t = TestGame::new(2);
    let poacher = t.battlefield(P0, "Bushmeat Poacher");
    let wall = t.battlefield(P0, "Wall of Stone");
    t.lands(P0, "Swamp", 1);
    t.answer_choose(P0, &[Entity::Object(wall)]);
    t.activate(P0, poacher, 0, &[]).unwrap();
    t.resolve();
    // Wall of Stone is a 0/8.
    assert_eq!(t.life(P0), 28);
}

// ---------------------------------------------------------------------------
// -X/-X, "for each", "each get"
// ---------------------------------------------------------------------------

#[test]
fn pt_value_cards_compile() {
    assert_compiles(&[
        "Festive Funeral",
        "Mutilate",
        "Sick and Tired",
        "Terror Tide",
        "Dead of Winter",
        "Timberwatch Elf",
        "Boon of Boseiju",
    ]);
}

#[test]
fn festive_funeral_locks_x_when_it_resolves() {
    cr!("608.2h");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mastodon = t.battlefield(P1, "Siege Mastodon");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Island");
    t.lands(P0, "Swamp", 5);
    let f = t.hand(P0, "Festive Funeral");
    t.cast(P0, f).target(mastodon).go();
    t.resolve();
    // Two cards in the graveyard when it resolved: the 3/5 Mastodon gets -2/-2.
    assert_eq!(t.pt(mastodon), (1, 3));
    // Festive Funeral itself is now in the graveyard; the amount doesn't change.
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.pt(mastodon), (1, 3));
    assert!(t.on_battlefield(bears));
}

#[test]
fn mutilate_counts_swamps_and_affects_only_creatures_there_as_it_resolves() {
    cr!("608.2h", "611.2c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let dreadmaw = t.battlefield(P1, "Colossal Dreadmaw");
    t.lands(P0, "Swamp", 4);
    let m = t.hand(P0, "Mutilate");
    t.cast(P0, m).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.pt(dreadmaw), (2, 2), "6/6 with -4/-4");
    // A creature entering afterwards isn't affected.
    let late = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));
}

#[test]
fn sick_and_tired_shrinks_two_targets() {
    cr!("115.1", "601.2c");
    let mut t = TestGame::new(2);
    let e1 = t.battlefield(P1, "Llanowar Elves");
    let e2 = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Sick and Tired");
    t.cast(P0, s)
        .targets(&[Entity::Object(e1), Entity::Object(e2)])
        .go();
    t.resolve();
    assert!(!t.on_battlefield(e1));
    assert!(!t.on_battlefield(e2));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn terror_tide_counts_only_permanent_cards() {
    cr!("608.2h", "110.4a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let mastodon = t.battlefield(P1, "Siege Mastodon");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Swamp", 4);
    let tide = t.hand(P0, "Terror Tide");
    t.cast(P0, tide).go();
    t.resolve();
    // Forest and Grizzly Bears are permanent cards: -2/-2.
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.pt(mastodon), (1, 3));
}

#[test]
fn timberwatch_elf_counts_elves_on_the_battlefield() {
    cr!("608.2h");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Timberwatch Elf");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, elf, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    // Three Elves (both players').
    assert_eq!(t.pt(bears), (5, 5));
}
