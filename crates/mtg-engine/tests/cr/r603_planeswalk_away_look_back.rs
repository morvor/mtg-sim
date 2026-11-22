//! CR 603.10g: abilities that trigger when a player planeswalks away from a plane look back
//! in time — they trigger from the plane as it was just before it was turned face down (or
//! left the game), although it has no abilities once that has happened.

use crate::r107_planechase::{add_planar_deck, face_up_names, planechase_game};
use crate::r703_common::supported;
use mtg_engine::card::card;
use mtg_engine::decision::Action;
use mtg_engine::planechase;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The triggered abilities waiting to be put on the stack or on it whose source is `src`.
fn triggers_from(t: &TestGame, src: ObjectId) -> usize {
    let pending =
        t.g.pending_triggers
            .iter()
            .filter(|p| p.source == src)
            .count();
    let stacked =
        t.g.stack
            .iter()
            .filter(|s| t.g.ability_source_of(**s) == src && **s != src)
            .count();
    pending + stacked
}

#[test]
fn planeswalking_away_triggers_from_the_plane_as_it_was_before_it_turned_face_down() {
    cr!("603.10", "603.10g");
    supported("Sanctum of Serra");
    let mut t = planechase_game(2, false);
    // Sanctum of Serra: "When you planeswalk away from Sanctum of Serra, destroy all
    // nonland permanents."
    let deck = add_planar_deck(&mut t, P0, &["Sanctum of Serra", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up_names(&t), vec!["Sanctum of Serra"]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    let forest = t.battlefield(P1, "Forest");
    planechase::planeswalk(&mut t.g, P0);
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
    // Right after the event, the Sanctum is a face-down card at the bottom of the planar
    // deck with no abilities: looking at the game after the event, nothing would trigger.
    let now = t.g.current(deck[0]);
    assert!(t.obj(now).face_down);
    assert!(t.obj(now).chars.abilities.is_empty());
    // The game looks back in time: the ability of the face-up plane triggered.
    assert_eq!(triggers_from(&t, deck[0]), 1);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(ring));
    assert!(t.on_battlefield(forest));
}

#[test]
fn planeswalking_away_uses_the_planes_appearance_just_before_the_event() {
    cr!("603.10", "603.10g", "608.2h");
    // (Its chaos ability isn't supported; the one under test is.)
    let keralia = card("Mount Keralia");
    let unsupported = keralia.unsupported_text();
    assert!(
        unsupported.iter().all(|a| !a.contains("planeswalk away")),
        "{unsupported:?}"
    );
    let mut t = planechase_game(2, false);
    // Mount Keralia: "When you planeswalk away from Mount Keralia, it deals damage equal
    // to the number of pressure counters on it to each creature and each planeswalker."
    let deck = add_planar_deck(&mut t, P0, &["Mount Keralia", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    t.g.add_counters(Entity::Object(deck[0]), "pressure", 3, None);
    assert_eq!(t.counters(deck[0], "pressure"), 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    planechase::planeswalk(&mut t.g, P0);
    // Turned face down, it's a new object with no counters; the ability uses the plane
    // as it last existed face up, with three pressure counters.
    assert_eq!(t.counters(t.g.current(deck[0]), "pressure"), 0);
    assert_eq!(triggers_from(&t, deck[0]), 1);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj(wurm).damage, 3);
}

#[test]
fn planeswalking_away_from_a_plane_that_leaves_the_game_with_its_owner() {
    cr!("603.10g");
    supported("Sanctum of Serra");
    ruling!(
        "Sanctum of Serra",
        "If you planeswalk away from Sanctum of Serra because the player who owns Sanctum of Serra leaves the game, its first ability will still trigger and resolve."
    );
    let mut t = planechase_game(3, false);
    // P1 owns the face-up Sanctum of Serra; P0, the planar controller, has Goldmeadow on
    // top of their planar deck.
    let theirs = add_planar_deck(&mut t, P1, &["Sanctum of Serra"]);
    add_planar_deck(&mut t, P0, &["Goldmeadow"]);
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, theirs[0]);
    t.g.recompute();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    let island = t.battlefield(P2, "Island");
    // P1 leaves the game: the Sanctum leaves the game with them (it no longer exists), and
    // P0 planeswalks away from it to Goldmeadow. Its ability still triggers and resolves.
    t.g.turn.priority = Some(P1);
    t.g.perform_action(P1, Action::Concede).unwrap();
    assert!(!t.g.is_live(theirs[0]));
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
    assert_eq!(triggers_from(&t, theirs[0]), 1);
    // P0, who planeswalked away from it, controls the trigger.
    t.settle();
    assert!(t.g.stack.iter().any(|s| t.g.ability_source_of(*s) == theirs[0]
        && t.obj(*s).controller == P0));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(giant));
    assert!(t.on_battlefield(island));
}
