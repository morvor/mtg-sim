//! Rulings batch P108 — "look at the top N cards of your library, put one (or two) of them
//! into your hand and the rest into your graveyard": with fewer than N cards you look at
//! all of them and do as much as possible (CR 609.3); putting cards into your hand this
//! way isn't drawing (CR 121.1) and putting them into your graveyard isn't discarding
//! (CR 701.9a), so an empty library doesn't make you lose (CR 104.3c, 704.5b).

use crate::r_p108_common::*;
use crate::r_s02_common::can_activate;
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Replaces `p`'s library with `n` real cards (Grizzly Bears).
fn library(t: &mut TestGame, p: PlayerId, n: usize) {
    t.g.players[p.idx()].library.clear();
    for _ in 0..n {
        t.library_top(p, "Grizzly Bears");
    }
}

/// The number of Grizzly Bears cards in `p`'s graveyard.
fn bears_in_graveyard(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p)
        .graveyard
        .iter()
        .filter(|c| t.g.obj(**c).chars.name == "Grizzly Bears")
        .count()
}

/// Asserts that `p`'s library is empty, `to_hand` Grizzly Bears were put into the hand
/// and `to_gy` into the graveyard, and that `p` hasn't lost.
fn split(t: &TestGame, p: PlayerId, to_hand: usize, to_gy: usize) {
    assert_eq!(t.library_size(p), 0);
    let in_hand = t
        .g
        .player(p)
        .hand
        .iter()
        .filter(|c| t.g.obj(**c).chars.name == "Grizzly Bears")
        .count();
    assert_eq!(in_hand, to_hand);
    assert_eq!(bears_in_graveyard(t, p), to_gy);
    assert!(!t.has_lost(p));
}

/// Casts the real spell `name` (with lands for its cost) with a library of `n` cards and
/// resolves it.
fn cast_with_library(name: &str, n: usize, targets: &[Entity]) -> TestGame {
    supported(name);
    let mut t = TestGame::new(2);
    library(&mut t, P0, n);
    cast_new(&mut t, P0, name, targets);
    t.resolve_all();
    t
}

#[test]
fn strategic_planning_with_fewer_than_three_cards() {
    cr!("609.3");
    ruling!(
        "Strategic Planning",
        "If there are fewer than three cards in your library, you look at all of them, put one of them into your hand, and put the rest into your graveyard."
    );
    let t = cast_with_library("Strategic Planning", 2, &[]);
    split(&t, P0, 1, 1);
}

#[test]
fn forbidden_alchemy_with_fewer_than_four_cards() {
    cr!("609.3");
    ruling!(
        "Forbidden Alchemy",
        "If you have fewer than four cards in your library, you'll look at all the cards there and put one into your hand and the rest into your graveyard."
    );
    let t = cast_with_library("Forbidden Alchemy", 3, &[]);
    split(&t, P0, 1, 2);
}

#[test]
fn ancestral_memories_with_fewer_than_seven_cards() {
    cr!("609.3");
    ruling!(
        "Ancestral Memories",
        "If there are less than 7 cards in the library, look at all of them. Put two into your hand, and the rest in the graveyard."
    );
    let t = cast_with_library("Ancestral Memories", 5, &[]);
    split(&t, P0, 2, 3);
}

#[test]
fn ancestral_memories_isnt_drawing_or_discarding() {
    cr!("121.1", "701.9a");
    ruling!(
        "Ancestral Memories",
        "This is not considered a draw or a discard."
    );
    let t = cast_with_library("Ancestral Memories", 10, &[]);
    assert_eq!(t.library_size(P0), 3);
    assert!(!t.g.turn_events.iter().any(|e| matches!(
        e,
        Event::Drew { .. } | Event::Discarded { .. }
    )));
    assert_eq!(bears_in_graveyard(&t, P0), 5);
}

#[test]
fn rals_outburst_with_one_card_doesnt_lose() {
    cr!("609.3", "104.3c", "704.5b");
    ruling!(
        "Ral's Outburst",
        "If you have only one card in your library, you put it into your hand. You won’t lose the game if your library’s empty until you try to draw from the empty library."
    );
    let mut t = cast_with_library("Ral's Outburst", 1, &[Entity::Player(P1)]);
    split(&t, P0, 1, 0);
    assert_eq!(t.life(P1), 17);
    t.settle();
    assert!(!t.has_lost(P0));
}

#[test]
fn rals_outburst_with_an_illegal_target_doesnt_look() {
    cr!("608.2b");
    ruling!(
        "Ral's Outburst",
        "If the target permanent or player is an illegal target by the time Ral’s Outburst tries to resolve, the spell doesn’t resolve. You won’t look at the top two cards of your library."
    );
    supported("Ral's Outburst");
    let mut t = TestGame::new(2);
    library(&mut t, P0, 5);
    let bears = t.battlefield(P1, "Hill Giant");
    let spell = cast_new(&mut t, P0, "Ral's Outburst", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert_eq!(t.library_size(P0), 5);
    assert_eq!(bears_in_graveyard(&t, P0), 0);
}

/// `act` makes P0's permanent's "look at the top two cards" ability trigger with a
/// one-card library; that card is put into P0's hand.
fn one_card_to_hand(name: &str, act: impl FnOnce(&mut TestGame)) {
    supported(name);
    let mut t = TestGame::new(2);
    library(&mut t, P0, 1);
    act(&mut t);
    t.resolve_all();
    split(&t, P0, 1, 0);
}

#[test]
fn tower_geist_with_one_card() {
    cr!("609.3");
    ruling!(
        "Tower Geist",
        "If there's only one card in your library when Tower Geist enters, you'll look at that card and put it into your hand."
    );
    one_card_to_hand("Tower Geist", |t| {
        t.enter(P0, "Tower Geist");
    });
}

#[test]
fn codecracker_hound_with_one_card() {
    cr!("609.3");
    ruling!(
        "Codecracker Hound",
        "If there’s only one card in your library as Codecracker Hound’s first ability resolves, you’ll put that card into your hand."
    );
    one_card_to_hand("Codecracker Hound", |t| {
        t.enter(P0, "Codecracker Hound");
    });
}

#[test]
fn talas_lookout_with_one_card() {
    cr!("609.3");
    ruling!(
        "Talas Lookout",
        "If there is only one card left in your library, that card is put into your hand."
    );
    one_card_to_hand("Talas Lookout", |t| {
        let l = t.battlefield(P0, "Talas Lookout");
        destroy(t, l);
    });
}

#[test]
fn waker_of_waves_is_activated_only_from_your_hand() {
    cr!("113.6", "602.1");
    ruling!(
        "Waker of Waves",
        "You can activate the last ability of Waker of Waves only if it's in your hand."
    );
    supported("Waker of Waves");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let on_bf = t.battlefield(P0, "Waker of Waves");
    assert!(!can_activate(&mut t, P0, on_bf));
    let in_gy = t.graveyard(P0, "Waker of Waves");
    assert!(!can_activate(&mut t, P0, in_gy));
    let in_hand = t.hand(P0, "Waker of Waves");
    assert!(can_activate(&mut t, P0, in_hand));
}

#[test]
fn riddles_in_the_dark_may_split_four_and_zero() {
    cr!("700.3d", "700.3a");
    ruling!(
        "Riddles in the Dark",
        "You may split the cards into one pile of four and one pile of zero."
    );
    supported("Riddles in the Dark");
    // All four face up (an empty face-down pile); the opponent takes the face-up pile.
    let mut t = TestGame::new(2);
    library(&mut t, P0, 4);
    t.answer_choose(P0, &[]);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    cast_new(&mut t, P0, "Riddles in the Dark", &[]);
    t.resolve_all();
    split(&t, P0, 4, 0);
    // All four face down; the opponent picks the empty face-up pile.
    let mut t = TestGame::new(2);
    library(&mut t, P0, 4);
    let top4: Vec<Entity> = t.g.player(P0).library.iter().map(|c| obj(*c)).collect();
    t.answer_choose(P0, &top4);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    cast_new(&mut t, P0, "Riddles in the Dark", &[]);
    t.resolve_all();
    split(&t, P0, 0, 4);
}
