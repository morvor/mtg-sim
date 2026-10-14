//! Rulings batch S32 — Pyrogoyf ("Pyrogoyf's power is equal to the number of card types
//! among cards in all graveyards and its toughness is equal to that number plus 1."):
//! card types (CR 205.2a) count; supertypes (CR 205.4) and subtypes (CR 205.3) don't.

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn only_card_types_count_not_supertypes_or_subtypes() {
    cr!("205.2a", "205.3a", "205.4a", "604.3", "712.8a");
    ruling!(
        "Pyrogoyf",
        "Card types that can appear on cards in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Lhurgoyf, Forest, and Siege are subtypes, not card types."
    );
    supported("Pyrogoyf");
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P0, "Pyrogoyf");
    assert_eq!(t.pt(goyf), (0, 1));
    // (card, count after it's in a graveyard)
    for (owner, name, n) in [
        // Basic snow land — Forest: one card type.
        (P0, "Snow-Covered Forest", 1),
        // Another land changes nothing.
        (P1, "Forest", 1),
        // Legendary creature — Dog.
        (P1, "Isamaru, Hound of Konda", 2),
        // A Lhurgoyf creature card adds no type of its own.
        (P0, "Polygoyf", 2),
        // Kindred instant — Shapeshifter: two card types.
        (P0, "Nameless Inversion", 4),
        // Kindred enchantment — Faerie: one more.
        (P1, "Bitterblossom", 5),
        // Legendary planeswalker — Jace.
        (P0, "Jace Beleren", 6),
        // Artifact creature.
        (P1, "Ornithopter", 7),
        (P0, "Mind Rot", 8),
        // A battle — Siege (its front face's characteristics in the graveyard).
        (P1, "Invasion of Kaladesh // Aetherwing, Golden-Scale Flagship", 9),
    ] {
        t.graveyard(owner, name);
        t.g.recompute();
        assert_eq!(t.pt(goyf), (n, n + 1), "after {name}");
    }
}
