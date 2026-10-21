//! Seat symmetry: games that seat the same decks differently are identical up to
//! renaming players and objects (see `mtg_engine::properties::seat_symmetry`).

use super::common::*;
use mtg_engine::properties::*;
use mtg_engine::*;

/// Every rotation of the seats of game `i` of the fixed set, but one for four players (to
/// stay fast).
fn check(i: usize) {
    let s = &fixed_set()[i];
    let n = s.players();
    let rotations: Vec<usize> = if n == 4 { vec![2] } else { (1..n).collect() };
    for r in rotations {
        seat_symmetry(s, r).unwrap_or_else(|e| panic!("seed {}: {e}", s.seed));
    }
}

#[test]
fn swapping_the_seats_of_two_players_changes_nothing() {
    // Simultaneous choices and actions in APNAP order (CR 101.4), simultaneous timestamps
    // in APNAP order (CR 613.7m), turn order from the starting player (CR 103.1).
    cr!("101.4", "613.7m", "103.1");
    check(0);
    check(1);
}

#[test]
fn rotating_the_seats_of_three_players_changes_nothing() {
    cr!("101.4", "613.7m", "103.1");
    check(2);
}

#[test]
fn rotating_the_seats_of_four_players_changes_nothing() {
    cr!("101.4", "613.7m", "103.1");
    check(3);
}

#[test]
fn per_player_randomness_follows_the_deck_not_the_seat() {
    // A harness option, not a rule: with per-player streams, each deck is shuffled alike wherever it sits.
    let s = setup(vec![red_green(), white_blue()], 9);
    let mut a = build(&s, 0, Default::default());
    let mut b = build(&s, 1, Default::default());
    a.step();
    b.step();
    let names = |g: &Game, p: PlayerId| -> Vec<String> {
        g.player(p)
            .library
            .iter()
            .chain(&g.player(p).hand)
            .map(|o| g.obj(*o).chars.name.to_string())
            .collect()
    };
    assert_eq!(names(&a.game, PlayerId(0)), names(&b.game, PlayerId(1)));
    assert_eq!(names(&a.game, PlayerId(1)), names(&b.game, PlayerId(0)));
}

#[test]
fn the_comparison_notices_different_games() {
    // Not a vacuous check: the same decks with another seed are a different game.
    let s = setup(vec![red_green(), white_blue()], 5);
    let mut other = s.clone();
    other.seed = 6;
    let mut a = build(&s, 0, Recording { states: true, ..Default::default() });
    let mut b = build(&other, 1, Recording { states: true, ..Default::default() });
    a.finish();
    b.finish();
    assert!(compare("different seeds", &a.lines(), &b.lines()).is_err());
}

#[test]
fn a_subgame_is_seat_symmetric() {
    // CR 729.2: who goes first in a subgame is determined at random; found by this
    // property, the engine drew among the players in seat order.
    cr!("729.2", "101.4");
    let shahrazad = || deck("16 Plains, 4 Shahrazad, 10 Savannah Lions, 10 Serra Angel");
    let mut s = setup(vec![shahrazad(), shahrazad(), shahrazad()], 21);
    s.max_actions = 3000;
    let rec = Recording {
        states: true,
        ..Default::default()
    };
    // Subgames nest: play on a thread with room for them.
    let subgames = std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let mut subgames = 0;
            for r in [2] {
                let mut a = build(&s, 0, rec);
                a.finish();
                let mut b = build(&s, r, rec);
                b.finish();
                subgames += a.game.subgames.played;
                compare("subgames", &a.lines(), &b.lines()).unwrap();
            }
            subgames
        })
        .unwrap()
        .join()
        .unwrap();
    assert!(subgames > 0, "a subgame was played");
}

#[test]
fn triggers_of_several_players_go_on_the_stack_in_apnap_order_whatever_the_seats() {
    // "Whenever another creature enters, you gain 1 life": a creature entering triggers
    // the Soul Wardens of every player at once, and the stack holds them in APNAP order
    // (CR 603.3b), not in seat order.
    cr!("603.3b", "101.4");
    let warden = || deck("17 Plains, 8 Soul Warden, 8 Savannah Lions, 7 Serra Angel");
    let s = setup(vec![warden(), warden(), warden()], 7);
    for r in [1, 2] {
        seat_symmetry(&s, r).unwrap_or_else(|e| panic!("seed {}: {e}", s.seed));
    }
}
