//! CR 308: kindred cards — the other card type rules casting and resolving, their
//! subtypes are creature types, and the old "tribal" type.

use crate::r300_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn kindred_subtypes_are_creature_types() {
    cr!("308.2");
    // "Kindred Enchantment — Treefolk Aura": a creature type and an enchantment type.
    assert_eq!(subtypes_of("Lignify"), vec!["Treefolk", "Aura"]);
    assert!(is_creature_type("Treefolk"));
    assert_eq!(subtypes_of("Boggart Shenanigans"), vec!["Goblin"]);
    // Boggart Shenanigans is a Goblin spell: Goblin Warchief ("Goblin spells you cast
    // cost {1} less to cast") makes its {2}{R} cost {1}{R}.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Boggart Shenanigans");
    assert!(!can_cast(&mut t, P0, c));
    t.battlefield(P0, "Goblin Warchief");
    assert!(can_cast(&mut t, P0, c));
    t.cast(P0, c).go();
    t.resolve();
    // On the battlefield it's a Goblin that isn't a creature.
    let bs = t.named_on_battlefield("Boggart Shenanigans")[0];
    assert!(t.obj(bs).chars.has_subtype("Goblin") && !t.obj(bs).is_creature());
    // A Lignify card in hand is a Treefolk card.
    let lig = t.hand(P0, "Lignify");
    assert!(t.obj(lig).chars.has_subtype("Treefolk"));
    assert_eq!(t.zone(lig), Zone::Hand(P0));
}

#[test]
fn tribal_cards_are_kindred_cards() {
    cr!("308.3");
    ruling!(
        "Crib Swap",
        "This card was originally printed with the \"tribal\" card type. That card type has been replaced with \"kindred\""
    );
    // Crib Swap's Oracle type line says Kindred.
    assert!(types_of("Crib Swap").contains(CardType::Kindred));
    // An old "Tribal" type line is read as Kindred.
    let tl = TypeLine::parse("Tribal Instant — Shapeshifter");
    assert!(tl.card_types.contains(CardType::Kindred));
    assert!(tl.card_types.contains(CardType::Instant));
    assert_eq!(CardType::from_word("tribal"), Some(CardType::Kindred));
}
