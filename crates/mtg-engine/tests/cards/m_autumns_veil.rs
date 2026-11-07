//! Autumn's Veil (hand-written, `src/cards/autumns_veil.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn spells_you_control_cant_be_countered_by_blue_spells() {
    cr!("701.6a");
    ruling!("Autumn's Veil", "the part of their effect that would counter the spell you control simply won’t do anything");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let veil = t.hand(P0, "Autumn's Veil");
    t.cast(P0, veil).go();
    t.resolve();
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn spells_of_other_colors_can_still_counter_them() {
    cr!("701.6a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let veil = t.hand(P0, "Autumn's Veil");
    t.cast(P0, veil).go();
    t.resolve();
    // A white counterspell isn't blue or black.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let spell = t.cast(P0, bolt).target(P1).go();
    t.lands(P1, "Plains", 1);
    let tithe = t.hand(P1, "Mana Tithe");
    t.cast(P1, tithe).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn creatures_you_control_cant_be_targeted_by_black_spells() {
    cr!("115.4");
    ruling!("Autumn's Veil", "no new blue or black spell may be cast that turn targeting a creature you control");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let veil = t.hand(P0, "Autumn's Veil");
    t.cast(P0, veil).go();
    t.resolve();
    t.lands(P1, "Swamp", 2);
    let blade = t.hand(P1, "Doom Blade");
    assert!(t.cast(P1, blade).target(bear).try_go().is_err());
}
