//! CR 603.7g: delayed triggered abilities created by a static ability that lets a player
//! take an action (revealing a card from an opening hand, CR 103.6).

use mtg_engine::object::*;
use mtg_engine::opening_hand::opening_hand_actions;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_revealed_cards_delayed_trigger_has_that_card_as_its_source() {
    cr!("603.7g", "103.6", "103.6b");
    // Chancellor of the Spires: "You may reveal this card from your opening hand. If you
    // do, at the beginning of the first upkeep, each opponent mills seven cards."
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Chancellor of the Spires");
    t.answer_yes(P0, true);
    opening_hand_actions(&mut t.g);
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { source: Some(s), .. } if *s == c)));
    let lib = t.library_size(P1);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let top = *t.g.stack.last().unwrap();
    // Its source is the card with the static ability (still in the hand), and it's
    // controlled by the player who revealed it.
    match &t.obj(top).stack.as_deref().unwrap().kind {
        StackKind::Triggered { source, .. } => assert_eq!(*source, c),
        other => panic!("not a triggered ability: {other:?}"),
    }
    assert_eq!(t.obj(top).controller, P0);
    t.resolve();
    assert_eq!(t.library_size(P1), lib - 7);
    // It triggers only once.
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);

    // Not revealing it: no delayed trigger.
    let mut t = TestGame::new(2);
    t.hand(P0, "Chancellor of the Spires");
    t.answer_yes(P0, false);
    opening_hand_actions(&mut t.g);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_card_may_begin_the_game_on_the_battlefield() {
    cr!("103.6", "103.6a");
    let mut t = TestGame::new(2);
    t.hand(P0, "Leyline of Sanctity");
    t.answer_yes(P0, true);
    opening_hand_actions(&mut t.g);
    assert_eq!(t.named_on_battlefield("Leyline of Sanctity").len(), 1);
    let mut t = TestGame::new(2);
    t.hand(P0, "Leyline of Sanctity");
    t.answer_yes(P0, false);
    opening_hand_actions(&mut t.g);
    assert!(t.in_hand(P0, "Leyline of Sanctity"));
}
