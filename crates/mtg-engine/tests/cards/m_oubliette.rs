//! Oubliette (hand-written, `src/cards/oubliette.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_creature_phases_in_tapped_when_oubliette_leaves() {
    cr!("702.26a", "610.3");
    ruling!("Oubliette", "it phases in immediately after Oubliette leaves the battlefield");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bear)]);
    let oub = t.enter(P0, "Oubliette");
    t.resolve_all();
    assert!(t.obj_now(bear).phased_out);
    t.g.destroy(oub, None);
    t.resolve_all();
    assert!(!t.obj_now(bear).phased_out);
    assert!(t.obj_now(bear).tapped);
}

#[test]
fn nothing_happens_if_it_left_before_the_ability_resolved() {
    cr!("610.3");
    ruling!("Oubliette", "the target creature won't be phased out or tapped");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bear)]);
    let oub = t.enter(P0, "Oubliette");
    t.settle();
    t.g.destroy(oub, None);
    t.resolve_all();
    assert!(!t.obj_now(bear).phased_out);
    assert!(!t.obj_now(bear).tapped);
}
