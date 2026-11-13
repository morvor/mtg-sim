//! CR 904: the Archenemy casual variant — one archenemy with a scheme deck against a team,
//! the Supervillain Rumble option, and the Archenemy Commander option.

use crate::r100_common::{fillers, pregame};
use crate::r703_common::{add_scheme_deck, run_effect, supported};
use crate::r900_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::casual::check_scheme_deck;
use mtg_engine::deck::DeckProblem;
use mtg_engine::events::Event;
use mtg_engine::game::GameConfig;
use mtg_engine::life_totals::is_archenemy;
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::variants::{self, SET_IN_MOTION};
use mtg_engine::*;
use std::sync::Arc;

const SCHEMES: [&str; 22] = [
    "What's Yours Is Now Mine",
    "Dark Wings Bring Your Downfall",
    "My Undead Horde Awakens",
    "Nature Demands an Offering",
    "Every Dream a Nightmare",
    "Dance, Pathetic Marionette",
    "Your Will Is Not Your Own",
    "Nothing Can Stop Me Now",
    "Because I Have Willed It",
    "Only Blood Ends Your Nightmares",
    "A Display of My Dark Power",
    "This World Belongs to Me",
    "Roots of All Evil",
    "Plots That Span Centuries",
    "Realms Befitting My Majesty",
    "I Call for Slaughter",
    "No Secret Is Hidden from Me",
    "Make Yourself Useful",
    "You Cannot Hide from Me",
    "Pay Tribute to Me",
    "Look Skyward and Despair",
    "The Very Soil Shall Shake",
];

fn real(name: &str) -> CardDef {
    (*card(name)).clone()
}

fn saprolings(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Saproling"))
        .count()
}

fn set_in_motion_count(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name, .. } if name.as_str() == SET_IN_MOTION))
        .count()
}

/// An Archenemy game (not started): the archenemy's deck lists `schemes`; the others play
/// 40 fillers.
fn archenemy_pregame(teams: Vec<u8>, archenemy: usize, schemes: &[&str]) -> TestGame {
    let decks = (0..teams.len())
        .map(|i| {
            let mut d = fillers(40);
            if i == archenemy {
                d.extend(cards(schemes));
            }
            d
        })
        .collect();
    pregame(
        GameConfig {
            skip_mulligans: true,
            ..GameConfig::archenemy_game(teams)
        },
        decks,
    )
}

#[test]
fn a_team_faces_a_single_archenemy_with_schemes() {
    cr!("904.1");
    supported("Roots of All Evil");
    let mut t = archenemy_pregame(vec![0, 1, 1, 1], 0, &["Roots of All Evil"]);
    t.g.start();
    assert_eq!(t.g.archenemy(), Some(P0));
    for q in [P1, P2, P3] {
        assert!(t.g.are_opponents(P0, q));
        assert!(!is_archenemy(&t.g, q));
    }
    assert!(!t.g.are_opponents(P1, P2));
    // The normal rules: opening hands of seven.
    assert_eq!(t.hand_size(P1), 7);
    // Strengthened by schemes: one is set in motion in the archenemy's main phase.
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(saprolings(&t, P0), 5);
}

#[test]
fn the_default_archenemy_setup() {
    cr!("904.2");
    let c = GameConfig::archenemy_game(vec![0, 1, 1, 1]);
    assert_eq!(c.validate(4), Ok(()));
    assert!(c.attack_multiple_players && c.shared_team_turns);
    // No other multiplayer options.
    let bad = GameConfig {
        range_of_influence: Some(2),
        ..GameConfig::archenemy_game(vec![0, 1, 1, 1])
    };
    assert!(bad.validate(4).unwrap_err().contains(&SetupError::VariantOptions));
    // Exactly two teams.
    let three = GameConfig::archenemy_game(vec![0, 1, 1, 2]);
    assert!(three.validate(4).unwrap_err().contains(&SetupError::ArchenemyTeams));
    // Shared team turns: the archenemy's opponents take their turns together.
    let mut t = TestGame::with_config(4, GameConfig::archenemy_game(vec![0, 1, 1, 1]));
    t.set_step(P1, Step::PrecombatMain);
    assert_eq!(t.g.active_players(), vec![P1, P2, P3]);
    t.set_step(P0, Step::PrecombatMain);
    assert_eq!(t.g.active_players(), vec![P0]);
    // Attack multiple players: the archenemy attacks two heroes at once.
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P3))], &[]);
    assert_eq!((t.life(P1), t.life(P3)), (18, 18));
}

#[test]
fn one_team_is_a_single_player_the_archenemy() {
    cr!("904.2a");
    let t = TestGame::with_config(3, GameConfig::archenemy_game(vec![1, 0, 1]));
    assert_eq!(t.g.archenemy(), Some(P1));
    assert!(is_archenemy(&t.g, P1));
    assert!(!is_archenemy(&t.g, P0));
    // Two teams of two: no single player, not an Archenemy game.
    let c = GameConfig::archenemy_game(vec![0, 0, 1, 1]);
    assert!(c.validate(4).unwrap_err().contains(&SetupError::ArchenemyTeams));
}

#[test]
fn the_other_team_has_any_number_of_players() {
    cr!("904.2b");
    for teams in [vec![0, 1], vec![0, 1, 1], vec![0, 1, 1, 1, 1, 1]] {
        let n = teams.len();
        assert_eq!(GameConfig::archenemy_game(teams).validate(n), Ok(()));
    }
    let t = TestGame::with_config(6, GameConfig::archenemy_game(vec![0, 1, 1, 1, 1, 1]));
    assert_eq!(t.g.archenemy(), Some(P0));
}

#[test]
fn a_scheme_deck_has_twenty_schemes_at_most_two_of_each() {
    cr!("904.3");
    let twenty = cards(&SCHEMES[..20]);
    assert!(check_scheme_deck(&twenty, false).is_empty());
    assert!(check_scheme_deck(&twenty[..19], false)
        .iter()
        .any(|p| matches!(p, DeckProblem::TooFewCards { have: 19, min: 20 })));
    // Two of a name is fine; three isn't.
    let mut two = twenty.clone();
    two.push(card("Roots of All Evil"));
    assert!(check_scheme_deck(&two, false).is_empty());
    two.push(card("Roots of All Evil"));
    assert!(check_scheme_deck(&two, false)
        .iter()
        .any(|p| matches!(p, DeckProblem::TooManyCopies { name, have: 3, max: 2 } if name == "Roots of All Evil")));
    // Scheme cards only.
    let mut other = twenty.clone();
    other.push(card("Krosa"));
    assert!(check_scheme_deck(&other, false)
        .iter()
        .any(|p| matches!(p, DeckProblem::WrongCardType { name, .. } if name == "Krosa")));
}

#[test]
fn scheme_cards_stay_in_the_command_zone() {
    cr!("904.4");
    let mut t = archenemy_pregame(vec![0, 1, 1], 0, &["Roots of All Evil", "The Very Soil Shall Shake"]);
    t.g.start();
    let deck = variants::scheme_deck(&t.g, P0);
    assert_eq!(deck.len(), 2);
    // In the scheme deck: not in the library, in the command zone.
    assert!(deck.iter().all(|id| t.zone(*id) == Zone::Command));
    assert_eq!(t.g.player(P0).library.len() + t.hand_size(P0), 40);
    // Face up after being set in motion: still there, and it can't be moved or cast.
    t.set_step(P0, Step::PrecombatMain);
    let top = variants::set_in_motion(&mut t.g, P0).unwrap();
    assert!(!t.obj(top).face_down);
    assert_eq!(t.zone(top), Zone::Command);
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, top, CastMethod::Normal).is_err());
    run_effect(
        &mut t,
        P1,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination::zone(ZoneKind::Exile),
        },
        &[Entity::Object(top)],
    );
    assert_eq!(t.zone(top), Zone::Command);
}

#[test]
fn archenemy_starting_life_totals() {
    cr!("904.5");
    let mut t = archenemy_pregame(vec![0, 1, 1, 1], 0, &[]);
    t.g.start();
    assert_eq!(
        (t.life(P0), t.life(P1), t.life(P2), t.life(P3)),
        (40, 20, 20, 20)
    );
}

#[test]
fn the_archenemy_takes_the_first_turn() {
    cr!("904.6");
    // The archenemy is seated third: they go first, every time.
    for seed in 0..5 {
        let mut t = pregame(
            GameConfig {
                skip_mulligans: true,
                seed,
                ..GameConfig::archenemy_game(vec![1, 1, 0, 1])
            },
            vec![fillers(40), fillers(40), fillers(40), fillers(40)],
        );
        t.g.start();
        assert_eq!(t.g.turn.starting_player, P2);
        assert_eq!(t.g.turn.active, P2);
    }
}

#[test]
fn a_scheme_is_owned_and_controlled_by_the_player_who_started_with_it() {
    cr!("904.7");
    let mut t = archenemy_pregame(vec![0, 1, 1], 0, &["The Very Soil Shall Shake"]);
    t.g.start();
    let scheme = variants::scheme_deck(&t.g, P0)[0];
    assert_eq!((t.obj(scheme).owner, t.obj(scheme).controller), (P0, P0));
    t.set_step(P0, Step::PrecombatMain);
    variants::set_in_motion(&mut t.g, P0);
    run_effect(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(scheme)],
    );
    assert_eq!(t.obj(scheme).controller, P0);
}

#[test]
fn a_face_up_schemes_abilities_function_from_the_command_zone() {
    cr!("904.8");
    supported("The Very Soil Shall Shake");
    let mut t = TestGame::with_config(3, GameConfig::archenemy_game(vec![0, 1, 1]));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    add_scheme_deck(
        &mut t,
        P0,
        vec![real("The Very Soil Shall Shake"), real("Roots of All Evil")],
    );
    // Face down in the scheme deck: nothing.
    assert_eq!(t.pt(bears), (2, 2));
    // Static: "Creatures you control get +2/+2 and have trample."
    variants::set_in_motion(&mut t.g, P0);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(theirs), (2, 2));
    // Triggered: "When you set this scheme in motion, create five 1/1 green Saproling
    // creature tokens."
    variants::set_in_motion(&mut t.g, P0);
    t.settle();
    t.resolve_all();
    assert_eq!(saprolings(&t, P0), 5);
}

#[test]
fn the_archenemy_sets_a_scheme_in_motion_as_their_main_phase_begins() {
    cr!("904.9");
    let mut t = archenemy_pregame(vec![0, 1, 1], 0, &["Roots of All Evil"]);
    t.g.start();
    assert_eq!(set_in_motion_count(&t), 0);
    t.advance_to(P0, Step::PrecombatMain);
    // Moved off the scheme deck and turned face up (a turn-based action, not using the
    // stack), and its "when you set this scheme in motion" ability triggered.
    assert_eq!(set_in_motion_count(&t), 1);
    assert!(variants::scheme_deck(&t.g, P0).is_empty());
    t.resolve_all();
    assert_eq!(saprolings(&t, P0), 5);
    // On the other team's turn, nothing is set in motion.
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(set_in_motion_count(&t), 0);
}

#[test]
fn a_non_ongoing_scheme_goes_back_once_no_scheme_ability_is_waiting() {
    cr!("904.10");
    let mut t = TestGame::with_config(3, GameConfig::archenemy_game(vec![0, 1, 1]));
    let deck = add_scheme_deck(
        &mut t,
        P0,
        vec![real("Roots of All Evil"), real("Look Skyward and Despair")],
    );
    variants::set_in_motion(&mut t.g, P0);
    t.settle();
    // Its ability is on the stack: it stays face up.
    assert!(!t.obj(deck[0]).face_down);
    t.resolve();
    // Then it's turned face down and put on the bottom of the scheme deck.
    let now = t.g.current(deck[0]);
    assert!(t.obj(now).face_down);
    assert_eq!(variants::scheme_deck(&t.g, P0), vec![deck[1], now]);
}

#[test]
fn an_ongoing_scheme_stays_face_up_until_abandoned() {
    cr!("904.11");
    let mut t = TestGame::with_config(3, GameConfig::archenemy_game(vec![0, 1, 1]));
    let bears = t.battlefield(P0, "Grizzly Bears");
    let deck = add_scheme_deck(&mut t, P0, vec![real("The Very Soil Shall Shake")]);
    variants::set_in_motion(&mut t.g, P0);
    for _ in 0..3 {
        t.settle();
    }
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj(deck[0]).face_down);
    // "When a creature you control dies, abandon this scheme."
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert!(t.obj(t.g.current(deck[0])).face_down);
}

#[test]
fn supervillain_rumble_is_free_for_all_between_archenemies() {
    cr!("904.12", "904.12a");
    let c = GameConfig::supervillain_rumble();
    assert_eq!(c.validate(4), Ok(()));
    assert!(c.attack_multiple_players);
    let bad = GameConfig {
        range_of_influence: Some(1),
        ..GameConfig::supervillain_rumble()
    };
    assert!(bad.validate(4).unwrap_err().contains(&SetupError::VariantOptions));
    // Each player has their own scheme deck.
    let mut t = TestGame::with_config(3, GameConfig::supervillain_rumble());
    let a = add_scheme_deck(&mut t, P0, vec![real("Roots of All Evil")]);
    let b = add_scheme_deck(&mut t, P1, vec![real("Look Skyward and Despair")]);
    assert_eq!(variants::scheme_deck(&t.g, P0), a);
    assert_eq!(variants::scheme_deck(&t.g, P1), b);
    for q in [P1, P2] {
        assert!(t.g.are_opponents(P0, q));
    }
}

#[test]
fn every_supervillain_rumble_player_is_an_archenemy() {
    cr!("904.12b");
    let mut t = TestGame::with_config(3, GameConfig::supervillain_rumble());
    for p in [P0, P1, P2] {
        assert!(is_archenemy(&t.g, p));
    }
    add_scheme_deck(&mut t, P1, vec![real("Roots of All Evil")]);
    t.set_step(P0, Step::End);
    t.advance_to(P1, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(saprolings(&t, P1), 5);
}

#[test]
fn in_supervillain_rumble_the_starting_player_is_random_and_everyone_starts_at_40() {
    cr!("904.12c");
    let mut starters = Vec::new();
    for seed in 0..12 {
        let mut t = pregame(
            GameConfig {
                skip_mulligans: true,
                seed,
                ..GameConfig::supervillain_rumble()
            },
            vec![fillers(40), fillers(40), fillers(40)],
        );
        t.g.start();
        assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (40, 40, 40));
        if !starters.contains(&t.g.turn.starting_player) {
            starters.push(t.g.turn.starting_player);
        }
    }
    assert!(starters.len() > 1);
}

#[test]
fn archenemy_commander_is_commander_with_the_archenemy_rules() {
    cr!("904.13", "904.13a");
    let config = GameConfig::archenemy_commander(vec![0, 1, 1]);
    assert_eq!(config.validate(3), Ok(()));
    let mut deck0: Vec<Arc<CardDef>> = vec![card("Isamaru, Hound of Konda")];
    deck0.extend((0..99).map(|_| card("Plains")));
    deck0.extend(cards(&["Roots of All Evil"]));
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            ..config
        },
        vec![deck0, fillers(100), fillers(100)],
    );
    assert!(t.g.designate_commander(P0, "Isamaru, Hound of Konda"));
    t.g.start();
    // The Commander rules: the commander is in the command zone.
    assert_eq!(t.g.find_in_zone(Zone::Command, "Isamaru, Hound of Konda").len(), 1);
    // The Archenemy rules: the archenemy goes first and sets schemes in motion.
    assert_eq!(t.g.turn.starting_player, P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(saprolings(&t, P0), 5);
}

#[test]
fn archenemy_commander_life_totals() {
    cr!("904.13b");
    let mut t = TestGame::with_config(4, GameConfig::archenemy_commander(vec![0, 1, 1, 1]));
    assert_eq!(t.life(P0), 60);
    // The opposing team's life total is shared, starting at 60.
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (60, 60, 60));
    t.g.lose_life(P2, 7);
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (53, 53, 53));
    assert_eq!(t.life(P0), 60);
    // When it reaches 0, the team loses.
    t.g.lose_life(P3, 53);
    t.settle();
    assert!(t.has_lost(P1) && t.has_lost(P2) && t.has_lost(P3));
    assert!(!t.has_lost(P0));
}

#[test]
fn archenemy_commander_poison_counters_arent_shared() {
    cr!("904.13c");
    let mut t = TestGame::with_config(4, GameConfig::archenemy_commander(vec![0, 1, 1, 1]));
    t.g.add_counters(Entity::Player(P1), counters::POISON, 5, None);
    t.g.add_counters(Entity::Player(P2), counters::POISON, 5, None);
    t.settle();
    assert!(!t.has_lost(P1) && !t.has_lost(P2));
    t.g.add_counters(Entity::Player(P2), counters::POISON, 5, None);
    t.settle();
    assert!(t.has_lost(P2));
    assert!(!t.has_lost(P1) && !t.has_lost(P3));
    // The archenemy with ten loses too.
    t.g.add_counters(Entity::Player(P0), counters::POISON, 10, None);
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn an_archenemy_commander_scheme_deck_has_ten_different_schemes() {
    cr!("904.13d");
    let ten = cards(&SCHEMES[..10]);
    assert!(check_scheme_deck(&ten, true).is_empty());
    assert!(!check_scheme_deck(&ten, false).is_empty());
    assert!(check_scheme_deck(&ten[..9], true)
        .iter()
        .any(|p| matches!(p, DeckProblem::TooFewCards { min: 10, .. })));
    let mut dup = ten.clone();
    dup.push(card("What's Yours Is Now Mine"));
    assert!(check_scheme_deck(&dup, true)
        .iter()
        .any(|p| matches!(p, DeckProblem::TooManyCopies { max: 1, .. })));
}
