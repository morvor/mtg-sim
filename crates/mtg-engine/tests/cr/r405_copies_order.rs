//! CR 405.3: when an effect puts several objects on the stack at the same time, a player
//! who controls more than one of them chooses their relative order. Copies of spells
//! created by one effect are put on the stack together (CR 707.10).

use crate::r703_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls Thousand-Year Storm ("Whenever you cast an instant or sorcery spell, copy
/// it for each other instant and sorcery spell you've cast before it this turn. You may
/// choose new targets for the copies.") and casts Shock, Shock, then Lightning Bolt at P1.
/// Bolt's trigger makes two copies: the first gets the new target P0, the second keeps P1.
/// They're ordered with `order` (first goes on the stack first). Returns the game after
/// the top copy resolved.
fn copies_ordered(order: Vec<usize>) -> TestGame {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thousand-Year Storm");
    t.lands(P0, "Mountain", 3);
    for name in ["Shock", "Shock"] {
        let c = t.hand(P0, name);
        t.cast(P0, c).target(P1).go();
        t.settle();
        t.resolve_all();
    }
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.life(P1), 14);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_yes(P0, false);
    t.answer(P0, DecisionKind::Order, Answer::Indices(order));
    t.resolve(); // the trigger
    assert_eq!(t.stack_len(), 3, "{}", t.dump_log());
    t.resolve(); // the top copy
    t
}

#[test]
fn copies_created_at_once_are_ordered_by_their_controller() {
    cr!("405.3", "707.10c");
    supported("Thousand-Year Storm");
    // The copy targeting P1 first (bottom), then the one targeting P0 on top.
    let t = copies_ordered(vec![1, 0]);
    assert_eq!((t.life(P0), t.life(P1)), (17, 14), "{}", t.dump_log());
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { items, .. } if items.len() == 2)));
    // The other order: the copy targeting P1 resolves first.
    let t = copies_ordered(vec![0, 1]);
    assert_eq!((t.life(P0), t.life(P1)), (20, 11));
}

#[test]
fn identical_copies_need_no_order() {
    cr!("405.3");
    // Thousand-Year Storm copies the third spell twice; the copies keep the same target,
    // so they're indistinguishable and nobody is asked to order them.
    supported("Thousand-Year Storm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thousand-Year Storm");
    t.lands(P0, "Mountain", 3);
    for name in ["Shock", "Shock", "Lightning Bolt"] {
        let c = t.hand(P0, name);
        t.cast(P0, c).target(P1).go();
        t.settle();
    }
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 2 * 3 - 3 * 3, "{}", t.dump_log());
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::Order { .. })));
}
