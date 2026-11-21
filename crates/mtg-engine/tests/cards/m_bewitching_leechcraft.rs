//! Bewitching Leechcraft (hand-written, `src/cards/bewitching_leechcraft.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn leeched(counters: u32) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    if counters > 0 {
        t.g.add_counters(Entity::Object(bear), "+1/+1", counters, None);
    }
    let l = t.battlefield(P0, "Bewitching Leechcraft");
    t.g.attach(l, Entity::Object(bear));
    t.g.objects[bear.0 as usize].tapped = true;
    t.g.recompute();
    t.advance_to(P1, Step::Upkeep);
    (t, bear)
}

#[test]
fn removes_a_counter_and_untaps() {
    cr!("502.3", "614.1a");
    let (t, bear) = leeched(2);
    assert!(!t.obj_now(bear).tapped);
    assert_eq!(t.counters(bear, "+1/+1"), 1);
}

#[test]
fn without_a_counter_it_stays_tapped() {
    cr!("502.3");
    let (t, bear) = leeched(0);
    assert!(t.obj_now(bear).tapped);
}
