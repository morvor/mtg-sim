//! Spy Network (hand-written, `src/cards/spy_network.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn looks_at_hand_top_card_and_rearranges_your_top_four() {
    ruling!("Spy Network", "Only you get to look.");
    let mut t = TestGame::new(2);
    t.hand(P1, "Lightning Bolt");
    t.library_top(P1, "Grizzly Bears");
    let lib_before = t.library_size(P0);
    t.lands(P0, "Island", 1);
    let s = t.hand(P0, "Spy Network");
    t.cast(P0, s).target(P1).go();
    t.resolve();
    let log = t.dump_log();
    assert!(log.contains("Grizzly Bears"), "{log}");
    // Nothing was revealed or moved.
    assert_eq!(t.library_size(P0), lib_before);
    assert!(t.in_hand(P1, "Lightning Bolt"));
}
