//! Undiscovered Paradise (hand-written, `src/cards/undiscovered_paradise.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn returns_to_hand_during_your_next_untap_step() {
    cr!("502.3", "502.4");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Undiscovered Paradise");
    t.activate(P0, land, 0, &[]).unwrap();
    // Not during the opponent's untap step.
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(land));
    t.advance_to(P0, Step::Upkeep);
    assert!(t.in_hand(P0, "Undiscovered Paradise"));
}

#[test]
fn stays_if_not_tapped_for_mana() {
    cr!("502.3");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Undiscovered Paradise");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.on_battlefield(land));
}
