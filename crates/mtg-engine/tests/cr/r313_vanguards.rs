//! CR 313: vanguards (the Vanguard variant) — nontraditional cards that stay face up in
//! the command zone, their abilities, ownership and control, and hand and life
//! modifiers.

use crate::r100_common::{fillers, pregame};
use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn vanguard_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Vanguard,
            ..Default::default()
        },
    )
}

/// A Vanguard game (not yet started) where P0's deck lists `vanguard`.
fn vanguard_pregame(variant: Variant, vanguard: &str) -> TestGame {
    let mut deck = fillers(40);
    deck.push(card(vanguard));
    pregame(
        GameConfig {
            variant,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![deck, fillers(40)],
    )
}

#[test]
fn vanguards_are_nontraditional_cards_used_in_the_vanguard_variant() {
    cr!("313.1");
    assert!(mtg_engine::variants::is_nontraditional(&card("Titania")));
    // Titania: life modifier -5. It modifies its owner's starting life in a Vanguard
    // game, not in a normal game.
    let t = vanguard_pregame(Variant::Vanguard, "Titania");
    assert_eq!(t.g.starting_life(P0), 15);
    let t = vanguard_pregame(Variant::Standard, "Titania");
    assert_eq!(t.g.starting_life(P0), 20);
}

#[test]
fn vanguard_cards_remain_in_the_command_zone() {
    cr!("313.2");
    let mut t = vanguard_pregame(Variant::Vanguard, "Titania");
    t.g.start();
    let v = t.g.vanguard_of(P0).expect("vanguard");
    assert_eq!(t.zone(v), Zone::Command);
    assert!(!t.obj(v).face_down);
    // Not in the library, not a permanent, can't be cast.
    assert!(t.g.player(P0).library.iter().all(|c| *c != v));
    assert!(t.g.permanents().all(|o| o.id != v));
    t.set_step(P0, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, v, CastMethod::Normal).is_err());
    // If it would leave the command zone, it remains there.
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Exile),
        },
        &[Entity::Object(v)],
    );
    assert_eq!(t.g.current(v), v);
    assert_eq!(t.zone(v), Zone::Command);
}

#[test]
fn vanguard_cards_have_no_subtypes() {
    cr!("313.3");
    for v in ["Titania", "Urza", "Maraxus", "Royal Assassin Avatar"] {
        assert!(subtypes_of(v).is_empty(), "{v}");
    }
}

#[test]
fn a_vanguards_abilities_function_from_the_command_zone() {
    cr!("313.4");
    let mut t = vanguard_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // Static: Maraxus — "Creatures you control get +1/+0."
    t.command(P0, "Maraxus");
    assert_eq!(t.pt(bears), (3, 2));
    assert_eq!(t.pt(theirs), (2, 2));
    // Activated: Urza — "{3}: Urza deals 1 damage to any target."
    let urza = t.command(P0, "Urza");
    t.lands(P0, "Wastes", 3);
    t.activate(P0, urza, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    // Triggered: Royal Assassin Avatar — "At the beginning of your upkeep, you draw a card
    // and you lose 1 life."
    t.command(P1, "Royal Assassin Avatar");
    let hand = t.hand_size(P1);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn a_vanguard_is_owned_and_controlled_by_the_player_who_started_with_it() {
    cr!("313.5");
    let mut t = vanguard_pregame(Variant::Vanguard, "Maraxus");
    t.g.start();
    let v = t.g.vanguard_of(P0).expect("vanguard");
    assert_eq!((t.obj(v).owner, t.obj(v).controller), (P0, P0));
    // An effect trying to give another player control of it doesn't.
    t.set_step(P0, Step::PrecombatMain);
    run_effect(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(v)],
    );
    assert_eq!(t.obj(v).controller, P0);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(mine), (3, 2));
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn the_hand_modifier_applies_to_starting_and_maximum_hand_size() {
    cr!("313.6");
    // Titania: hand modifier +2.
    let mut t = vanguard_pregame(Variant::Vanguard, "Titania");
    t.g.start();
    assert_eq!(t.hand_size(P0), 9);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.player(P0).max_hand_size, Some(9));
    assert_eq!(t.player(P1).max_hand_size, Some(7));
    // At the cleanup step, P0 discards down to nine cards.
    let mut t = vanguard_game();
    t.command(P0, "Titania");
    for _ in 0..12 {
        t.hand(P0, "Grizzly Bears");
    }
    let n = t.hand_size(P0);
    assert!(n >= 12);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 9);
}

#[test]
fn the_life_modifier_applies_to_the_starting_life_total() {
    cr!("313.7");
    // Titania: life modifier -5; Urza: +10.
    let mut t = vanguard_pregame(Variant::Vanguard, "Titania");
    t.g.start();
    assert_eq!(t.life(P0), 15);
    assert_eq!(t.life(P1), 20);
    let mut t = vanguard_pregame(Variant::Vanguard, "Urza");
    t.g.start();
    assert_eq!(t.life(P0), 30);
}
