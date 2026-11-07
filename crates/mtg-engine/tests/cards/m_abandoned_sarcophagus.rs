//! Abandoned Sarcophagus (hand-written, `src/cards/abandoned_sarcophagus.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn cards_with_cycling_are_exiled_unless_cycled() {
    cr!("702.29c", "614.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    // A creature with cycling dies: exiled.
    let c = t.battlefield(P0, "Desert Cerodon");
    t.g.destroy(c, None);
    t.settle();
    assert!(t.in_exile("Desert Cerodon"));
    // A card without cycling goes to the graveyard.
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(b, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_cycled_card_goes_to_the_graveyard() {
    cr!("702.29c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    t.lands(P0, "Mountain", 1);
    let c = t.hand(P0, "Desert Cerodon");
    t.activate(P0, c, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Desert Cerodon"));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn a_card_with_cycling_discarded_otherwise_is_exiled() {
    cr!("701.9a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abandoned Sarcophagus");
    t.hand(P0, "Desert Cerodon");
    t.lands(P0, "Swamp", 3);
    let rot = t.hand(P0, "Mind Rot");
    t.cast(P0, rot).target(P0).go();
    t.resolve();
    assert!(t.in_exile("Desert Cerodon"));
}
