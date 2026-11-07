//! Pyramids (hand-written, `src/cards/pyramids.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_next_destruction_of_the_land_is_replaced() {
    cr!("614.1a", "701.8a");
    let mut t = TestGame::new(2);
    let pyr = t.battlefield(P0, "Pyramids");
    let forest = t.battlefield(P0, "Forest");
    t.lands(P0, "Plains", 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.activate(P0, pyr, 0, &[Entity::Object(forest)]).unwrap();
    t.resolve_all();
    t.g.objects[forest.0 as usize].tapped = false;
    t.g.objects[forest.0 as usize].damage = 2;
    t.g.destroy(forest, None);
    assert!(t.on_battlefield(forest));
    assert_eq!(t.obj_now(forest).damage, 0);
    // Unlike regeneration, it isn't tapped.
    assert!(!t.obj_now(forest).tapped);
    // Used up.
    t.g.destroy(forest, None);
    assert!(!t.on_battlefield(forest));
}

#[test]
fn destroys_an_aura_attached_to_a_land() {
    cr!("701.8a");
    let mut t = TestGame::new(2);
    let pyr = t.battlefield(P0, "Pyramids");
    let forest = t.battlefield(P1, "Forest");
    let wg = t.battlefield(P1, "Wild Growth");
    t.g.attach(wg, Entity::Object(forest));
    t.lands(P0, "Plains", 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.activate(P0, pyr, 0, &[Entity::Object(wg)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Wild Growth"));
}
