//! The Temporal Anchor (hand-written, `src/cards/the_temporal_anchor.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn exiles_as_many_cards_from_the_bottom_as_scry_put_there() {
    cr!("701.22a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Temporal Anchor");
    let a = t.library_top(P0, "Island");
    let b = t.library_top(P0, "Forest");
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![b, a]));
    mtg_engine::library::scry(&mut t.g, P0, 2);
    t.resolve_all();
    // The two scried cards were put on the bottom, then exiled from there.
    assert!(t.in_exile("Island"));
    assert!(t.in_exile("Forest"));
    assert_eq!(t.library_size(P0), 30);
}

#[test]
fn keeping_them_on_top_exiles_nothing() {
    cr!("701.22a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Temporal Anchor");
    let a = t.library_top(P0, "Island");
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![a], vec![]));
    mtg_engine::library::scry(&mut t.g, P0, 1);
    t.resolve_all();
    assert_eq!(t.library_size(P0), 31);
}
