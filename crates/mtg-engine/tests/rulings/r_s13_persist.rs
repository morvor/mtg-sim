//! Rulings batch S13 — persist (CR 702.79a): "When this permanent is put into a graveyard
//! from the battlefield, if it had no -1/-1 counters on it, return it to the battlefield
//! under its owner's control with a -1/-1 counter on it."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_persist_card_that_left_the_graveyard_before_the_trigger_resolves_isnt_returned() {
    cr!("702.79a", "400.7");
    ruling!(
        "Lesser Masticore",
        "If a card with persist is removed from the graveyard after it dies but before the ability trigger resolves, it won't be returned to the battlefield."
    );
    supported("Lesser Masticore");
    supported("Bojuka Bog");
    // Normally, Lesser Masticore returns with a -1/-1 counter.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Lesser Masticore");
    t.g.destroy(m, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let back = t.named_on_battlefield("Lesser Masticore");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], counters::MINUS1), 1);
    // With the persist trigger on the stack, Bojuka Bog ("When this land enters, exile
    // target player's graveyard.") exiles the card first: it stays in exile.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Lesser Masticore");
    t.g.destroy(m, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_graveyard(P0, "Lesser Masticore"));
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.enter(P1, "Bojuka Bog");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_exile("Lesser Masticore"));
    assert!(t.named_on_battlefield("Lesser Masticore").is_empty());
}
