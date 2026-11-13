//! Gilded Drake (hand-written, `src/cards/gilded_drake.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn exchanges_control_with_the_target() {
    cr!("701.12a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let drake = t.enter(P0, "Gilded Drake");
    t.resolve_all();
    assert_eq!(t.obj_now(giant).controller, P0);
    assert_eq!(t.obj_now(drake).controller, P1);
}

#[test]
fn still_resolves_and_is_sacrificed_if_its_target_becomes_illegal() {
    cr!("608.2b");
    ruling!("Gilded Drake", "you sacrifice this card on resolution");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Gilded Drake");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(giant, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Gilded Drake"));
}

#[test]
fn sacrificed_without_a_target() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.enter(P0, "Gilded Drake");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Gilded Drake"));
}
