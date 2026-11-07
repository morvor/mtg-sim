//! CR 108.3a, 109.4d: ownership and control of plane cards (Planechase, CR 901).

use super::r105_util::*;
use super::r107_planechase::*;
use mtg_engine::planechase::{self, PlanarFace};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_face_up_plane_is_controlled_by_the_planar_controller() {
    cr!("109.4d");
    let mut t = planechase_game(2, false);
    let deck = add_planar_deck(&mut t, P0, &["Goldmeadow", "The Fourth Sphere"]);
    planechase::set_starting_plane(&mut t.g);
    let plane = deck[0];
    // On P0's turn, P0 (the active player) is the planar controller and controls it.
    assert_eq!(planechase::planar_controller(&t.g), Some(P0));
    assert_eq!(t.obj(plane).controller, P0);
    roll(&mut t, P0, PlanarFace::Chaos);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0).len(), 1);
    // On P1's turn, P1 controls it although P0 owns it: its chaos ability is P1's.
    t.set_step(P1, Step::PrecombatMain);
    t.g.recompute();
    assert_eq!(planechase::planar_controller(&t.g), Some(P1));
    assert_eq!(t.obj(plane).controller, P1);
    assert_eq!(t.obj(plane).owner, P0);
    roll(&mut t, P1, PlanarFace::Chaos);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P1).len(), 1);
    assert_eq!(tokens_of(&t, P0).len(), 1);
}

#[test]
fn a_planes_you_is_the_planar_controller() {
    cr!("109.4d");
    let mut t = planechase_game(2, false);
    // The Fourth Sphere: "At the beginning of your upkeep, sacrifice a nonblack creature."
    add_planar_deck(&mut t, P0, &["The Fourth Sphere", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    let p0_bears = t.battlefield(P0, "Grizzly Bears");
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    // It triggers at the beginning of P1's upkeep, when P1 controls it.
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(p1_bears));
    assert!(t.on_battlefield(p0_bears));
}

#[test]
fn with_a_single_planar_deck_the_planar_controller_owns_its_cards() {
    cr!("108.3a");
    let mut t = planechase_game(3, true);
    // P1 and P2 bring cards to the communal planar deck.
    add_planar_deck(&mut t, P1, &["Goldmeadow", "Panopticon"]);
    add_planar_deck(&mut t, P2, &["The Fourth Sphere", "Krosa"]);
    planechase::set_starting_plane(&mut t.g);
    let cards: Vec<ObjectId> = planechase::face_up_planar_cards(&t.g)
        .into_iter()
        .chain(planechase::planar_deck(&t.g, P0))
        .collect();
    assert_eq!(cards.len(), 4);
    // On P0's turn, P0 (the planar controller) owns them all.
    assert!(cards.iter().all(|c| t.obj(*c).owner == P0));
    // So when P2 leaves the game, the cards P2 brought don't leave with P2.
    t.g.player_loses(P2);
    let after = planechase::face_up_planar_cards(&t.g).len() + planechase::planar_deck(&t.g, P0).len();
    assert_eq!(after, 4);
    // On P1's turn, P1 owns them.
    t.set_step(P1, Step::PrecombatMain);
    t.g.recompute();
    assert!(planechase::planar_deck(&t.g, P1)
        .iter()
        .all(|c| t.obj(*c).owner == P1));
}
