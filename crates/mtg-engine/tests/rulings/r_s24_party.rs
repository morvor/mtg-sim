//! Rulings batch S24 — the number of creatures in your party (CR 700.8): one check each
//! for a Cleric, a Rogue, a Warrior and a Wizard you control, each creature counted for
//! one check only, in the way that gives the highest number.

use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::can_cast;
use crate::r_s05_common::enter;
use mtg_engine::game_terms::party_size;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_cleric_wizard_counts_as_the_wizard_when_you_also_control_a_cleric() {
    cr!("700.8", "700.8b");
    ruling!(
        "Archpriest of Iona",
        "If a creature has more than one party creature type, and there are multiple ways to count that creature that could result in a different number of creatures in your party, the highest such number is used. For example, if you control a Cleric and a Cleric Wizard, the number of creatures in your party is two."
    );
    supported("Archpriest of Iona");
    supported("Shieldmage Elder");
    // "Archpriest of Iona's power is equal to the number of creatures in your party." It's
    // a Human Cleric; Shieldmage Elder is a Human Cleric Wizard.
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Archpriest of Iona");
    assert_eq!(t.pt(priest).0, 1);
    t.battlefield(P0, "Shieldmage Elder");
    t.g.recompute();
    assert_eq!(t.pt(priest).0, 2);
}

#[test]
fn a_cleric_wizard_counts_as_the_wizard_when_you_also_control_a_cleric_curly() {
    cr!("700.8", "700.8b");
    ruling!(
        "Practiced Tactics",
        "If a creature has more than one party creature type, and there are multiple ways to count that creature that could result in a different number of creatures in your party, the highest such number is used. For example, if you control a Cleric and a Cleric Wizard, the number of creatures in your party is two. You can’t choose to have it be just one by counting the Cleric Wizard first as a Cleric."
    );
    supported("Practiced Tactics");
    // "Choose target attacking or blocking creature. Practiced Tactics deals damage to
    // that creature equal to twice the number of creatures in your party."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Keepers of the Faith");
    t.battlefield(P1, "Shieldmage Elder");
    t.battlefield(P1, "Plains");
    // Colossal Dreadmaw: 6/6 trample.
    let dreadmaw = t.battlefield(P0, "Colossal Dreadmaw");
    attack_with(&mut t, &[(dreadmaw, Entity::Player(P1))]);
    let tactics = t.hand(P1, "Practiced Tactics");
    t.cast(P1, tactics).target(dreadmaw).go();
    t.resolve_all();
    // Party of two: 4 damage, not 2.
    assert_eq!(t.obj_now(dreadmaw).damage, 4);
}

#[test]
fn each_creature_is_counted_for_only_one_party_check() {
    cr!("700.8", "700.8a");
    ruling!(
        "Malakir Blood-Priest",
        "To determine “the number of creatures in your party,” check whether you control a Cleric, whether you control a Rogue, whether you control a Warrior, and whether you control a Wizard. The number is the total number of those checks to which you answered yes. Each creature you control can be counted for only one of those checks."
    );
    supported("Malakir Blood-Priest");
    // "When this creature enters, each opponent loses X life and you gain X life, where X
    // is the number of creatures in your party." Two Clerics answer one check.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keepers of the Faith");
    enter(&mut t, P0, "Malakir Blood-Priest");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
    // A Rogue and a Warrior answer two more; a creature with no party type answers none.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keepers of the Faith");
    t.battlefield(P0, "Bane Alley Blackguard");
    t.battlefield(P0, "Oreskos Swiftclaw");
    t.battlefield(P0, "Grizzly Bears");
    enter(&mut t, P0, "Malakir Blood-Priest");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn each_creature_is_counted_for_only_one_party_check_straight() {
    cr!("700.8", "700.8a", "601.2f");
    ruling!(
        "Deadly Alliance",
        "To determine \"the number of creatures in your party,\" check whether you control a Cleric, whether you control a Rogue, whether you control a Warrior, and whether you control a Wizard. The number is the total number of those checks to which you answered yes. Each creature you control can be counted for only one of those checks."
    );
    supported("Deadly Alliance");
    // "This spell costs {1} less to cast for each creature in your party." ({4}{B})
    // A changeling is every party type, but it's one creature: it answers one check.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Changeling Outcast");
    t.battlefield(P0, "Keepers of the Faith");
    t.battlefield(P0, "Archpriest of Iona");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let alliance = t.hand(P0, "Deadly Alliance");
    assert_eq!(party_size(&t.g, P0), 2);
    // {4}{B} minus {2}: three lands are needed, two aren't enough.
    assert!(!can_cast(&mut t, P0, alliance, CastMethod::Normal));
    t.lands(P0, "Swamp", 1);
    assert!(can_cast(&mut t, P0, alliance, CastMethod::Normal));
}
