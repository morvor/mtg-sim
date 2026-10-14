//! Structure-coverage tests (round 8): keyword structures shared by many cards that no
//! other test exercised (see `docs/STRUCTURE_COVERAGE.md`).

use super::structure_activated_2::untapped;
use super::structure_seed_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn affinity_for_equipment_counts_only_your_equipment() {
    cr!("702.41a", "601.2f");
    // Rebel Salvo {2}{R}: "Affinity for Equipment" and "Rebel Salvo deals 5 damage to
    // target creature or planeswalker. That permanent loses indestructible until end of
    // turn."
    supported("Rebel Salvo");
    // Two Equipment you control: it costs {R}. Your other artifacts and the opponent's
    // Equipment don't count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Short Sword");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P1, "Bonesplitter");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Rebel Salvo");
    t.cast(P0, s).target(wurm).go();
    assert_eq!(untapped(&t, P0, "Mountain"), 0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"), "5 damage to a 6/4");
    // One Equipment: {1}{R}; a single Mountain isn't enough.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Memnite");
    t.battlefield(P1, "Bonesplitter");
    t.battlefield(P1, "Short Sword");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Rebel Salvo");
    assert!(t.cast(P0, s).target(wurm).try_go().is_err());
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    t.cast(P0, s).target(wurm).go();
    assert_eq!(untapped(&t, P0, "Mountain"), 0, "{{1}}{{R}}");
    // Three Equipment reduce only the generic part: it still costs {R}.
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.battlefield(P0, "Bonesplitter");
    }
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Wastes", 1);
    let s = t.hand(P0, "Rebel Salvo");
    assert!(t.cast(P0, s).target(wurm).try_go().is_err(), "{{R}} remains");
    t.clear_answers();
    t.lands(P0, "Mountain", 1);
    t.cast(P0, s).target(wurm).go();
    assert_eq!(untapped(&t, P0, "Mountain"), 0);
    assert_eq!(untapped(&t, P0, "Wastes"), 1);
}
