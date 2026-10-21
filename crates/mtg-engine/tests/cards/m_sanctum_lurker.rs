//! Sanctum Lurker (hand-written, `src/cards/sanctum_lurker.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn your_planeswalkers_survive_0_loyalty() {
    cr!("704.5i");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanctum Lurker");
    let jace = t.battlefield(P0, "Jace Beleren");
    let theirs = t.battlefield(P1, "Jace Beleren");
    t.g.objects[jace.0 as usize].counters.remove("loyalty");
    t.g.objects[theirs.0 as usize].counters.remove("loyalty");
    t.g.recompute();
    t.settle();
    assert!(t.on_battlefield(jace));
    assert!(!t.on_battlefield(theirs));
}
