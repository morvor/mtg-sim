//! Rulings batch P102 — mass land denial: "Nonbasic lands are Mountains/Islands" (Blood
//! Moon, Magus of the Moon, Harbinger of the Seas, Zhao; CR 305.7, 613), untap
//! restrictions (Winter Orb, Winter Moon, Static Orb, Hokori, Dovin Baan's emblem,
//! Winter's Night; CR 502.3), Quicksilver Fountain and Thieves' Auction.

use crate::r_p102_common::*;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const MOONS: [(&str, &str); 3] = [
    ("Blood Moon", "Mountain"),
    ("Magus of the Moon", "Mountain"),
    ("Harbinger of the Seas", "Island"),
];

#[test]
fn moons_dont_change_names_or_supertypes() {
    cr!("305.7", "205.4");
    ruling!(
        "Blood Moon",
        "This effect doesn't affect names or supertypes. It won't turn any land into a basic land"
    );
    ruling!(
        "Magus of the Moon",
        "Magus of the Moon doesn't affect names or supertypes. It won't turn any land into a basic land"
    );
    ruling!(
        "Harbinger of the Seas",
        "Harbinger of the Seas doesn't affect names or supertypes. It won't turn any land into a basic land"
    );
    for (moon, ty) in MOONS {
        supported(moon);
        let mut t = TestGame::new(2);
        t.battlefield(P1, moon);
        let cradle = t.battlefield(P0, "Gaea's Cradle");
        let o = t.obj_now(cradle);
        assert!(o.chars.has_subtype(ty), "{moon}");
        assert_eq!(o.chars.name, "Gaea's Cradle", "{moon}");
        assert!(o.chars.supertypes.contains(Supertype::Legendary), "{moon}");
        assert!(!o.chars.supertypes.contains(Supertype::Basic), "{moon}");
    }
}

#[test]
fn moons_replace_land_types_and_abilities() {
    cr!("305.7", "613.1d", "613.1f");
    ruling!(
        "Magus of the Moon",
        "Nonbasic lands lose any other land types and abilities they had. They gain the land type Mountain and gain the ability \"{T}: Add {R}.\""
    );
    ruling!(
        "Harbinger of the Seas",
        "Nonbasic lands lose any other land types and abilities they had. They gain the land type Island and gain the ability \"{T}: Add {U}.\""
    );
    for (moon, ty, mana) in [
        ("Magus of the Moon", "Mountain", ManaType::R),
        ("Harbinger of the Seas", "Island", ManaType::U),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, moon);
        let pool = t.battlefield(P0, "Breeding Pool");
        let o = t.obj_now(pool);
        assert_eq!(o.chars.subtypes.len(), 1, "{moon}");
        assert!(o.chars.has_subtype(ty), "{moon}");
        assert_eq!(o.chars.abilities.len(), 1, "{moon}");
        assert!(tap_for_mana(&mut t, P0, pool, ""));
        assert_eq!(t.g.player(P0).mana_pool.count(mana), 1, "{moon}");
        // Basic lands are unaffected.
        let forest = t.battlefield(P0, "Forest");
        assert!(t.obj_now(forest).chars.has_subtype("Forest"));
    }
}

#[test]
fn moons_keep_working_after_losing_their_abilities() {
    cr!("613.1d", "613.1f", "613.6");
    ruling!(
        "Harbinger of the Seas",
        "If Harbinger of the Seas loses its abilities, it continues to turn nonbasic lands into Islands."
    );
    ruling!(
        "Magus of the Moon",
        "If Magus of the Moon loses its abilities, it continues to turn nonbasic lands into Mountains."
    );
    for (moon, ty) in [
        ("Harbinger of the Seas", "Island"),
        ("Magus of the Moon", "Mountain"),
    ] {
        let mut t = TestGame::new(2);
        let m = t.battlefield(P1, moon);
        let cradle = t.battlefield(P0, "Gaea's Cradle");
        lots(&mut t, P0);
        let frog = t.hand(P0, "Turn to Frog");
        t.cast(P0, frog).target(Entity::Object(m)).go();
        t.resolve_all();
        assert_eq!(t.pt(m), (1, 1), "{moon}: it's a Frog without abilities");
        assert!(t.obj_now(m).chars.abilities.is_empty(), "{moon}");
        assert!(t.obj_now(cradle).chars.has_subtype(ty), "{moon}");
        // Even a later "loses all abilities" (Dress Down) doesn't stop it.
        let mut t = TestGame::new(2);
        let m = t.battlefield(P1, moon);
        let cradle = t.battlefield(P0, "Gaea's Cradle");
        t.battlefield(P0, "Dress Down");
        assert!(t.obj_now(cradle).chars.has_subtype(ty), "{moon}");
        assert!(t.obj_now(m).chars.abilities.is_empty(), "{moon}");
    }
}

#[test]
fn blood_moon_stops_enters_tapped_and_as_enters_abilities() {
    cr!("305.7", "614.12", "613.1f");
    ruling!(
        "Blood Moon",
        "If a nonbasic land has an ability that causes it to enter the battlefield tapped, it will lose that ability before it can apply."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Blood Moon");
    let gate = t.hand(P0, "Selesnya Guildgate");
    t.play_land(P0, gate).unwrap();
    t.settle();
    assert!(t.on_battlefield(gate));
    assert!(!t.obj_now(gate).tapped);
    // Cavern of Souls: no creature type is chosen as it enters.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Blood Moon");
    let cavern = t.hand(P0, "Cavern of Souls");
    let from = t.asked().len();
    t.play_land(P0, cavern).unwrap();
    t.settle();
    assert!(t.on_battlefield(cavern));
    assert!(!t.asked()[from..].iter().any(|(p, _)| *p == P0));
}

#[test]
fn zhao_makes_conditional_taplands_enter_tapped_anyway() {
    cr!("614.1c", "614.12");
    ruling!(
        "Zhao, the Moon Slayer",
        "If an effect states that a land enters tapped unless a condition is met, Zhao, the Moon Slayer's second ability has it enter tapped even if that condition is true."
    );
    supported("Zhao, the Moon Slayer");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Zhao, the Moon Slayer");
    t.battlefield(P0, "Island");
    let fort = t.hand(P0, "Glacial Fortress");
    t.play_land(P0, fort).unwrap();
    t.settle();
    assert!(t.obj_now(fort).tapped);
    // Basic lands don't.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Zhao, the Moon Slayer");
    let isl = t.hand(P0, "Island");
    t.play_land(P0, isl).unwrap();
    assert!(!t.obj_now(isl).tapped);
}

#[test]
fn zhao_with_a_conqueror_counter_stops_land_etb_triggers() {
    cr!("305.7", "603.6a", "613.1f");
    ruling!(
        "Zhao, the Moon Slayer",
        "While Zhao has a conqueror counter on him, if a nonbasic land has an ability that triggers \"when\" it enters, it will lose that ability before it can trigger."
    );
    let mut t = TestGame::new(2);
    let zhao = t.battlefield(P1, "Zhao, the Moon Slayer");
    let garden = t.hand(P0, "Khalni Garden");
    // Without the counter, Khalni Garden's trigger works (and it enters tapped).
    t.play_land(P0, garden).unwrap();
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Plant").len(), 1);
    assert!(t.obj_now(garden).tapped);
    let mut t = TestGame::new(2);
    let zhao2 = t.battlefield(P1, "Zhao, the Moon Slayer");
    crate::r_s29_common::put_counters(&mut t, zhao2, "conqueror", 1);
    let garden = t.hand(P0, "Khalni Garden");
    t.play_land(P0, garden).unwrap();
    t.resolve_all();
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Plant").is_empty());
    let o = t.obj_now(garden);
    assert!(o.chars.has_subtype("Mountain"));
    assert_eq!(o.chars.name, "Khalni Garden");
    assert!(!o.chars.supertypes.contains(Supertype::Basic));
    let _ = zhao;
}
