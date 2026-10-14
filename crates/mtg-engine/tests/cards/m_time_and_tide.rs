//! Time and Tide (hand-written, `src/cards/time_and_tide.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn phased_out_creatures_phase_in_and_phasing_creatures_phase_out() {
    cr!("702.26b", "702.26c");
    let mut t = TestGame::new(2);
    let gone = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[gone.0 as usize].phased_out = true;
    t.g.objects[gone.0 as usize].phased_out_under = Some(P1);
    let phaser = t.battlefield(P1, "Breezekeeper");
    t.g.recompute();
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Time and Tide");
    t.cast(P0, s).go();
    t.resolve();
    assert!(!t.obj_now(gone).phased_out);
    assert!(t.obj_now(phaser).phased_out);
}
