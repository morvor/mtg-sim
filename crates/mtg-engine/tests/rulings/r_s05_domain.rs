//! Rulings batch S05 — domain (an ability word): "for each basic land type among lands
//! you control".

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn only_basic_land_types_count_for_domain() {
    cr!("205.3i", "305.6");
    ruling!(
        "Kavu Scout",
        "The basic land types are Plains, Island, Swamp, Mountain, and Forest. Land types other than basic land types (such as Desert) don't contribute to domain abilities."
    );
    ruling!(
        "Nishoba Brawler",
        "The basic land types are Plains, Island, Swamp, Mountain, and Forest. Land types other than basic land types (such as Desert) don’t contribute to domain abilities."
    );
    supported("Kavu Scout");
    supported("Nishoba Brawler");
    supported("Sunscorched Desert");
    supported("Tropical Island");
    // Kavu Scout: "This creature gets +1/+0 for each basic land type among lands you
    // control." Nishoba Brawler: "power is equal to the number of basic land types among
    // lands you control."
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P0, "Kavu Scout");
    let brawler = t.battlefield(P0, "Nishoba Brawler");
    // A Desert and Wastes: no basic land types.
    t.lands(P0, "Sunscorched Desert", 2);
    t.lands(P0, "Wastes", 1);
    t.g.recompute();
    assert_eq!(t.pt(scout), (0, 2));
    assert_eq!(t.pt(brawler), (0, 3));
    // Two Mountains: one type.
    t.lands(P0, "Mountain", 2);
    t.g.recompute();
    assert_eq!(t.pt(scout), (1, 2));
    assert_eq!(t.pt(brawler), (1, 3));
    // Tropical Island is a Forest and an Island.
    t.lands(P0, "Tropical Island", 1);
    t.g.recompute();
    assert_eq!(t.pt(scout), (3, 2));
    assert_eq!(t.pt(brawler), (3, 3));
    // An opponent's Plains doesn't count.
    t.lands(P1, "Plains", 1);
    t.g.recompute();
    assert_eq!(t.pt(scout), (3, 2));
}
