//! Rulings batch P092 — Champion of the Path's "Whenever another Elemental you control
//! enters, it deals damage equal to its power to each opponent." (The permanent is put
//! onto the battlefield directly, without paying its behold cost.)

use crate::r_s01_common::custom_card;
use crate::r_s02_common::destroy;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The whole card compiles, including the Elemental trigger the rulings below concern.
fn the_card_compiles() {
    let c = card("Champion of the Path");
    let u = c.unsupported_text();
    assert!(u.is_empty(), "{u:?}");
}

#[test]
fn champion_of_the_path_triggers_for_a_noncreature_elemental_that_deals_nothing() {
    cr!("603.2", "308.1", "120.2");
    ruling!(
        "Champion of the Path",
        "If an Elemental you control that isn't a creature enters (probably because it's a kindred permanent with the Elemental subtype), Champion of the Path's second ability will still trigger. If that Elemental still isn't a creature when the ability resolves, the ability won't deal any damage."
    );
    the_card_compiles();
    let mut t = TestGame::new(2);
    let champion = t.battlefield(P0, "Champion of the Path");
    let def = custom_card("Elemental Totem", "Kindred Enchantment — Elemental", "{2}", None, "");
    let totem = t.custom(P0, def, Zone::Hand(P0));
    crate::r_s05_common::move_to(&mut t, totem, Zone::Battlefield);
    t.settle();
    assert_eq!(crate::r_s11_common::triggered_from(&t, champion), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn champion_of_the_path_uses_the_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Champion of the Path",
        "If the Elemental that caused Champion of the Path's second ability to trigger is no longer on the battlefield when that ability resolves, use that Elemental's power as it last existed on the battlefield to determine how much damage is dealt."
    );
    the_card_compiles();
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Champion of the Path");
    // Air Elemental is a 4/4 Elemental.
    let air = t.enter(P0, "Air Elemental");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, air);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}
