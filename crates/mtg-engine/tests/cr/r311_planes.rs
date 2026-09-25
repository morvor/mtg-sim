//! CR 311: planes (Planechase) — nontraditional cards that stay in the command zone,
//! planar types, their abilities while face up, the planar controller, and chaos
//! abilities.

use crate::r107_planechase::{add_planar_deck, face_up_names, planechase_game, roll};
use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Action;
use mtg_engine::object::{CastMethod, StackKind, Zone};
use mtg_engine::planechase::{self, PlanarFace};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A custom plane card (face down in `owner`'s planar deck, on top).
fn custom_plane(t: &mut TestGame, owner: PlayerId, name: &str, text: &str) -> ObjectId {
    let def = oracle_card(name, "Plane — Test Realm", "", None, text);
    let id = t.custom(owner, def, Zone::Command);
    t.g.objects[id.0 as usize].face_down = true;
    // On top of the planar deck.
    t.g.command.retain(|x| *x != id);
    t.g.command.insert(0, id);
    t.g.recompute();
    id
}

#[test]
fn planes_are_nontraditional_cards_used_only_in_planechase() {
    cr!("311.1");
    assert!(mtg_engine::variants::is_nontraditional(
        &mtg_engine::card::card("Goldmeadow")
    ));
    // Outside a Planechase game there's no planar controller and no plane is turned
    // face up.
    let mut t = TestGame::new(2);
    add_planar_deck(&mut t, P0, &["Krosa", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    assert!(face_up_names(&t).is_empty());
    assert_eq!(planechase::planar_controller(&t.g), None);
    // In a Planechase game, the starting plane is turned face up.
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Krosa", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
}

#[test]
fn plane_cards_remain_in_the_command_zone() {
    cr!("311.2");
    let mut t = planechase_game(2, false);
    let deck = add_planar_deck(&mut t, P0, &["Krosa", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    let krosa = deck[0];
    // Neither the face-up plane nor one in the planar deck is a permanent or can be cast.
    for id in [krosa, deck[1]] {
        assert_eq!(t.zone(id), Zone::Command);
        assert!(t.g.permanents().all(|o| o.id != id));
        t.g.turn.priority = Some(P0);
        assert!(t.g.cast_spell(P0, id, CastMethod::Normal).is_err());
    }
    // If a plane card would leave the command zone, it remains there.
    for to in [
        Destination::zone(ZoneKind::Hand),
        Destination::zone(ZoneKind::Graveyard),
        Destination::battlefield(),
    ] {
        run_effect(
            &mut t,
            P0,
            None,
            Effect::Move {
                what: Sel::Target(0),
                to,
            },
            &[Entity::Object(krosa)],
        );
        assert_eq!(t.g.current(krosa), krosa);
        assert_eq!(t.zone(krosa), Zone::Command);
    }
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
}

#[test]
fn a_planar_type_is_every_word_after_the_dash() {
    cr!("311.3");
    // "Plane — Thunder Junction": one subtype.
    assert_eq!(subtypes_of("Tarnation"), vec!["Thunder Junction"]);
    assert_eq!(subtypes_of("Krosa"), vec!["Dominaria"]);
    let tl = TypeLine::parse("Plane — Serra's Realm");
    assert_eq!(tl.subtypes, vec!["Serra's Realm"]);
}

#[test]
fn a_face_up_planes_abilities_function_from_the_command_zone() {
    cr!("311.4");
    let mut t = planechase_game(2, false);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // A plane with an activated ability.
    let shrine = custom_plane(&mut t, P0, "Shrine Realm", "{1}: You gain 2 life.");
    add_planar_deck(&mut t, P0, &["Krosa", "Goldmeadow"]);
    // While it's face down in the planar deck, Krosa's "All creatures get +2/+2" doesn't
    // apply.
    assert_eq!(t.pt(bears), (2, 2));
    planechase::set_starting_plane(&mut t.g);
    assert_eq!(face_up_names(&t), vec!["Shrine Realm"]);
    // Its activated ability can be activated by its controller.
    t.lands(P0, "Plains", 1);
    t.activate(P0, shrine, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 22);
    // Planeswalk to Krosa: its static ability affects the game.
    planechase::planeswalk(&mut t.g, P0);
    t.g.recompute();
    assert_eq!(face_up_names(&t), vec!["Krosa"]);
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(theirs), (4, 4));
    // And to Goldmeadow: its triggered abilities trigger ("Whenever a land enters, that
    // land's controller creates three 0/1 white Goat creature tokens").
    planechase::planeswalk(&mut t.g, P0);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Goat Token").len(), 3);
}

#[test]
fn the_planar_controller_controls_the_face_up_plane() {
    cr!("311.5");
    let mut t = planechase_game(3, false);
    // P1's plane, face up during P0's turn.
    let deck = add_planar_deck(&mut t, P1, &["Krosa"]);
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, deck[0]);
    t.g.recompute();
    assert_eq!(planechase::planar_controller(&t.g), Some(P0));
    assert_eq!(t.obj(deck[0]).controller, P0);
    // When the active player changes, so does the planar controller.
    t.set_step(P2, Step::Upkeep);
    t.g.recompute();
    assert_eq!(t.obj(deck[0]).controller, P2);
    // If the planar controller leaves the game, the next player in turn order becomes the
    // planar controller.
    t.set_step(P1, Step::PrecombatMain);
    t.set_step(P2, Step::PrecombatMain);
    t.g.perform_action(P2, Action::Concede).unwrap();
    t.g.recompute();
    assert!(!t.player(P2).in_game());
    assert_eq!(planechase::planar_controller(&t.g), Some(P0));
    assert_eq!(t.obj(deck[0]).controller, P0);
}

#[test]
fn a_plane_turned_face_down_becomes_a_new_object() {
    cr!("311.6");
    let mut t = planechase_game(2, false);
    let deck = add_planar_deck(&mut t, P0, &["Krosa", "Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    let krosa = deck[0];
    planechase::planeswalk(&mut t.g, P0);
    // Krosa went face down to the bottom of the planar deck as a new object.
    assert!(!t.g.is_live(krosa));
    let now = t.g.current(krosa);
    assert_ne!(now, krosa);
    assert!(t.obj(now).face_down);
    assert_eq!(*planechase::planar_deck(&t.g, P0).last().unwrap(), now);
}

#[test]
fn chaos_abilities_trigger_when_chaos_ensues_and_are_the_planar_controllers() {
    cr!("311.7");
    // When the chaos symbol is rolled.
    let mut t = planechase_game(2, false);
    add_planar_deck(&mut t, P0, &["Goldmeadow"]);
    planechase::set_starting_plane(&mut t.g);
    roll(&mut t, P0, PlanarFace::Chaos);
    let top = *t.g.stack.last().unwrap();
    assert!(matches!(
        t.obj(top).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { .. })
    ));
    assert_eq!(t.obj(top).controller, P0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Goat Token").len(), 1);
    // When a resolving ability says that chaos ensues: a plane whose upkeep trigger makes
    // chaos ensue, on P1's turn (P1 is the planar controller).
    let mut t = planechase_game(2, false);
    custom_plane(
        &mut t,
        P0,
        "Unstable Realm",
        "At the beginning of your upkeep, chaos ensues.\nWhenever chaos ensues, you gain 2 life.",
    );
    planechase::set_starting_plane(&mut t.g);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 22);
    assert_eq!(t.life(P0), 20);
}
