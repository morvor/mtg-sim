//! Panglacial Wurm (hand-written, `src/cards/panglacial_wurm.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn may_be_cast_from_your_library_while_searching_it() {
    cr!("701.23a", "601.2");
    ruling!("Panglacial Wurm", "you pick up the search effect where you left off");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Panglacial Wurm");
    t.lands(P0, "Forest", 9);
    let rg = t.hand(P0, "Rampant Growth");
    t.cast(P0, rg).go();
    t.answer_yes(P0, true);
    t.resolve();
    // The Wurm is on the stack; the search found the Forest.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.permanents_controlled_by(P0).len(), 10);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Panglacial Wurm").len(), 1);
}

#[test]
fn not_cast_if_you_decline() {
    cr!("701.23a");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Panglacial Wurm");
    t.lands(P0, "Forest", 9);
    let rg = t.hand(P0, "Rampant Growth");
    t.cast(P0, rg).go();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.named_on_battlefield("Panglacial Wurm").is_empty());
}
