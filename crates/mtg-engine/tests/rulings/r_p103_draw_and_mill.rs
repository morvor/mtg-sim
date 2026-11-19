//! Rulings batch P103 — spells and abilities that mill and draw, followed in the order
//! written (CR 608.2c), and Jace, Wielder of Mysteries's replacement of a draw from an
//! empty library with winning the game (CR 614.1a, 104.2b, 121.4).

use crate::r_p103_common::*;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s25_common::cast_new;
use mtg_engine::game::GameResult;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn milling_yourself_then_drawing_follows_the_written_order() {
    cr!("608.2c", "701.17a");
    ruling!(
        "Thought Scour",
        "Follow the instructions in the order listed on the card: if you target yourself, you'll put the top two cards of your library into your graveyard and then draw a card."
    );
    ruling!(
        "Thassa's Bounty",
        "Follow the instructions in the order listed on the card: if you target yourself, you'll draw three cards, then put the top three cards of your library into your graveyard."
    );
    ruling!(
        "Pilfered Plans",
        "If you target yourself, you’ll put the top two cards of your library into your graveyard before drawing cards."
    );
    ruling!(
        "Jace, Wielder of Mysteries",
        "Follow the instructions in the order listed on Jace’s first loyalty ability: if you target yourself, you'll put the top two cards of your library into your graveyard and then draw a card."
    );
    // (card, how many of the top seven cards are milled first, drawn, milled after)
    for (name, milled_first, drawn, milled_after) in [
        ("Thought Scour", 2, 1, 0),
        ("Thassa's Bounty", 0, 3, 3),
        ("Pilfered Plans", 2, 2, 0),
        ("Jace, Wielder of Mysteries", 2, 1, 0),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let top = stack_library(
            &mut t,
            P0,
            &[
                "Island", "Forest", "Swamp", "Mountain", "Plains", "Wastes", "Island",
            ],
        );
        if name.starts_with("Jace") {
            let jace = t.battlefield(P0, name);
            t.activate(P0, jace, 0, &[pl(P0)]).unwrap();
        } else {
            cast_new(&mut t, P0, name, &[pl(P0)]);
        }
        t.resolve_all();
        let mut i = 0;
        for (n, zone) in [
            (milled_first, Zone::Graveyard(P0)),
            (drawn, Zone::Hand(P0)),
            (milled_after, Zone::Graveyard(P0)),
        ] {
            for c in &top[i..i + n] {
                assert_eq!(t.zone(*c), zone, "{name}: card {i}");
            }
            i += n;
        }
        assert_eq!(t.zone(top[i]), Zone::Library(P0), "{name}");
    }
}

#[test]
fn jace_cant_win_against_platinum_angel_but_the_draw_is_still_replaced() {
    cr!("614.1a", "614.6", "121.4", "104.2b", "101.2");
    ruling!(
        "Jace, Wielder of Mysteries",
        "If for some reason you can’t win the game (because your opponent controls Platinum Angel, for example), you won’t lose for having tried to draw a card from a library with no cards in it. The draw was still replaced."
    );
    supported("Jace, Wielder of Mysteries");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace, Wielder of Mysteries");
    t.battlefield(P1, "Platinum Angel");
    empty_library(&mut t, P0);
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert!(t.g.result.is_none());
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    // Without the Angel, the same draw wins the game.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace, Wielder of Mysteries");
    empty_library(&mut t, P0);
    t.g.draw_cards(P0, 1);
    t.settle();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn with_two_jaces_the_active_player_draws_and_wins_first() {
    cr!("101.4", "614.1a", "104.2b", "104.1");
    ruling!(
        "Jace, Wielder of Mysteries",
        "If two or more players control Jace, Wielder of Mysteries and each player is instructed to draw a number of cards, first the player whose turn it is draws that many cards. If this causes that player to win the game instead, the game is immediately over."
    );
    supported("Jace, Wielder of Mysteries");
    // On P1's turn, P0 casts Vision Skeins ("Each player draws two cards."): P1, the
    // active player, draws first and wins.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace, Wielder of Mysteries");
    t.battlefield(P1, "Jace, Wielder of Mysteries");
    empty_library(&mut t, P0);
    empty_library(&mut t, P1);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    cast_new(&mut t, P0, "Vision Skeins", &[]);
    t.resolve();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P1])));
    // On P0's turn P0 draws first and wins at once: P1, with a card left in its library,
    // never draws.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jace, Wielder of Mysteries");
    t.battlefield(P1, "Jace, Wielder of Mysteries");
    empty_library(&mut t, P0);
    library_of(&mut t, P1, "Island", 1);
    cast_new(&mut t, P0, "Vision Skeins", &[]);
    t.resolve();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
    assert_eq!(t.library_size(P1), 1, "P1 never drew");
}

#[test]
fn jaces_ultimate_wins_even_after_jace_left_and_the_library_ran_out() {
    cr!("608.2c", "104.3c", "704.3", "104.2b", "121.4");
    ruling!(
        "Jace, Wielder of Mysteries",
        "If your library has fewer than seven cards in it while resolving Jace’s last ability, and Jace has already left the battlefield, you’ll draw as many cards as you can and then win the game before state-based actions would cause you to lose the game for trying to draw from an empty library."
    );
    supported("Jace, Wielder of Mysteries");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace, Wielder of Mysteries");
    t.g.objects[jace.0 as usize]
        .counters
        .insert("loyalty".into(), 8);
    library_of(&mut t, P0, "Island", 3);
    t.activate(P0, jace, 1, &[]).unwrap();
    // The −8 left Jace with no loyalty: it's put into the graveyard before the ability
    // resolves.
    t.settle();
    assert_eq!(t.zone(jace), Zone::Graveyard(P0));
    let hand = t.hand_size(P0);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 3);
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
    assert!(!t.has_lost(P0));
}
