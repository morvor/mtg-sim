//! Rulings batch P226 — Matzalantli, the Great Door // The Core: "Activate only if there
//! are four or more permanent types among cards in your graveyard" (CR 110.4a, 602.5b),
//! and a permanent that transforms stays the same tapped object (CR 712.18).

use crate::r_s01_common::*;
use crate::r_s02_common::can_activate;
use crate::r_s06_common::activate_containing;
use crate::r_s08_common::is_tapped;
use crate::r_s17_common::name_of;
use mtg_engine::testing::*;
use mtg_engine::*;

const MATZALANTLI: &str = "Matzalantli, the Great Door // The Core";

#[test]
fn matzalantli_transforms_in_place_with_four_permanent_types_in_the_graveyard() {
    cr!("110.4a", "602.5b", "712.18", "701.27a");
    ruling!(
        "Matzalantli, the Great Door // The Core",
        "Transforming Matzalantli doesn't cause it to leave the battlefield. It also doesn't cause it to become untapped."
    );
    supported(MATZALANTLI);
    // "{4}, {T}: Transform Matzalantli. Activate only if there are four or more permanent
    // types among cards in your graveyard." Artifact, creature, land and an instant: four
    // card types but three permanent types.
    let mut t = TestGame::new(2);
    let door = t.battlefield(P0, MATZALANTLI);
    t.lands(P0, "Wastes", 4);
    bury(
        &mut t,
        &["Mind Stone", "Grizzly Bears", "Forest", "Lightning Bolt"],
    );
    assert!(activate_containing(&mut t, P0, door, "Transform").is_err());
    // An enchantment makes four.
    t.graveyard(P0, "Pacifism");
    activate_containing(&mut t, P0, door, "Transform").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, door), "The Core");
    assert_eq!(t.g.current(door), door);
    assert!(t.on_battlefield(door));
    assert!(is_tapped(&t, door));
    assert!(!can_activate(&mut t, P0, door));
}

fn bury(t: &mut TestGame, names: &[&str]) {
    for n in names {
        t.graveyard(P0, n);
    }
}

#[test]
fn matzalantli_loots_and_the_core_adds_mana_for_each_permanent_card() {
    cr!("110.4a", "207.2c");
    // "{T}: Draw a card, then discard a card."
    let mut t = TestGame::new(2);
    let door = t.battlefield(P0, MATZALANTLI);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, door, "Draw a card").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), 1);
    // The Core: "Fathomless descent — {T}: Add X mana of any one color, where X is the
    // number of permanent cards in your graveyard." Two permanent cards and an instant.
    let mut t = TestGame::new(2);
    let core = crate::r_s17_common::enter_transformed(&mut t, P0, MATZALANTLI);
    assert_eq!(name_of(&t, core), "The Core");
    bury(&mut t, &["Grizzly Bears", "Forest", "Lightning Bolt"]);
    t.activate(P0, core, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}
