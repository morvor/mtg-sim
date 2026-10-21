//! Limits on the value announced for X (CR 107.3a, 601.2b, 602.2b): "X can't be greater
//! than the number of players in the game." (Open the Way), "X can't be greater than the
//! greatest toughness among creatures you control." after an additional cost (Soul
//! Immolation), and an X the text defines: "X is the number of cards in an opponent's
//! hand." (Bargaining Table, CR 107.3c).

use crate::basic_effects_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn lands_on_battlefield(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.is(types::CardType::Land))
        .count()
}

#[test]
fn open_the_way_x_is_at_most_the_number_of_players() {
    cr!("107.3a", "601.2b");
    assert_supported("Open the Way");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    for _ in 0..6 {
        t.library_top(P0, "Forest");
    }
    let before = lands_on_battlefield(&t, P0);
    let otw = t.hand(P0, "Open the Way");
    // Three is more than the number of players: not a value that may be announced.
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let spell = t.cast(P0, otw).go();
    assert_eq!(t.g.obj(spell).stack.as_ref().and_then(|s| s.x), Some(2));
    t.resolve();
    assert_eq!(lands_on_battlefield(&t, P0), before + 2);
}

#[test]
fn open_the_way_checks_the_number_of_players_only_as_x_is_chosen() {
    cr!("107.3a", "601.2b");
    ruling!("Open the Way", "checked only as you choose the value for X");
    let mut t = TestGame::new(3);
    t.lands(P0, "Forest", 6);
    for _ in 0..6 {
        t.library_top(P0, "Forest");
    }
    let before = lands_on_battlefield(&t, P0);
    let otw = t.hand(P0, "Open the Way");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.cast(P0, otw).go();
    // A player leaves the game in response: X stays 3.
    t.g.lose_game(P2);
    t.resolve();
    assert_eq!(lands_on_battlefield(&t, P0), before + 3);
}

#[test]
fn soul_immolation_x_is_at_most_the_greatest_toughness() {
    cr!("107.3a", "601.2b", "601.2f");
    assert_supported("Soul Immolation");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 10);
    let si = t.hand(P0, "Soul Immolation");
    // X = 4 is greater than the greatest toughness (3): the greatest legal value is used.
    t.answer(P0, DecisionKind::X, Answer::Number(4));
    t.cast(P0, si).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.g.obj(wurm).damage, 3);
}

#[test]
fn bargaining_table_x_is_the_number_of_cards_in_an_opponents_hand() {
    cr!("107.3c", "602.2b");
    ruling!("Bargaining Table", "You choose an opponent on announcement");
    assert_supported("Bargaining Table");
    let mut t = TestGame::new(3);
    let table = t.battlefield(P0, "Bargaining Table");
    t.lands(P0, "Plains", 4);
    for _ in 0..3 {
        t.hand(P1, "Island");
    }
    t.hand(P2, "Island");
    // X = 2 is neither opponent's hand size.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, table, 0, &[]).unwrap();
    let untapped = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.chars.is(types::CardType::Land) && !o.tapped)
            .count()
    };
    // The least legal value (P2's one card) was used.
    assert_eq!(untapped(&t), 3);
    t.resolve();
    // P1's three cards: X = 3 is legal.
    let mut t2 = TestGame::new(3);
    let table = t2.battlefield(P0, "Bargaining Table");
    t2.lands(P0, "Plains", 4);
    for _ in 0..3 {
        t2.hand(P1, "Island");
    }
    t2.hand(P2, "Island");
    t2.answer(P0, DecisionKind::X, Answer::Number(3));
    let hand = t2.hand_size(P0);
    t2.activate(P0, table, 0, &[]).unwrap();
    assert_eq!(untapped(&t2), 1);
    t2.resolve();
    assert_eq!(t2.hand_size(P0), hand + 1);
}
