//! Rulings batch S31 — Humility and Opalescence: Opalescence's type-changing effect
//! applies in layer 4 and its P/T-setting effect in layer 7b; Humility's ability-removing
//! effect applies in layer 6 and its P/T-setting effect in layer 7b. Each effect keeps
//! applying in its later layers even after its source has lost its abilities (CR 613.6),
//! and within layer 7b they apply in timestamp order (CR 613.7).

use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Puts the enchantments onto P0's battlefield in the given order, plus a Grizzly Bears;
/// returns (Humility, Worship, Opalescence, Grizzly Bears).
fn enchantments(humility_first: bool) -> (TestGame, [ObjectId; 4]) {
    supported("Humility");
    supported("Opalescence");
    supported("Worship");
    let mut t = TestGame::new(2);
    let (humility, opalescence) = if humility_first {
        let h = t.battlefield(P0, "Humility");
        (h, t.battlefield(P0, "Opalescence"))
    } else {
        let o = t.battlefield(P0, "Opalescence");
        (t.battlefield(P0, "Humility"), o)
    };
    let worship = t.battlefield(P0, "Worship");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    (t, [humility, worship, opalescence, bears])
}

/// Whether the permanent is a creature that's still an enchantment, with no abilities.
fn is_creature_enchantment_without_abilities(t: &TestGame, id: ObjectId) -> bool {
    let o = t.obj_now(id);
    o.chars.is(CardType::Creature)
        && o.chars.is(CardType::Enchantment)
        && o.chars.abilities.is_empty()
}

#[test]
fn opalescence_before_humility_makes_them_1_1() {
    cr!("613.1d", "613.1f", "613.4b", "613.6", "613.7");
    ruling!(
        "Humility",
        "So if Opalescence, Humility, and Worship are on the battlefield and Opalescence entered before Humility, the following is true"
    );
    ruling!(
        "Opalescence",
        "So if Opalescence, Humility, and Worship are on the battlefield and Opalescence entered before Humility, the following is true"
    );
    let (t, [humility, worship, opalescence, bears]) = enchantments(false);
    // Layer 4: Humility and Worship are creatures that are still enchantments
    // (Opalescence isn't: "each other"). Layer 6: they lose their abilities (Humility,
    // which keeps applying after losing its own). Layer 7b: 4/4 (Opalescence), then 1/1
    // (Humility, the later timestamp).
    assert!(is_creature_enchantment_without_abilities(&t, humility));
    assert!(is_creature_enchantment_without_abilities(&t, worship));
    assert!(!t.obj_now(opalescence).chars.is(CardType::Creature));
    assert_eq!(t.pt(humility), (1, 1));
    assert_eq!(t.pt(worship), (1, 1));
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn humility_before_opalescence_makes_them_4_4() {
    cr!("613.1d", "613.1f", "613.4b", "613.6", "613.7");
    ruling!(
        "Humility",
        "But if Humility entered before Opalescence, the following is true"
    );
    let (t, [humility, worship, opalescence, bears]) = enchantments(true);
    // Layer 7b: 1/1 (Humility), then each becomes 4/4 (Opalescence, the later timestamp:
    // its mana value).
    assert!(is_creature_enchantment_without_abilities(&t, humility));
    assert!(is_creature_enchantment_without_abilities(&t, worship));
    assert!(!t.obj_now(opalescence).chars.is(CardType::Creature));
    assert_eq!(t.pt(humility), (4, 4));
    assert_eq!(t.pt(worship), (4, 4));
    // Grizzly Bears isn't an enchantment: only Humility sets it.
    assert_eq!(t.pt(bears), (1, 1));
}
