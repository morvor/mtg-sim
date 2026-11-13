//! Rulings batch S35 — kindred (CR 308): a card type that only appears with another card
//! type, and lets noncreature cards have creature types.

use crate::r_s01_common::supported;
use crate::r_s04_common::spell_targets;
use crate::r_s06_common::activate_containing;
use crate::r_s24_common::pool;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn kindred_counts_as_a_card_type_among_cards_in_a_graveyard() {
    cr!("205.2a", "308.1");
    ruling!(
        "Bitterblossom",
        "While it appears only on cards that already have other card types, kindred is a card type and will be counted by effects that refer to the number of card types among cards in a zone."
    );
    supported("Tarmogoyf");
    supported("Bitterblossom");
    // Tarmogoyf: "power is equal to the number of card types among cards in all
    // graveyards and its toughness is equal to that number plus 1."
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P0, "Tarmogoyf");
    assert_eq!(t.pt(goyf), (0, 1));
    // Bitterblossom, a Kindred Enchantment — Faerie: two card types.
    t.graveyard(P1, "Bitterblossom");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (2, 3));
    // Another enchantment adds nothing new; a creature card does.
    t.graveyard(P1, "Pacifism");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (2, 3));
    t.graveyard(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (3, 4));
}

#[test]
fn a_kindred_card_has_creature_types_but_isnt_a_creature() {
    cr!("308.2", "205.3m");
    ruling!(
        "Morcant's Eyes",
        "Kindred is a card type that allows noncreature cards to have creature types. For example, Morcant's Eyes is an Elf (although not a creature) while on the battlefield and an Elf card (although not a creature card) in zones other than the battlefield."
    );
    supported("Morcant's Eyes");
    supported("Elvish Archdruid");
    supported("Raise Dead");
    // On the battlefield, it's an Elf: Elvish Archdruid ("{T}: Add {G} for each Elf you
    // control.") counts it. It isn't a creature.
    let mut t = TestGame::new(2);
    let eyes = t.battlefield(P0, "Morcant's Eyes");
    assert!(t.obj_now(eyes).chars.has_subtype("Elf"));
    assert!(!t.obj_now(eyes).chars.card_types.contains(CardType::Creature));
    let druid = t.battlefield(P0, "Elvish Archdruid");
    activate_containing(&mut t, P0, druid, "Add").expect("Archdruid's mana ability");
    assert_eq!(pool(&t, P0, ManaType::G), 2);
    // In the graveyard, it's an Elf card but not a creature card. Morcant's Eyes:
    // "{4}{G}{G}, Sacrifice this enchantment: Create X 2/2 black and green Elf creature
    // tokens, where X is the number of Elf cards in your graveyard." Another Morcant's
    // Eyes in the graveyard (and this one, sacrificed to pay the cost) are Elf cards.
    let mut t = TestGame::new(2);
    let in_yard = t.graveyard(P0, "Morcant's Eyes");
    t.graveyard(P0, "Grizzly Bears");
    let eyes = t.battlefield(P0, "Morcant's Eyes");
    t.lands(P0, "Forest", 6);
    activate_containing(&mut t, P0, eyes, "Create").expect("Morcant's Eyes");
    t.resolve_all();
    let elves = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.has_subtype("Elf"))
        .count();
    assert_eq!(elves, 2);
    // Raise Dead ("Return target creature card from your graveyard to your hand.") can't
    // target it.
    let targets = spell_targets(&mut t, P0, "Raise Dead");
    assert!(!targets.contains(&Entity::Object(in_yard)));
    assert!(targets.iter().any(|e| t.obj_now(e.object().unwrap()).chars.name == "Grizzly Bears"));
}
