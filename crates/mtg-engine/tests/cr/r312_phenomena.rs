//! CR 312: phenomena (Planechase) — nontraditional cards in the planar deck that trigger
//! when encountered and are then planeswalked away from.

use crate::r107_planechase::{add_planar_deck, face_up_names, planechase_game, roll};
use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::object::{CastMethod, StackKind, Zone};
use mtg_engine::planechase::{self, PlanarFace};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const EPIPHANY: &str = "Mutual Epiphany";

#[test]
fn phenomena_are_nontraditional_cards_used_only_in_planechase() {
    cr!("312.1");
    assert!(mtg_engine::variants::is_nontraditional(
        &mtg_engine::card::card(EPIPHANY)
    ));
    // Outside a Planechase game nobody planeswalks to it.
    let mut t = TestGame::new(2);
    add_planar_deck(&mut t, P0, &["Goldmeadow", EPIPHANY]);
    planechase::planeswalk(&mut t.g, P0);
    assert!(face_up_names(&t).is_empty());
}

#[test]
fn phenomenon_cards_remain_in_the_command_zone() {
    cr!("312.2");
    let mut t = planechase_game(2, false);
    let deck = add_planar_deck(&mut t, P0, &[EPIPHANY, "Goldmeadow"]);
    let e = deck[0];
    assert!(t.g.permanents().all(|o| o.id != e));
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, e, CastMethod::Normal).is_err());
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Exile),
        },
        &[Entity::Object(e)],
    );
    assert_eq!(t.g.current(e), e);
    assert_eq!(t.zone(e), Zone::Command);
}

#[test]
fn phenomena_have_no_subtypes() {
    cr!("312.3");
    assert!(subtypes_of(EPIPHANY).is_empty());
    assert!(subtypes_of("Planewide Disaster").is_empty());
}

#[test]
fn encountering_a_phenomenon_triggers_its_ability_for_the_planar_controller() {
    cr!("312.4", "312.5");
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow"]);
    add_planar_deck(&mut t, P1, &[EPIPHANY, "Krosa"]);
    planechase::set_starting_plane(&mut t.g);
    // On P1's turn P1 is the planar controller: they roll the Planeswalker symbol and
    // planeswalk, turning Mutual Epiphany face up off the top of their planar deck.
    t.set_step(P1, Step::PrecombatMain);
    t.g.recompute();
    roll(&mut t, P1, PlanarFace::Planeswalker);
    let hands = (t.hand_size(P0), t.hand_size(P1));
    t.resolve();
    assert_eq!(face_up_names(&t), vec![EPIPHANY]);
    let e = planechase::face_up_planar_cards(&t.g)[0];
    assert_eq!(t.obj(e).controller, P1);
    // "When you encounter Mutual Epiphany, each player draws four cards": P1's ability.
    assert_eq!(t.stack_len(), 1);
    let top = t.g.stack[0];
    assert_eq!(t.obj(top).controller, P1);
    assert!(matches!(
        t.obj(top).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { source, .. }) if *source == e
    ));
    t.resolve();
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P1)),
        (hands.0 + 4, hands.1 + 4)
    );
}

#[test]
fn a_phenomenon_turned_face_down_becomes_a_new_object() {
    cr!("312.6");
    let mut t = planechase_game(2, false);
    let deck = add_planar_deck(&mut t, P0, &[EPIPHANY, "Goldmeadow"]);
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, deck[0]);
    t.g.recompute();
    planechase::planeswalk(&mut t.g, P0);
    assert!(!t.g.is_live(deck[0]));
    let now = t.g.current(deck[0]);
    assert!(t.obj(now).face_down);
    assert_eq!(face_up_names(&t), vec!["Goldmeadow"]);
}

#[test]
fn the_planar_controller_planeswalks_away_from_a_phenomenon_once_its_trigger_is_gone() {
    cr!("312.7");
    supported(EPIPHANY);
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow", EPIPHANY, "Krosa"]);
    planechase::set_starting_plane(&mut t.g);
    roll(&mut t, P0, PlanarFace::Planeswalker);
    t.resolve();
    // Its encounter ability is on the stack: the phenomenon stays face up.
    t.settle();
    assert_eq!(face_up_names(&t), vec![EPIPHANY]);
    // Once the ability has left the stack, the planar controller planeswalks.
    t.resolve();
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
}
