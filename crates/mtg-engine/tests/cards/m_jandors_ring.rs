//! Jandor's Ring (hand-written, `src/cards/jandors_ring.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn discards_the_last_card_drawn_this_turn() {
    cr!("602.2b", "118.3");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Jandor's Ring");
    t.lands(P0, "Plains", 2);
    t.hand(P0, "Lightning Bolt");
    t.library_top(P0, "Grizzly Bears");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.activate(P0, ring, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn cant_be_activated_without_that_card_in_hand() {
    cr!("602.2b", "118.3");
    ruling!("Jandor's Ring", "If you do not have the card still in your hand, you can’t pay the cost.");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Jandor's Ring");
    t.lands(P0, "Plains", 2);
    t.hand(P0, "Lightning Bolt");
    // Nothing drawn this turn.
    assert!(t.activate(P0, ring, 0, &[]).is_err());
    assert!(t.in_hand(P0, "Lightning Bolt"));
}
