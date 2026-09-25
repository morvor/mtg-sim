//! CR 315: conspiracies (Conspiracy Draft) — limited-only cards put into the command zone
//! from the sideboard as the game starts, face down with hidden agenda; their abilities,
//! ownership and control, and looking at face-down conspiracies.

use crate::r100_common::{fillers, pregame};
use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::deck::{check_constructed_with, check_limited, DeckProblem, NameEquivalence};
use mtg_engine::facedown::can_look_at;
use mtg_engine::game::GameConfig;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

fn copies(name: &str, n: usize) -> Vec<Arc<CardDef>> {
    (0..n).map(|_| card(name)).collect()
}

#[test]
fn conspiracies_arent_used_in_constructed_play() {
    cr!("315.1");
    let mut deck = copies("Forest", 60);
    let side = vec![card("Power Play")];
    let problems = check_constructed_with(&deck, &side, &NameEquivalence::default());
    assert_eq!(
        problems,
        vec![DeckProblem::ConspiracyNotAllowed {
            name: "Power Play".into()
        }]
    );
    assert!(check_constructed_with(&deck, &[], &NameEquivalence::default()).is_empty());
    deck.push(card("Power Play"));
    assert!(!check_constructed_with(&deck, &[], &NameEquivalence::default()).is_empty());
    // In limited play a conspiracy in the sideboard (the pool) is fine.
    let deck = copies("Forest", 40);
    let mut pool = deck.clone();
    pool.push(card("Power Play"));
    assert!(check_limited(&deck, &pool).is_empty());
}

#[test]
fn conspiracies_start_the_game_in_the_command_zone_from_the_sideboard() {
    cr!("315.2");
    let mut t = pregame(
        GameConfig {
            limited: true,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![fillers(40), fillers(40)],
    );
    let side = t.g.add_to_sideboard(
        P0,
        vec![card("Power Play"), card("Iterative Analysis"), card("Weight Advantage")],
    );
    // P0 puts two of them into the command zone.
    t.answer_choose(P0, &[Entity::Object(side[0]), Entity::Object(side[1])]);
    t.g.start();
    assert_eq!(t.zone(side[0]), Zone::Command);
    assert!(!t.obj(side[0]).face_down);
    // Hidden agenda: face down.
    assert_eq!(t.zone(side[1]), Zone::Command);
    assert!(t.obj(side[1]).face_down);
    assert_eq!(t.zone(side[2]), Zone::Outside(P0));
    // They're not part of the library that was shuffled.
    assert_eq!(t.g.player(P0).library.len() + t.hand_size(P0), 40);
}

#[test]
fn conspiracy_cards_remain_in_the_command_zone_and_cant_be_brought_into_the_game() {
    cr!("315.3");
    // Listed with a deck, a conspiracy isn't part of the library.
    let mut deck = fillers(40);
    deck.push(card("Power Play"));
    let t = pregame(GameConfig::default(), vec![deck, fillers(40)]);
    assert_eq!(t.g.player(P0).library.len(), 40);
    // Nor can it be included in a limited deck.
    let mut deck = copies("Forest", 40);
    deck.push(card("Power Play"));
    assert!(check_limited(&deck, &deck).contains(&DeckProblem::ConspiracyNotAllowed {
        name: "Power Play".into()
    }));
    // In the command zone: not a permanent, can't be cast, and it stays there.
    let mut t = TestGame::new(2);
    let c = t.command(P0, "Weight Advantage");
    assert!(t.g.permanents().all(|o| o.id != c));
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, c, CastMethod::Normal).is_err());
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Graveyard),
        },
        &[Entity::Object(c)],
    );
    assert_eq!(t.g.current(c), c);
    assert_eq!(t.zone(c), Zone::Command);
    // One that isn't in the game can't be brought in: Death Wish ("You may put a card
    // you own from outside the game into your hand") leaves it in the sideboard.
    let outside = t.custom(P0, (*card("Power Play")).clone(), Zone::Outside(P0));
    let wish = t.hand(P0, "Death Wish");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(outside)]);
    t.cast(P0, wish).go();
    t.resolve();
    assert_eq!(t.zone(outside), Zone::Outside(P0));
    assert!(!t.in_hand(P0, "Power Play"));
}

#[test]
fn conspiracies_have_no_subtypes() {
    cr!("315.4");
    for c in ["Power Play", "Weight Advantage", "Iterative Analysis"] {
        assert!(subtypes_of(c).is_empty(), "{c}");
    }
}

#[test]
fn a_face_up_conspiracys_static_and_triggered_abilities_function() {
    cr!("315.5");
    let mut t = TestGame::new(2);
    // Static: Weight Advantage — "Each creature you control assigns combat damage equal to
    // its toughness rather than its power."
    t.command(P0, "Weight Advantage");
    let wall = t.custom(P0, bear("Stout Bear", 1, 4), Zone::Battlefield);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(wall, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    // Triggered, face up: it triggers; face down: it doesn't.
    let mut t = TestGame::new(2);
    let up = t.custom(
        P0,
        oracle_card(
            "Open Plot",
            "Conspiracy",
            "",
            None,
            "Whenever you cast a spell, you gain 1 life.",
        ),
        Zone::Command,
    );
    let down = t.custom(
        P0,
        oracle_card(
            "Secret Plot",
            "Conspiracy",
            "",
            None,
            "Whenever you cast a spell, you gain 2 life.",
        ),
        Zone::Command,
    );
    t.g.objects[down.0 as usize].face_down = true;
    t.g.recompute();
    hold_stack(&mut t, P0);
    t.resolve_all();
    // 1 from the Holding Spell itself, 1 from the face-up conspiracy.
    assert_eq!(t.life(P0), 22);
    let _ = up;
}

#[test]
fn conspiracies_may_affect_the_start_of_the_game() {
    cr!("315.5a");
    let mut t = pregame(
        GameConfig {
            first_turn_chooser: Some(P0),
            skip_mulligans: true,
            ..Default::default()
        },
        vec![fillers(20), fillers(20)],
    );
    // Power Play: "You are the starting player."
    t.g.add_to_sideboard(P1, vec![card("Power Play")]);
    t.g.start();
    assert_eq!(t.g.turn.starting_player, P1);
}

#[test]
fn a_face_down_conspiracy_has_no_characteristics() {
    cr!("315.5b");
    let mut t = TestGame::new(2);
    let c = t.command(P0, "Iterative Analysis");
    t.g.objects[c.0 as usize].face_down = true;
    t.g.recompute();
    let o = t.obj(c);
    assert!(o.chars.name.is_empty());
    assert!(o.chars.card_types.is_empty());
    assert!(o.chars.abilities.is_empty());
    assert_eq!((o.chars.power, o.chars.toughness), (None, None));
}

#[test]
fn a_conspiracy_is_owned_and_controlled_by_the_player_who_put_it_into_the_command_zone() {
    cr!("315.6");
    let mut t = pregame(
        GameConfig {
            limited: true,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![fillers(40), fillers(40)],
    );
    let side = t.g.add_to_sideboard(P1, vec![card("Weight Advantage")]);
    t.g.start();
    let c = side[0];
    assert_eq!(t.zone(c), Zone::Command);
    assert_eq!((t.obj(c).owner, t.obj(c).controller), (P1, P1));
    // An effect trying to give P0 control of it doesn't.
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
        &[Entity::Object(c)],
    );
    assert_eq!(t.obj(c).controller, P1);
    let _ = CardType::Conspiracy;
}

#[test]
fn only_its_controller_may_look_at_a_face_down_conspiracy() {
    cr!("315.7");
    let mut t = TestGame::new(2);
    let c = t.command(P0, "Iterative Analysis");
    t.g.objects[c.0 as usize].face_down = true;
    t.g.recompute();
    assert!(can_look_at(&t.g, P0, c));
    assert!(!can_look_at(&t.g, P1, c));
    // Face up, anyone can.
    let d = t.command(P0, "Weight Advantage");
    assert!(can_look_at(&t.g, P1, d));
}
