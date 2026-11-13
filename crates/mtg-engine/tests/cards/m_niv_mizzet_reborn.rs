//! Niv-Mizzet Reborn (hand-written, `src/cards/niv_mizzet_reborn.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn takes_one_card_of_each_color_pair() {
    cr!("105.1");
    ruling!("Niv-Mizzet Reborn", "you choose as many cards as you can");
    let mut t = TestGame::new(2);
    // Two blue-red cards, one white-blue, one mono-red.
    t.library_top(P0, "Izzet Charm");
    t.library_top(P0, "Lightning Helix");
    t.library_top(P0, "Electrolyze");
    t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Azorius Charm");
    t.enter(P0, "Niv-Mizzet Reborn");
    t.resolve_all();
    // Azorius Charm (WU), Lightning Helix (RW) and one of the UR cards.
    assert!(t.in_hand(P0, "Azorius Charm"));
    assert!(t.in_hand(P0, "Lightning Helix"));
    assert!(!t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(t.hand_size(P0), 3);
    assert_eq!(t.library_size(P0), 32);
}
