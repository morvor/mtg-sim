//! Rulings batch S30 — excess damage (CR 120.10): "the amount of excess damage dealt this
//! way" (Windswift Slice), with deathtouch making any damage beyond 1 excess.

use crate::r_s01_common::{supported, with_subtype};
use crate::r_s25_common::{cast_new, creature_tokens};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's `biter` deals damage equal to its power to P1's Hill Giant (3/3) with Windswift
/// Slice ("Target creature you control deals damage equal to its power to target creature
/// you don't control. Create a number of 1/1 green Elf Warrior creature tokens equal to
/// the amount of excess damage dealt this way."). Returns the number of tokens created
/// and whether the Giant died.
fn slice(biter: &str) -> (usize, bool) {
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, biter);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(
        &mut t,
        P0,
        "Windswift Slice",
        &[Entity::Object(mine), Entity::Object(giant)],
    );
    t.resolve_all();
    (creature_tokens(&t, P0), t.in_graveyard(P1, "Hill Giant"))
}

#[test]
fn windswift_slice_creates_a_token_for_each_excess_damage() {
    cr!("120.10");
    supported("Windswift Slice");
    // Colossal Dreadmaw (6/6) deals 6 to a 3/3: 3 excess.
    assert_eq!(slice("Colossal Dreadmaw"), (3, true));
    // Grizzly Bears (2/2) deals 2: none.
    assert_eq!(slice("Grizzly Bears"), (0, false));
}

#[test]
fn with_deathtouch_any_damage_beyond_1_is_excess() {
    cr!("120.10");
    ruling!(
        "Windswift Slice",
        "Even 1 damage dealt to a creature from a source with deathtouch is considered lethal damage, so any amount greater than that will cause excess damage to be dealt, even if the total amount of damage isn't greater than the creature's toughness."
    );
    supported("Windswift Slice");
    supported("Gifted Aetherborn");
    // Gifted Aetherborn (2/3 deathtouch) deals 2 to a 3/3: 1 is lethal, 1 is excess.
    assert_eq!(slice("Gifted Aetherborn"), (1, true));
}

#[test]
fn a_planeswalker_is_dealt_excess_damage_beyond_its_current_loyalty() {
    cr!("120.10", "120.3c");
    ruling!(
        "Aegar, the Freezing Flame",
        "A planeswalker is dealt excess damage if it's dealt damage greater than its current loyalty."
    );
    supported("Bottle-Cap Blast");
    // Jace Beleren (printed loyalty 3) is dealt 2 damage by Shock: its loyalty is 1.
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    cast_new(&mut t, P0, "Shock", &[Entity::Object(jace)]);
    t.resolve_all();
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
    // "Bottle-Cap Blast deals 5 damage to any target. If excess damage was dealt to a
    // permanent this way, create that many tapped Treasure tokens." 4 excess, not 2.
    cast_new(&mut t, P0, "Bottle-Cap Blast", &[Entity::Object(jace)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 4);
}
