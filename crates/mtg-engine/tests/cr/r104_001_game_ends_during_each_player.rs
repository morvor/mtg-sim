//! CR 104.1: the game ends immediately, also in the middle of an "each player"
//! instruction (CR 608.2e): the instructions after the one that ended it aren't performed.

use mtg_engine::game::GameResult;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_win_while_each_player_draws_ends_the_game_before_they_discard() {
    cr!("104.1", "104.2b", "608.2e");
    let mut t = TestGame::new(2);
    // "If you would draw a card while your library has no cards in it, you win the game
    // instead."
    t.battlefield(P0, "Laboratory Maniac");
    t.g.players[0].library.clear();
    // "Each player draws three cards, then discards three cards at random."
    let inquiry = t.hand(P0, "Burning Inquiry");
    t.lands(P0, "Mountain", 1);
    t.cast(P0, inquiry).go();
    t.resolve();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
    // Nobody discarded: the game ended with the draws.
    assert_eq!(t.graveyard_size(P1), 0);
}

#[test]
fn a_win_while_each_player_draws_ends_the_game_before_anyone_chooses_a_discard() {
    cr!("104.1", "104.2b", "608.2e");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laboratory Maniac");
    t.g.players[0].library.clear();
    // "{T}: Each player draws a card, then discards a card." The discard is a choice each
    // player makes, planned for all of them together; once P0 has won nobody makes it.
    let broker = t.battlefield(P0, "Lore Broker");
    t.hand(P1, "Island");
    t.activate(P0, broker, 0, &[]).expect("activate Lore Broker");
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.in_hand(P1, "Island"));
}
