//! CR 701.48: learn.

use crate::a701_028_071_common::*;
use mtg_engine::card::card;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Eyetwitch: "When this creature dies, learn." Returns the game after it died, with its
/// ability on the stack.
fn eyetwitch_dies(t: &mut TestGame) {
    supported("Eyetwitch");
    let eye = t.battlefield(P0, "Eyetwitch");
    t.g.move_object(eye, Zone::Graveyard(P0), MoveCause::Destroy, None);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

fn lesson(t: &mut TestGame) -> ObjectId {
    t.custom(
        P0,
        (*card("Environmental Sciences")).clone(),
        Zone::Outside(P0),
    )
}

#[test]
fn learn_discard_to_draw_or_a_lesson_from_outside_the_game_or_nothing() {
    cr!("701.48a");
    ruling!(
        "Eyetwitch",
        "If instructed to learn, you may do nothing. Discarding a card and putting a Lesson card into your hand are both optional."
    );
    // Options: 0 = nothing, 1 = discard then draw, 2 = a Lesson card.
    // Discard a card, then draw a card.
    let mut t = TestGame::new(2);
    let lesson_card = lesson(&mut t);
    t.hand(P0, "Forest");
    let top = t.library_top(P0, "Lightning Bolt");
    let _ = top;
    eyetwitch_dies(&mut t);
    option(&mut t, P0, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert_eq!(t.zone(lesson_card), Zone::Outside(P0));
    // A Lesson card from outside the game (the sideboard) into the hand.
    let mut t = TestGame::new(2);
    let lesson_card = lesson(&mut t);
    t.hand(P0, "Forest");
    eyetwitch_dies(&mut t);
    option(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(t.zone(lesson_card), Zone::Hand(P0));
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.g.player(P0).sideboard.is_empty());
    // Nothing.
    let mut t = TestGame::new(2);
    let lesson_card = lesson(&mut t);
    t.hand(P0, "Forest");
    let library = t.library_size(P0);
    eyetwitch_dies(&mut t);
    option(&mut t, P0, 0);
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert_eq!(t.zone(lesson_card), Zone::Outside(P0));
    assert_eq!(t.library_size(P0), library);
}

#[test]
fn only_a_lesson_card_can_be_fetched_and_only_without_discarding() {
    cr!("701.48a");
    let mut t = TestGame::new(2);
    // A non-Lesson card outside the game isn't an option; with no cards in hand there's
    // nothing to discard either.
    let other = t.custom(P0, (*card("Lightning Bolt")).clone(), Zone::Outside(P0));
    eyetwitch_dies(&mut t);
    t.resolve_all();
    assert_eq!(t.zone(other), Zone::Outside(P0));
    let offered: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .collect();
    // Only "do nothing" was possible, so no choice was asked.
    assert!(offered.is_empty());
}
