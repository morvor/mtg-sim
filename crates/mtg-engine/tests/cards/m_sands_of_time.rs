//! Sands of Time (hand-written, `src/cards/sands_of_time.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn upkeep_toggles_tapped_and_untapped_permanents() {
    cr!("614.10", "603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sands of Time");
    let forest = t.battlefield(P1, "Forest");
    t.g.objects[forest.0 as usize].tapped = true;
    let bears = t.battlefield(P1, "Grizzly Bears");
    // An enchantment isn't affected.
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    // The untap step was skipped: the Forest is still tapped until the trigger resolves.
    assert!(t.obj_now(forest).tapped);
    t.resolve_all();
    assert!(!t.obj_now(forest).tapped);
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(anthem).tapped);
}
