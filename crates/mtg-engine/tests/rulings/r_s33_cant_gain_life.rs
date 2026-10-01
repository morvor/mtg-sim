//! Rulings batch S33 — "players can't gain life" (CR 119.7): spells and abilities that
//! would cause a player to gain life still resolve, but the life-gain part has no
//! effect; likewise, while a player's life total can't change, the life-gain and
//! life-loss parts do nothing (CR 119.7, 119.8).

use crate::r_s01_common::supported;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// With `name` on P1's battlefield, P0 casts Lightning Helix ("Lightning Helix deals 3
/// damage to any target and you gain 3 life.") at P1: the damage is dealt, the spell
/// resolves, and P0 gains no life.
fn helix_under(name: &str) {
    supported(name);
    supported("Lightning Helix");
    let mut t = TestGame::new(2);
    t.battlefield(P1, name);
    set_life(&mut t, P0, 10);
    let from = t.g.turn_events.len();
    cast_and_resolve(&mut t, P0, "Lightning Helix", &[Entity::Player(P1)]);
    assert!(t.in_graveyard(P0, "Lightning Helix"));
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 10);
    assert!(life_gains_since(&t, from, P0).is_empty());
    // Without it, P0 gains 3.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    cast_and_resolve(&mut t, P0, "Lightning Helix", &[Entity::Player(P1)]);
    assert_eq!((t.life(P0), t.life(P1)), (13, 17));
}

#[test]
fn leyline_of_punishment_spells_that_gain_life_still_resolve() {
    cr!("119.7", "609.3", "101.3");
    ruling!(
        "Leyline of Punishment",
        "Spells and abilities that would normally cause a player to gain life still resolve, but the life-gain part simply has no effect."
    );
    helix_under("Leyline of Punishment");
}

#[test]
fn everlasting_torment_spells_that_gain_life_still_resolve() {
    cr!("119.7", "609.3", "101.3");
    ruling!(
        "Everlasting Torment",
        "Spells and abilities that would normally cause a player to gain life still resolve, but the life-gain part simply has no effect."
    );
    helix_under("Everlasting Torment");
}

#[test]
fn havoc_festival_spells_that_gain_life_still_resolve() {
    cr!("119.7", "609.3", "101.3");
    ruling!(
        "Havoc Festival",
        "Spells and abilities that would cause a player to gain life still resolve, but the life-gain part has no effect."
    );
    helix_under("Havoc Festival");
}
