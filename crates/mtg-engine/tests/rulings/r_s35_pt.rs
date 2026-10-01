//! Rulings batch S35 — power and toughness (CR 208, 613.4): switching twice, setting
//! base power and toughness again, and a negative power used as an amount (CR 107.1b).

use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::activate_containing;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn switching_power_and_toughness_twice_returns_them() {
    cr!("613.4d");
    ruling!(
        "Aquamoeba",
        "Switching a creature's power and toughness twice (or any even number of times) effectively returns the creature to the power and toughness it had before any switches."
    );
    supported("Aquamoeba");
    supported("Twisted Image");
    // Aquamoeba (1/3): "Discard a card: Switch this creature's power and toughness until
    // end of turn." Twisted Image: "Switch target creature's power and toughness until end
    // of turn. Draw a card."
    let mut t = TestGame::new(2);
    let aq = t.battlefield(P0, "Aquamoeba");
    for _ in 0..4 {
        t.hand(P0, "Forest");
    }
    let mut expected = (1, 3);
    for _ in 0..3 {
        activate_containing(&mut t, P0, aq, "Switch").expect("Aquamoeba");
        t.resolve_all();
        expected = (expected.1, expected.0);
        assert_eq!(t.pt(aq), expected);
    }
    assert_eq!(t.pt(aq), (3, 1));
    t.lands(P0, "Island", 1);
    let image = t.hand(P0, "Twisted Image");
    t.cast(P0, image).target(aq).go();
    t.resolve_all();
    // Four switches: as before any.
    assert_eq!(t.pt(aq), (1, 3));
}

#[test]
fn reanimating_a_keyrune_sets_its_base_power_and_toughness_again() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Dimir Keyrune",
        "Activating the ability that turns the Keyrune into a creature while it's already a creature will override any effects that set its power and/or toughness to another number, but effects that modify power and/or toughness without directly setting them will still apply."
    );
    supported("Dimir Keyrune");
    supported("Diminish");
    supported("Giant Growth");
    // "{U}{B}: This artifact becomes a 2/2 blue and black Horror artifact creature until
    // end of turn and can't be blocked this turn."
    let mut t = TestGame::new(2);
    let keyrune = t.battlefield(P0, "Dimir Keyrune");
    t.lands(P0, "Island", 3);
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Forest", 1);
    activate_containing(&mut t, P0, keyrune, "becomes").expect("Keyrune");
    t.resolve_all();
    assert_eq!(t.pt(keyrune), (2, 2));
    // Diminish: "Target creature has base power and toughness 1/1 until end of turn."
    let diminish = t.hand(P0, "Diminish");
    t.cast(P0, diminish).target(keyrune).go();
    t.resolve_all();
    assert_eq!(t.pt(keyrune), (1, 1));
    // Giant Growth: +3/+3.
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(keyrune).go();
    t.resolve_all();
    assert_eq!(t.pt(keyrune), (4, 4));
    // Activated again: base 2/2 overrides Diminish; Giant Growth still applies.
    activate_containing(&mut t, P0, keyrune, "becomes").expect("Keyrune");
    t.resolve_all();
    assert_eq!(t.pt(keyrune), (5, 5));
}

#[test]
fn a_negative_power_gives_x_of_zero() {
    cr!("107.1b");
    ruling!(
        "Wild Beastmaster",
        "If this creature's power is negative as its ability resolves, X is considered to be 0."
    );
    supported("Wild Beastmaster");
    supported("Shrink");
    supported("Yew Spirit");
    // Wild Beastmaster (1/1): "Whenever this creature attacks, each other creature you
    // control gets +X/+X until end of turn, where X is this creature's power." Shrink
    // ("Target creature gets -5/-0 until end of turn.") makes its power -4.
    let mut t = TestGame::new(2);
    let beastmaster = t.battlefield(P0, "Wild Beastmaster");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let shrink = t.hand(P0, "Shrink");
    t.cast(P0, shrink).target(beastmaster).go();
    t.resolve_all();
    assert_eq!(t.pt(beastmaster), (-4, 1));
    attack_with(&mut t, &[(beastmaster, Entity::Player(P1))]);
    t.resolve_all();
    // The Bears get +0/+0, not -4/-4.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.pt(bears), (2, 2));
    // Yew Spirit (3/3): "{2}{G}{G}: This creature gets +X/+X until end of turn, where X is
    // its power." At -2 power, X is 0.
    let mut t = TestGame::new(2);
    let spirit = t.battlefield(P0, "Yew Spirit");
    t.lands(P0, "Forest", 5);
    let shrink = t.hand(P0, "Shrink");
    t.cast(P0, shrink).target(spirit).go();
    t.resolve_all();
    assert_eq!(t.pt(spirit), (-2, 3));
    activate_containing(&mut t, P0, spirit, "gets").expect("Yew Spirit");
    t.resolve_all();
    assert_eq!(t.pt(spirit), (-2, 3));
}

#[test]
fn a_bonus_from_a_creatures_own_ability_doesnt_change_its_base_power_and_toughness() {
    cr!("208.4b", "604.3", "613.4c");
    ruling!(
        "Duskana, the Rage Mother",
        "Some creatures have base power and toughness 0/0 and an ability that gives them a bonus based on some criteria. Those are not characteristic-defining abilities, and that ability doesn't change its base power and toughness."
    );
    supported("Duskana, the Rage Mother");
    supported("Nighthowler");
    // Nighthowler (0/0): "This creature and enchanted creature each get +X/+X, where X is
    // the number of creature cards in all graveyards." With two, it's 2/2, but its base
    // power and toughness are 0/0. A 1/1 with a +1/+1 counter isn't 2/2 at base either.
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let howler = t.battlefield(P0, "Nighthowler");
    let elves = t.battlefield(P0, "Llanowar Elves");
    crate::r_s13_common::add(&mut t, elves, "+1/+1", 1);
    t.g.recompute();
    assert_eq!(t.pt(howler), (2, 2));
    assert_eq!(t.pt(elves), (2, 2));
    // Duskana: "When Duskana enters, draw a card for each creature you control with base
    // power and toughness 2/2." Only the Bears.
    let hand = t.hand_size(P0);
    crate::r_s05_common::enter(&mut t, P0, "Duskana, the Rage Mother");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // "Whenever a creature you control with base power and toughness 2/2 attacks, it gets
    // +3/+3 until end of turn."
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (howler, Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(howler), (2, 2));
}
