//! CR 121.1, 121.2: a player who draws "from the bottom of [their] library rather than the
//! top" (River Song's rule, the `draws_from_bottom` hook): each draw takes the bottom card
//! and is still a draw. Tested with a custom permanent carrying River Song's marker until
//! the card itself is supported.

use super::r600_common::CB;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const FROM_BOTTOM: &str = mtg_engine::cards::river_song::FROM_BOTTOM;

/// A permanent with "You draw cards from the bottom of your library rather than the top."
fn reverser(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let def = CB::new("Reverser")
        .creature(1, 1)
        .ability(mtg_engine::cards::marker(
            FROM_BOTTOM,
            "You draw cards from the bottom of your library rather than the top.",
        ))
        .build();
    t.custom(p, def, Zone::Battlefield)
}

/// Puts a card named `name` on the bottom of `p`'s library.
fn library_bottom(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.library_top(p, name);
    let lib = &mut t.g.players[p.idx()].library;
    lib.retain(|x| *x != id);
    lib.insert(0, id);
    id
}

#[test]
fn every_draw_takes_the_bottom_card_and_is_still_a_draw() {
    cr!("121.1", "121.2");
    let mut t = TestGame::new(2);
    reverser(&mut t, P0);
    library_bottom(&mut t, P0, "Shock");
    t.library_top(P0, "Island");
    let before = t.g.history.cards_drawn.get(&P0).copied().unwrap_or(0);
    let drawn = t.g.draw_cards(P0, 1);
    assert_eq!(t.obj_now(drawn[0]).chars.name, "Shock");
    // Counted as a card drawn.
    assert_eq!(t.g.history.cards_drawn.get(&P0).copied(), Some(before + 1));
    // The draw for the turn too.
    library_bottom(&mut t, P0, "Lightning Bolt");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(!t.in_hand(P0, "Island"));
}

#[test]
fn other_players_still_draw_from_the_top() {
    cr!("121.1");
    let mut t = TestGame::new(2);
    reverser(&mut t, P0);
    library_bottom(&mut t, P1, "Shock");
    t.library_top(P1, "Island");
    let drawn = t.g.draw_cards(P1, 1);
    assert_eq!(t.obj_now(drawn[0]).chars.name, "Island");
}
