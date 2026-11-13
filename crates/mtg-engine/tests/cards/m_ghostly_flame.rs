//! Ghostly Flame (hand-written, `src/cards/ghostly_flame.rs`): black and/or red
//! permanents and spells are colorless sources of damage.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn protection_from_red_doesnt_prevent_damage_from_a_red_spell() {
    cr!("702.16e");
    ruling!("Ghostly Flame", "The spells just act like colorless sources when dealing damage");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ghostly Flame");
    let walker = t.battlefield(P1, "Paladin en-Vec");
    t.lands(P0, "Mountain", 2);
    let p = t.hand(P0, "Pyroclasm");
    t.cast(P0, p).go();
    t.resolve();
    assert!(!t.on_battlefield(walker));
}

#[test]
fn without_it_protection_prevents_the_damage() {
    cr!("702.16e");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P1, "Paladin en-Vec");
    t.lands(P0, "Mountain", 2);
    let p = t.hand(P0, "Pyroclasm");
    t.cast(P0, p).go();
    t.resolve();
    assert!(t.on_battlefield(walker));
}

#[test]
fn red_spells_keep_their_color_for_targeting() {
    cr!("702.16b");
    ruling!("Ghostly Flame", "A red spell can’t target a creature with Protection from Red");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ghostly Flame");
    let walker = t.battlefield(P1, "Paladin en-Vec");
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    // The creature isn't a legal target: the spell can't be cast targeting it.
    let _ = t.cast(P0, b).target(walker).try_go();
    t.resolve_all();
    assert_eq!(t.obj_now(walker).damage, 0);
    assert!(t.on_battlefield(walker));
}
