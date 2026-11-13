//! Rulings batch P057 — Planar Nexus: "This land is every nonbasic land type." gives it
//! each nonbasic land type listed in CR 205.3i (not the reminder text's older list), so it
//! is an Urza's Mine, Power-Plant and Tower for the Urza lands' conditions.

use crate::r_s01_common::supported;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn has_subtype(t: &mut TestGame, id: ObjectId, st: &str) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.subtypes.iter().any(|s| s == st)
}

#[test]
fn planar_nexus_completes_the_urza_lands() {
    cr!("205.3i");
    ruling!(
        "Planar Nexus",
        "Since Planar Nexus has the subtypes Urza's, Mine, Power-Plant, and Tower, it will satisfy all of the requirements for the abilities of Urza's Mine, Urza's Power Plant, and Urza's Tower to add additional mana. For example, if you control Planar Nexus and Urza's Tower, the ability of Urza's Tower will add {C}{C}{C}."
    );
    supported("Planar Nexus");
    supported("Urza's Tower");
    supported("Urza's Mine");
    supported("Urza's Power Plant");
    for (name, n) in [("Urza's Tower", 3), ("Urza's Mine", 2), ("Urza's Power Plant", 2)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Planar Nexus");
        let land = t.battlefield(P0, name);
        assert!(tap_for_mana(&mut t, P0, land, "Add"), "{name}");
        assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), n, "{name}");
    }
    // Without the Nexus, the Tower adds {C}.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Urza's Tower");
    assert!(tap_for_mana(&mut t, P0, land, "Add"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::C), 1);
}

#[test]
fn planar_nexus_has_every_nonbasic_land_type_in_the_comprehensive_rules() {
    cr!("205.3i");
    ruling!(
        "Planar Nexus",
        "The full list of land types can always be found in the Comprehensive Rules of Magic."
    );
    supported("Planar Nexus");
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P0, "Planar Nexus");
    // Beyond the reminder text's list: Planet and Town are land types now too.
    for st in [
        "Cave", "Desert", "Gate", "Lair", "Locus", "Mine", "Power-Plant", "Sphere", "Tower",
        "Urza's", "Planet", "Town",
    ] {
        assert!(has_subtype(&mut t, nexus, st), "{st}");
    }
    // Not the basic land types.
    for st in ["Forest", "Island", "Mountain", "Plains", "Swamp"] {
        assert!(!has_subtype(&mut t, nexus, st), "{st}");
    }
}
