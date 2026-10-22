//! Rulings batch S10 — learn (CR 701.48): "You may reveal a Lesson card you own from
//! outside the game and put it into your hand, or discard a card to draw a card."

use crate::r_s01_common::{give_mana_for, supported};
use mtg_engine::card::card;
use mtg_engine::decision::Answer;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p`'s Eyetwitch ("When this creature dies, learn.") dies; `p` learns by putting the
/// Lesson card Environmental Sciences from outside the game into their hand.
fn learn_a_lesson(t: &mut TestGame, p: PlayerId) -> ObjectId {
    supported("Eyetwitch");
    let lesson = t.custom(
        p,
        (*card("Environmental Sciences")).clone(),
        Zone::Outside(p),
    );
    // (A card in hand, so that the options are the same as with one to discard.)
    t.hand(p, "Forest");
    let eye = t.battlefield(p, "Eyetwitch");
    t.g.move_object(eye, Zone::Graveyard(p), MoveCause::Destroy, None);
    t.settle();
    // Options: 0 = nothing, 1 = discard then draw, 2 = a Lesson card.
    t.answer(p, DecisionKind::Option, Answer::Index(2));
    t.resolve_all();
    assert_eq!(t.zone(lesson), Zone::Hand(p));
    lesson
}

#[test]
fn a_card_brought_in_from_outside_the_game_stays_in_the_game() {
    cr!("400.11a", "701.48a", "800.4a");
    ruling!(
        "Eyetwitch",
        "If a card is brought into the game from outside the game, it will stay in the game until it ends or until its owner leaves the game, whichever comes first."
    );
    supported("Environmental Sciences");
    // Once cast, the Lesson goes to the graveyard like any other card, not back to the
    // sideboard.
    let mut t = TestGame::new(2);
    let lesson = learn_a_lesson(&mut t, P0);
    t.library_top(P0, "Forest");
    t.g.search_finds_by_default = true;
    give_mana_for(&mut t, P0, "Environmental Sciences");
    let in_hand = t.g.current(lesson);
    t.cast(P0, in_hand).go();
    t.resolve_all();
    assert_eq!(t.zone(lesson), Zone::Graveyard(P0));
    assert!(t.g.player(P0).sideboard.is_empty());
    assert_eq!(t.life(P0), 22);
    // In a multiplayer game, it leaves the game with its owner.
    let mut t = TestGame::new(3);
    let lesson = learn_a_lesson(&mut t, P0);
    t.g.player_loses(P0);
    assert_eq!(t.zone(lesson), Zone::Nowhere);
    assert!(t.g.player(P0).sideboard.is_empty());
    assert!(t.g.result.is_none());
}
