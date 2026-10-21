//! CR 104.4a, 800.4a: players who lose at the same time leave the game together, whatever
//! their seats. Found by the seat symmetry property (docs/PROPERTIES.md): when several
//! players lost at once, only the first one in seat order had their objects leave the
//! game (the next ones found the game already over); in a subgame (CR 729) that sent the
//! main-game hand and permanents of the player in the lowest seat into their library.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn players_losing_at_once_are_treated_alike_whatever_their_seats() {
    cr!("104.2a", "800.4a");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.lose_game_simultaneously(&[P0, P1]);
    // P2 wins; the game is over, so neither loser's objects had to leave.
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P2])));
    assert!(t.g.player(P0).left_game && t.g.player(P1).left_game);
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
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

#[test]
fn players_losing_at_once_lose_in_apnap_order() {
    // Found by seat symmetry: players losing at once were handled in the order they were
    // listed, by seat, and what happened as the first of them lost (effects ending,
    // characteristics recomputed) depended on which one it was.
    cr!("101.4");
    let mut t = TestGame::new(4);
    t.set_step(P2, mtg_engine::turn::Step::PrecombatMain);
    let from = t.g.turn_events.len();
    t.g.lose_game_simultaneously(&[P0, P1, P3]);
    let lost: Vec<PlayerId> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            mtg_engine::events::Event::PlayerLost { player } => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(lost, vec![P3, P0, P1]);
}
