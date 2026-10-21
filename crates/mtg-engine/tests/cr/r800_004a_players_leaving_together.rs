//! CR 104.4a, 800.4a: players who lose at the same time leave the game together, whatever
//! their seats. Found by the seat symmetry property (docs/PROPERTIES.md): when several
//! players lost at once, only the first one in seat order had their objects leave the
//! game (the next ones found the game already over); in a subgame (CR 729) that sent the
//! main-game hand and permanents of the player in the lowest seat into their library.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn players_losing_at_once_are_treated_alike_whatever_their_seats() {
    cr!("104.4a", "800.4a");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.lose_game_simultaneously(&[P0, P1]);
    // P2 wins; the game is over, so neither loser's objects had to leave.
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P2])));
    assert!(t.g.player(P0).left_game && t.g.player(P1).left_game);
    assert_eq!(t.on_battlefield(a), t.on_battlefield(b));
}

#[test]
fn players_losing_at_once_in_a_game_that_goes_on_both_leave() {
    cr!("800.4a");
    let mut t = TestGame::new(4);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.lose_game_simultaneously(&[P0, P1]);
    assert!(t.g.result.is_none());
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}

#[test]
fn a_player_leaving_a_subgame_keeps_their_main_game_cards() {
    cr!("729.4", "729.5", "800.4a");
    let mut t = TestGame::new(3);
    let bears = t.hand(P2, "Grizzly Bears");
    let wurm = t.battlefield(P2, "Craw Wurm");
    let mut sub = mtg_engine::subgame::begin(&mut t.g);
    // P2 loses the subgame while the others play on: their cards outside it leave the
    // subgame with them, but they were never brought into it.
    sub.game.player_loses(P2);
    let _ = mtg_engine::subgame::finish(&mut t.g, sub);
    assert_eq!(t.g.obj(bears).zone, mtg_engine::object::Zone::Hand(P2));
    assert!(t.on_battlefield(wurm));
}

#[test]
fn players_losing_a_subgame_together_keep_their_main_game_cards() {
    cr!("729.5", "104.4a");
    let mut t = TestGame::new(3);
    let cards: Vec<ObjectId> = [P0, P1, P2]
        .iter()
        .map(|p| t.hand(*p, "Grizzly Bears"))
        .collect();
    let mut sub = mtg_engine::subgame::begin(&mut t.g);
    sub.game.lose_game_simultaneously(&[P0, P1, P2]);
    assert_eq!(sub.game.result, Some(GameResult::Draw));
    let _ = mtg_engine::subgame::finish(&mut t.g, sub);
    for (c, p) in cards.iter().zip([P0, P1, P2]) {
        assert_eq!(t.g.obj(*c).zone, mtg_engine::object::Zone::Hand(p));
    }
}
