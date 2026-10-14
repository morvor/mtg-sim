//! Mana Web (hand-written, `src/cards/mana_web.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn taps_the_lands_that_could_produce_the_same_type() {
    cr!("106.7", "603.2");
    ruling!("Mana Web", "The lands that are tapped by Mana Web’s ability don’t produce mana");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Web");
    let f1 = t.battlefield(P1, "Forest");
    let f2 = t.battlefield(P1, "Forest");
    let m = t.battlefield(P1, "Mountain");
    // A Taiga could produce green too.
    let taiga = t.battlefield(P1, "Taiga");
    t.activate(P1, f1, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(f1).tapped);
    assert!(t.obj_now(f2).tapped);
    assert!(t.obj_now(taiga).tapped);
    assert!(!t.obj_now(m).tapped);
    // The lands it tapped produced no mana: only the first Forest's {G} is in the pool.
    assert_eq!(t.g.player(P1).mana_pool.total(), 1);
}

#[test]
fn your_own_lands_dont_trigger_it() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mana Web");
    let f1 = t.battlefield(P0, "Forest");
    let f2 = t.battlefield(P0, "Forest");
    t.activate(P0, f1, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(f2).tapped);
}
