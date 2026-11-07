//! Agyrem (compiled in rulings batch P122): "Whenever chaos ensues, creatures can't attack
//! you until a player planeswalks." "You" is the plane's controller, the planar controller
//! (CR 901.6), as the chaos ability resolves; the effect ends when a player planeswalks
//! (CR 901.11) and applies to creatures that arrive later (CR 611.2c doesn't lock it).
//! Its first two abilities are exercised too.

use crate::r703_common::supported;
use crate::r900_common::*;
use mtg_engine::planechase::{self, PlanarFace};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn agyrem_chaos_creatures_cant_attack_you_until_a_player_planeswalks() {
    cr!("901.6", "901.11", "611.2c");
    supported("Agyrem");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Agyrem"]);
    add_planar_deck(&mut t, P1, &["Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up_names(&t), vec!["Agyrem"]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    roll_effect(&mut t, P0, PlanarFace::Chaos);
    t.resolve_all();
    // On P1's turn (P1 is now the planar controller), P1's creatures still can't attack
    // P0, the planar controller when the ability resolved; nor can one arriving later.
    t.set_step(P1, Step::BeginningOfCombat);
    let goblin = t.enter(P1, "Raging Goblin");
    assert!(!t.g.can_attack_target(bears, Entity::Player(P0)));
    assert!(!t.g.can_attack_target(goblin, Entity::Player(P0)));
    // A player planeswalks: the effect ends.
    planechase::planeswalk(&mut t.g, P1);
    t.settle();
    t.resolve_all();
    assert!(t.g.can_attack_target(bears, Entity::Player(P0)));
    assert!(t.g.can_attack_target(goblin, Entity::Player(P0)));
}

#[test]
fn agyrem_returns_dead_creatures_at_the_next_end_step() {
    cr!("603.7a");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Agyrem"]);
    planechase::set_starting_plane(&mut t.g);
    t.set_step(P0, Step::PrecombatMain);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(lions, None);
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Savannah Lions"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // The white creature returns to the battlefield, the nonwhite one to its owner's hand.
    assert!(t.g.permanents().any(|o| o.chars.name.as_str() == "Savannah Lions"));
    assert!(t
        .g
        .players[P1.idx()]
        .hand
        .iter()
        .any(|&c| t.g.obj(c).chars.name.as_str() == "Grizzly Bears"));
}
