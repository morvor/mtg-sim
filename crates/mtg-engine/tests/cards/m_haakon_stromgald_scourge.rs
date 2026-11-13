//! Haakon, Stromgald Scourge (hand-written, `src/cards/haakon_stromgald_scourge.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn cast_from_your_graveyard() {
    cr!("601.3");
    ruling!("Haakon, Stromgald Scourge", "While Haakon is in your graveyard, you may cast it");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let h = t.graveyard(P0, "Haakon, Stromgald Scourge");
    t.cast(P0, h).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Haakon, Stromgald Scourge").len(), 1);
}

#[test]
fn not_from_your_hand() {
    cr!("601.3");
    ruling!("Haakon, Stromgald Scourge", "Haakon can't be cast from your hand");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let h = t.hand(P0, "Haakon, Stromgald Scourge");
    assert!(t.cast(P0, h).try_go().is_err());
}
