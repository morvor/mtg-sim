//! Rulings batch S33 — revealing the top cards of a library: "reveal the top three cards
//! of your library" reveals all of them if the library has three or fewer (CR 701.20a);
//! every revealed card of the named kind is put into the hand, the rest where the text
//! says.

use crate::r_s01_common::{stack_library, supported};
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Leaves `p` with exactly the cards `top_first` in their library (the others cease to
/// exist).
fn only_library(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    let lib = t.g.players[p.idx()].library.clone();
    for id in lib {
        t.g.objects[id.0 as usize].zone = Zone::Nowhere;
    }
    t.g.players[p.idx()].library.clear();
    stack_library(t, p, top_first)
}

/// The cards revealed since index `from` of this turn's events.
fn revealed_since(t: &TestGame, from: usize) -> Vec<ObjectId> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            mtg_engine::events::Event::Custom { name, obj, .. }
                if name == mtg_engine::reveal::REVEALED =>
            {
                *obj
            }
            _ => None,
        })
        .collect()
}

#[test]
fn beast_hunt_with_three_or_fewer_cards_you_reveal_all_of_them() {
    cr!("701.20a", "609.3");
    ruling!(
        "Beast Hunt",
        "If there are three or fewer cards in your library, you’ll reveal all of them."
    );
    supported("Beast Hunt");
    // "Reveal the top three cards of your library. Put all creature cards revealed this
    // way into your hand and the rest into your graveyard."
    let mut t = TestGame::new(2);
    let cards = only_library(&mut t, P0, &["Grizzly Bears", "Forest"]);
    let from = t.g.turn_events.len();
    cast_and_resolve(&mut t, P0, "Beast Hunt", &[]);
    let revealed = revealed_since(&t, from);
    assert!(cards.iter().all(|c| revealed.contains(c)));
    assert_eq!(t.library_size(P0), 0);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Forest"));
    // With more cards, only the top three; both creatures go to the hand.
    let mut t = TestGame::new(2);
    only_library(
        &mut t,
        P0,
        &["Hill Giant", "Island", "Grizzly Bears", "Llanowar Elves"],
    );
    cast_and_resolve(&mut t, P0, "Beast Hunt", &[]);
    assert!(t.in_hand(P0, "Hill Giant") && t.in_hand(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Island"));
    assert_eq!(
        library_top_first(&t, P0)
            .iter()
            .map(|c| t.obj_now(*c).chars.name.to_string())
            .collect::<Vec<_>>(),
        vec!["Llanowar Elves"]
    );
}

#[test]
fn merfolk_wayfinder_with_three_or_fewer_cards_you_reveal_all_of_them() {
    cr!("701.20a", "603.6a");
    ruling!(
        "Merfolk Wayfinder",
        "If there are three or fewer cards in your library, you’ll reveal all of them."
    );
    supported("Merfolk Wayfinder");
    // "When this creature enters, reveal the top three cards of your library. Put all
    // Island cards revealed this way into your hand and the rest on the bottom of your
    // library in any order."
    let mut t = TestGame::new(2);
    let cards = only_library(&mut t, P0, &["Island", "Grizzly Bears"]);
    let from = t.g.turn_events.len();
    cast_and_resolve(&mut t, P0, "Merfolk Wayfinder", &[]);
    let revealed = revealed_since(&t, from);
    assert!(cards.iter().all(|c| revealed.contains(c)));
    assert!(t.in_hand(P0, "Island"));
    assert_eq!(library_top_first(&t, P0), vec![t.g.current(cards[1])]);
}
