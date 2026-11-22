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

#[test]
fn a_win_while_each_player_draws_ends_the_game_before_anyone_sacrifices() {
    cr!("104.1", "104.2b", "608.2e");
    // "Each player draws a card, then sacrifices all creatures they control." (The
    // sacrifice asks nothing, and is performed for all the players together.) P0's draw
    // wins the game: nobody sacrifices anything.
    use crate::r608_each_player_common::*;
    use mtg_engine::ability::*;
    let body = each_player(Effect::Seq(vec![
        draw(1),
        Effect::SacrificeObjects {
            what: Sel::All(creature_you_control()),
        },
    ]));
    let mut t = TestGame::new(2);
    let maniac = t.battlefield(P0, "Laboratory Maniac");
    t.g.players[0].library.clear();
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_body(&mut t, body);
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
    assert!(t.on_battlefield(maniac) && t.on_battlefield(bears));
}
