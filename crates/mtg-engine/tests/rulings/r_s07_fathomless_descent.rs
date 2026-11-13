//! Rulings batch S07 — fathomless descent: an ability word for abilities that care how
//! many permanent cards are in your graveyard (CR 207.2c).

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn fathomless_descent_counts_permanent_cards_in_your_graveyard() {
    cr!("207.2c", "110.4a");
    ruling!(
        "Terror Tide",
        "Cards with the ability word \"fathomless descent\" have abilities that care how many permanent cards are in your graveyard."
    );
    supported("Terror Tide");
    // Terror Tide: "Fathomless descent — All creatures get -X/-X until end of turn, where X
    // is the number of permanent cards in your graveyard."
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let wall = t.battlefield(P0, "Wall of Stone");
    // Permanent cards: a creature, a land, an artifact (3). Instants and sorceries don't
    // count; neither do the cards in an opponent's graveyard.
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Millstone");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Divination");
    t.graveyard(P1, "Hill Giant");
    t.graveyard(P1, "Mountain");
    t.lands(P0, "Swamp", 4);
    let tide = t.hand(P0, "Terror Tide");
    t.cast(P0, tide).go();
    t.resolve_all();
    // Craw Wurm (6/4) gets -3/-3; Wall of Stone (0/8) too.
    assert_eq!(t.pt(wurm), (3, 1));
    assert_eq!(t.pt(wall), (-3, 5));
    // Souls of the Lost: "Fathomless descent — Souls of the Lost's power is equal to the
    // number of permanent cards in your graveyard and its toughness is equal to that
    // number plus 1." Terror Tide (a sorcery) is now in the graveyard too; still three.
    supported("Souls of the Lost");
    let souls = t.battlefield(P0, "Souls of the Lost");
    assert_eq!(t.pt(souls), (3, 4));
    t.graveyard(P0, "Plains");
    assert_eq!(t.pt(souls), (4, 5));
}
