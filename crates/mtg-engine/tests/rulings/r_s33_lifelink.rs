//! Rulings batch S33 — multiple instances of lifelink (and deathtouch) on one creature
//! are redundant (CR 702.15f, 702.2f): its controller gains life equal to the damage
//! dealt once, in one life-gain event.

use crate::r_s01_common::supported;
use crate::r_s06_common::attach_new;
use crate::r_s22_common::attack_p1_unblocked;
use crate::r_s33_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn unflinching_courage_on_a_lifelink_creature_gains_the_damage_once() {
    cr!("702.15f", "702.15b", "510.2");
    ruling!(
        "Unflinching Courage",
        "Multiple instances of lifelink on the same creature are redundant."
    );
    supported("Unflinching Courage");
    // "Enchanted creature gets +2/+2 and has trample and lifelink." on Vampire Nighthawk
    // (2/3 flying, deathtouch, lifelink): a 4/5 with two instances of lifelink.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    attach_new(&mut t, P0, "Unflinching Courage", hawk);
    assert_eq!(t.pt(hawk), (4, 5));
    let from = t.g.turn_events.len();
    attack_p1_unblocked(&mut t, hawk);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    assert_eq!(life_gains_since(&t, from, P0), vec![4]);
}

#[test]
fn vault_of_the_archangel_on_a_deathtouch_lifelink_creature_is_redundant() {
    cr!("702.15f", "702.2f", "702.2b", "510.2");
    ruling!(
        "Vault of the Archangel",
        "Multiple instances of deathtouch or lifelink on the same creature are redundant."
    );
    supported("Vault of the Archangel");
    // "{2}{W}{B}, {T}: Creatures you control gain deathtouch and lifelink until end of
    // turn." Vampire Nighthawk already has both; it's blocked by Giant Spider (2/4, reach).
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let vault = t.battlefield(P0, "Vault of the Archangel");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, vault, 1, &[]).unwrap();
    t.resolve_all();
    let spider = t.battlefield(P1, "Giant Spider");
    let from = t.g.turn_events.len();
    crate::r_s01_common::attack_with(&mut t, &[(hawk, Entity::Player(P1))]);
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(spider, hawk)]);
    t.resolve_all();
    // Deathtouch destroys the Spider (once); P0 gains 2 life, once.
    assert!(t.in_graveyard(P1, "Giant Spider"));
    assert_eq!(t.life(P0), 22);
    assert_eq!(life_gains_since(&t, from, P0), vec![2]);
}
