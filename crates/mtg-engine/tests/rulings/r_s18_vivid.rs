//! Rulings batch S18 — vivid: "the number of colors among permanents you control".

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn lands_are_colorless_even_if_they_tap_for_colored_mana() {
    cr!("105.2c", "202.2b", "604.3");
    ruling!(
        "Squawkroaster",
        "Lands are normally colorless permanents, even if they tap for mana of a certain color."
    );
    supported("Squawkroaster");
    // Squawkroaster (red, */4): "Squawkroaster's power is equal to the number of colors
    // among permanents you control."
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P0, "Squawkroaster");
    t.settle();
    assert_eq!(t.pt(bird), (1, 4));
    for land in ["Plains", "Island", "Swamp", "Forest", "Tropical Island"] {
        t.lands(P0, land, 1);
    }
    t.settle();
    assert_eq!(t.pt(bird), (1, 4));
    // A green creature adds green.
    t.battlefield(P0, "Llanowar Elves");
    t.settle();
    assert_eq!(t.pt(bird), (2, 4));
}

#[test]
fn the_five_colors_and_colorless_isnt_one() {
    cr!("105.1", "105.2c");
    ruling!(
        "Squawkroaster",
        "The colors are white, blue, black, red, and green. Colorless is not a color."
    );
    // Wildvine Pummeler ({6}{G}): "This spell costs {1} less to cast for each color among
    // permanents you control."
    supported("Wildvine Pummeler");
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P0, "Squawkroaster");
    // Colorless permanents add nothing.
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Memnite");
    t.settle();
    assert_eq!(t.pt(bird), (1, 4));
    // Four colors, each counted once however many permanents have it: Wildvine Pummeler
    // costs {2}{G}, too much for two Forests.
    for name in [
        "Serra Angel",
        "Sprouting Thrinax",
        "Tidehollow Sculler",
        "Grizzly Bears",
    ] {
        t.battlefield(P0, name);
    }
    t.settle();
    assert_eq!(t.pt(bird), (4, 4));
    t.lands(P0, "Forest", 2);
    let pummeler = t.hand(P0, "Wildvine Pummeler");
    assert!(!can_cast(&mut t, P0, pummeler, CastMethod::Normal));
    // The fifth color, blue: it costs {1}{G}.
    t.battlefield(P0, "Delver of Secrets");
    t.settle();
    assert_eq!(t.pt(bird), (5, 4));
    assert!(can_cast(&mut t, P0, pummeler, CastMethod::Normal));
    t.cast(P0, pummeler).go();
    t.resolve_all();
    assert!(t.on_battlefield(pummeler));
}
