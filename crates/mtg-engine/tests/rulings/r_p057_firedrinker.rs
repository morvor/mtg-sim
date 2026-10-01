//! Rulings batch P057 — Firedrinker Satyr: "Whenever this creature is dealt damage, it
//! deals that much damage to you." triggers even on lethal damage (CR 603.10a, 704.5g) and
//! deals noncombat damage (CR 510.2, 120.2); "{1}{R}: This creature gets +1/+0 until end of
//! turn and deals 1 damage to you."

use crate::r_p057_common::pump;
use crate::r_s01_common::supported;
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn firedrinker_satyr_blocking_a_7_7_deals_7_noncombat_damage_to_you() {
    cr!("603.10a", "510.2", "120.2");
    ruling!(
        "Firedrinker Satyr",
        "Firedrinker Satyr's first ability will trigger even if it's dealt lethal damage. For example, if it blocks a 7/7 creature, its ability will trigger and Firedrinker Satyr will deal 7 damage to you."
    );
    ruling!(
        "Firedrinker Satyr",
        "Damage dealt by Firedrinker Satyr due to its first ability isn't combat damage, even if combat damage caused the ability to trigger."
    );
    supported("Firedrinker Satyr");
    let mut t = TestGame::new(2);
    let satyr = t.battlefield(P0, "Firedrinker Satyr"); // 2/1
    let big = t.battlefield(P1, "Rumbling Baloth");
    pump(&mut t, big, 3, 3);
    assert_eq!(t.pt(big), (7, 7));
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(big, Entity::Player(P0))], &[(satyr, big)]);
    assert!(!t.on_battlefield(satyr), "lethal damage");
    assert_eq!(t.life(P0), 13);
    // It wasn't combat damage dealt to P0.
    assert!(t
        .g
        .history
        .combat_damage_to_players
        .iter()
        .all(|r| r.player != P0));
}

#[test]
fn firedrinker_satyr_pump_deals_1_damage_to_you() {
    cr!("602.2", "120.3a");
    supported("Firedrinker Satyr");
    let mut t = TestGame::new(2);
    let satyr = t.battlefield(P0, "Firedrinker Satyr");
    add_mana(&mut t, P0, ManaType::R, 2);
    activate_containing(&mut t, P0, satyr, "+1/+0").expect("activated");
    t.resolve_all();
    assert_eq!(t.pt(satyr), (3, 1));
    assert_eq!(t.life(P0), 19);
}
