//! CR 902: the Vanguard casual variant — a face-up vanguard card for each player, whose
//! life and hand modifiers change the player's starting life, starting hand size and
//! maximum hand size, and whose abilities function from the command zone.

use crate::r100_common::{fillers, pregame};
use crate::r703_common::run_effect;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A Vanguard game (not started) of `vanguards.len()` players, each with a 40-card deck
/// listing their vanguard card (if any).
fn vanguard_pregame(vanguards: &[Option<&str>], mulligans: bool) -> TestGame {
    let decks = vanguards
        .iter()
        .map(|v| {
            let mut d = fillers(40);
            if let Some(v) = v {
                d.push(card(v));
            }
            d
        })
        .collect();
    pregame(
        GameConfig {
            skip_mulligans: !mulligans,
            starting_player: Some(P0),
            ..GameConfig::vanguard_game()
        },
        decks,
    )
}

#[test]
fn each_player_plays_a_character_through_a_face_up_vanguard() {
    cr!("902.1");
    // Maraxus: "Creatures you control get +1/+0." (hand +1, life +2); Serra: "Creatures
    // you control get +0/+2." (hand +1, life +1).
    let mut t = vanguard_pregame(&[Some("Maraxus"), Some("Serra")], false);
    t.g.start();
    let (m, s) = (t.g.vanguard_of(P0).unwrap(), t.g.vanguard_of(P1).unwrap());
    assert!(!t.obj(m).face_down && !t.obj(s).face_down);
    assert_eq!(
        (t.obj(m).chars.name.as_str(), t.obj(s).chars.name.as_str()),
        ("Maraxus", "Serra")
    );
    // Its characteristics affect the game — the modifiers — and so do its abilities.
    assert_eq!((t.life(P0), t.life(P1)), (22, 21));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (8, 8));
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(a), (3, 2));
    assert_eq!(t.pt(b), (2, 4));
    // Otherwise the normal rules apply: the game goes on as usual.
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.hand_size(P1), 9);
}

#[test]
fn vanguard_is_two_player_or_multiplayer() {
    cr!("902.2");
    let c = GameConfig::vanguard_game();
    assert_eq!(c.validate(2), Ok(()));
    assert_eq!(c.validate(4), Ok(()));
    // A four-player Vanguard game: each player's own vanguard modifies their life.
    let mut t = vanguard_pregame(
        &[Some("Titania"), Some("Urza"), None, Some("Royal Assassin Avatar")],
        false,
    );
    t.g.start();
    assert_eq!(
        (t.life(P0), t.life(P1), t.life(P2), t.life(P3)),
        (15, 30, 20, 20)
    );
}

#[test]
fn vanguard_cards_start_face_up_in_the_command_zone_and_stay_there() {
    cr!("902.3");
    let mut t = vanguard_pregame(&[Some("Urza"), Some("Titania")], false);
    let v = t
        .g
        .command
        .iter()
        .copied()
        .find(|id| t.obj(*id).owner == P0)
        .expect("vanguard");
    // Before the game begins, it's already face up in the command zone.
    assert_eq!(t.zone(v), Zone::Command);
    assert!(!t.obj(v).face_down);
    t.g.start();
    assert!(t.g.player(P0).library.iter().all(|c| *c != v));
    assert_eq!(t.g.vanguard_of(P0), Some(v));
    // It remains in the command zone throughout the game.
    t.set_step(P0, Step::PrecombatMain);
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Graveyard),
        },
        &[Entity::Object(v)],
    );
    assert_eq!(t.zone(v), Zone::Command);
    assert_eq!(t.g.current(v), v);
}

#[test]
fn starting_life_is_twenty_plus_or_minus_the_life_modifier() {
    cr!("902.4");
    // Titania: life −5; Eladamri: +15.
    let mut t = vanguard_pregame(&[Some("Titania"), Some("Eladamri")], false);
    t.g.start();
    assert_eq!((t.life(P0), t.life(P1)), (15, 35));
}

#[test]
fn starting_hand_size_is_seven_as_modified_by_the_hand_modifier() {
    cr!("902.5");
    // Titania: hand +2; Royal Assassin Avatar: hand −2.
    let mut t = vanguard_pregame(&[Some("Titania"), Some("Royal Assassin Avatar")], false);
    t.g.start();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (9, 5));
    assert_eq!(
        (t.g.starting_hand_size(P0), t.g.starting_hand_size(P1)),
        (9, 5)
    );
}

#[test]
fn a_vanguard_mulligan_draws_a_hand_of_the_starting_hand_size() {
    cr!("902.5a");
    // Two players: P0 (Titania, nine cards) mulligans once: a new hand of nine cards, one
    // of them put on the bottom.
    let mut t = vanguard_pregame(&[Some("Titania"), None], true);
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P0), 8);
    assert_eq!(t.library_size(P0), 32);
    // Multiplayer: the first mulligan is for the same number of cards.
    let mut t = vanguard_pregame(&[Some("Titania"), None, None], true);
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P0), 9);
    // Royal Assassin Avatar (five cards) mulligans twice in a two-player game: five cards,
    // two on the bottom.
    let mut t = vanguard_pregame(&[None, Some("Royal Assassin Avatar")], true);
    t.answer(P1, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer(P1, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert_eq!(t.hand_size(P1), 3);
}

#[test]
fn maximum_hand_size_is_seven_as_modified_by_the_hand_modifier() {
    cr!("902.5b");
    let mut t = vanguard_pregame(&[Some("Titania"), Some("Royal Assassin Avatar")], false);
    t.g.start();
    assert_eq!(t.player(P0).max_hand_size, Some(9));
    assert_eq!(t.player(P1).max_hand_size, Some(5));
    // In the cleanup step, Royal Assassin Avatar's owner discards down to five cards.
    for _ in 0..4 {
        t.hand(P1, "Grizzly Bears");
    }
    assert!(t.hand_size(P1) > 5);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.hand_size(P1), 5);
}

#[test]
fn a_vanguard_is_owned_and_controlled_by_the_player_who_started_with_it() {
    cr!("902.6");
    let mut t = vanguard_pregame(&[Some("Serra"), Some("Maraxus")], false);
    t.g.start();
    let v = t.g.vanguard_of(P1).unwrap();
    assert_eq!((t.obj(v).owner, t.obj(v).controller), (P1, P1));
    // P0 tries to gain control of it: its controller is still its owner.
    t.set_step(P0, Step::PrecombatMain);
    run_effect(
        &mut t,
        P0,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(v)],
    );
    assert_eq!(t.obj(v).controller, P1);
    // Maraxus still pumps only P1's creatures.
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(mine), (2, 4));
    assert_eq!(t.pt(theirs), (3, 2));
}

#[test]
fn a_face_up_vanguards_abilities_function_from_the_command_zone() {
    cr!("902.7");
    let mut t = vanguard_pregame(&[Some("Urza"), Some("Royal Assassin Avatar")], false);
    t.g.start();
    // Activated: Urza — "{3}: Urza deals 1 damage to any target."
    let urza = t.g.vanguard_of(P0).unwrap();
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Wastes", 3);
    let life = t.life(P1);
    t.activate(P0, urza, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), life - 1);
    // Triggered: Royal Assassin Avatar — "At the beginning of your upkeep, you draw a card
    // and you lose 1 life."
    let (hand, life) = (t.hand_size(P1), t.life(P1));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), life - 1);
    assert_eq!(t.hand_size(P1), hand + 1);
    // Static: Serra — "Creatures you control get +0/+2."
    let mut t = vanguard_pregame(&[Some("Serra"), None], false);
    t.g.start();
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(b), (2, 4));
}
