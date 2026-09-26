//! CR 717: Attraction cards.

use super::r100_common::{fillers, pregame};
use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::attraction_cards::{self, check_constructed_deck, check_limited_deck};
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::{check_constructed, DeckProblem};
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::game::GameConfig;
use mtg_engine::kwa::attractions::attraction_deck;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::variants::{self, ROLLED_TO_VISIT};
use mtg_engine::*;
use std::sync::Arc;

/// Information Booth (lit up: 2, 6): "Visit — Draw a card."
const BOOTH: &str = "Information Booth";

const ATTRACTIONS: [&str; 12] = [
    "Push Your Luck",
    "Scavenger Hunt",
    "The Superlatorium",
    "Guess Your Fate",
    "Gallery of Legends",
    "Information Booth",
    "Trivia Contest",
    "Log Flume",
    "Costume Shop",
    "Concession Stand",
    "Hall of Mirrors",
    "Pick-a-Beeble",
];

fn cards(names: &[&str]) -> Vec<Arc<CardDef>> {
    names.iter().map(|n| card(n)).collect()
}

/// Puts Attraction cards into `p`'s Attraction deck (face down in the command zone).
fn attraction_deck_of(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let ids = names
        .iter()
        .map(|n| {
            let id = t.command(p, n);
            t.g.objects[id.0 as usize].face_down = true;
            id
        })
        .collect();
    t.g.recompute();
    ids
}

fn rolls(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .chain(t.g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name, .. } if name == ROLLED_TO_VISIT))
        .count()
}

#[test]
fn an_attraction_is_a_nontraditional_artifact_with_lit_up_numbers() {
    cr!("717.1");
    supported(BOOTH);
    let def = card(BOOTH);
    let c = &def.front().chars;
    assert!(c.is(CardType::Artifact) && c.has_subtype("Attraction"));
    assert!(attraction_cards::is_attraction_card(&def));
    assert!(variants::is_nontraditional(&def));
    assert_eq!(def.attraction_lights, vec![2, 6]);
    let mut t = TestGame::new(2);
    let booth = t.battlefield(P0, BOOTH);
    assert!(variants::lit_up(t.obj(booth), 2) && variants::lit_up(t.obj(booth), 6));
    assert!(!variants::lit_up(t.obj(booth), 3));
}

#[test]
fn attractions_start_the_game_in_a_shuffled_attraction_deck() {
    cr!("717.2");
    let deck: Vec<Arc<CardDef>> = fillers(60)
        .into_iter()
        .chain(cards(&ATTRACTIONS[..10]))
        .collect();
    // They don't count toward the deck's size.
    let too_few = |deck: &[Arc<CardDef>]| {
        check_constructed(deck)
            .into_iter()
            .find(|p| matches!(p, DeckProblem::TooFewCards { .. }))
    };
    assert_eq!(too_few(&deck), None);
    let short: Vec<Arc<CardDef>> = fillers(59)
        .into_iter()
        .chain(cards(&ATTRACTIONS[..10]))
        .collect();
    assert_eq!(
        too_few(&short),
        Some(DeckProblem::TooFewCards { have: 59, min: 60 })
    );
    let mut orders = std::collections::BTreeSet::new();
    for seed in 0..6 {
        let mut t = pregame(
            GameConfig {
                skip_mulligans: true,
                seed,
                ..GameConfig::default()
            },
            vec![deck.clone(), fillers(60)],
        );
        // Not in the library: face down in the command zone.
        assert_eq!(t.library_size(P0), 60);
        assert_eq!(attraction_deck(&t.g, P0).len(), 10);
        t.g.start();
        let order = attraction_deck(&t.g, P0);
        assert_eq!(order.len(), 10);
        orders.insert(order);
    }
    // Shuffled before the game begins.
    assert!(orders.len() > 1);
}

#[test]
fn a_constructed_attraction_deck_has_ten_differently_named_attractions() {
    cr!("717.2a");
    assert!(check_constructed_deck(&cards(&ATTRACTIONS[..10])).is_empty());
    assert_eq!(
        check_constructed_deck(&cards(&ATTRACTIONS[..9])),
        vec![DeckProblem::TooFewCards { have: 9, min: 10 }]
    );
    let mut dup = cards(&ATTRACTIONS[..10]);
    dup.push(card(BOOTH));
    assert_eq!(
        check_constructed_deck(&dup),
        vec![DeckProblem::TooManyCopies {
            name: BOOTH.into(),
            have: 2,
            max: 1
        }]
    );
    let mut other = cards(&ATTRACTIONS[..10]);
    other.push(card("Grizzly Bears"));
    assert!(check_constructed_deck(&other).contains(&DeckProblem::NotAnAttraction {
        name: "Grizzly Bears".into()
    }));
}

#[test]
fn a_limited_attraction_deck_has_three_attractions_from_the_pool() {
    cr!("717.2b");
    let pool = cards(&[BOOTH, BOOTH, "Log Flume", "Costume Shop"]);
    // Duplicates are fine if the pool has them.
    assert!(check_limited_deck(&cards(&[BOOTH, BOOTH, "Log Flume"]), &pool).is_empty());
    assert_eq!(
        check_limited_deck(&cards(&[BOOTH, "Log Flume"]), &pool),
        vec![DeckProblem::TooFewCards { have: 2, min: 3 }]
    );
    assert_eq!(
        check_limited_deck(&cards(&[BOOTH, "Log Flume", "Concession Stand"]), &pool),
        vec![DeckProblem::NotInPool {
            name: "Concession Stand".into(),
            have: 1,
            available: 0
        }]
    );
}

#[test]
fn an_attraction_enters_the_battlefield_from_the_command_zone_when_opened() {
    cr!("717.3");
    supported("Seasoned Buttoneer");
    let mut t = TestGame::new(2);
    let deck = attraction_deck_of(&mut t, P0, &[BOOTH]);
    assert_eq!(t.zone(deck[0]), Zone::Command);
    // Seasoned Buttoneer: "When this creature enters, open an Attraction."
    t.enter(P0, "Seasoned Buttoneer");
    t.resolve_all();
    let booth = t.g.current(deck[0]);
    assert_eq!(t.zone(booth), Zone::Battlefield);
    assert!(attraction_deck(&t.g, P0).is_empty());
}

#[test]
fn a_player_with_attractions_rolls_to_visit_them_as_their_precombat_main_phase_begins() {
    cr!("717.4", "717.5");
    let mut t = TestGame::new(2);
    t.battlefield(P0, BOOTH);
    let hand = t.hand_size(P0);
    // A 2 is lit up on Information Booth: "Visit — Draw a card."
    t.g.dice.loaded.push_back(2);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    // The roll is a turn-based action: the visit ability has triggered, nothing else
    // used the stack.
    assert_eq!(rolls(&t), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // A 3 isn't lit up: no visit.
    let mut t = TestGame::new(2);
    t.battlefield(P0, BOOTH);
    let hand = t.hand_size(P0);
    t.g.dice.loaded.push_back(3);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(rolls(&t), 1);
    assert_eq!(t.hand_size(P0), hand);
    // A player who controls no Attractions doesn't roll.
    let mut t = TestGame::new(2);
    t.battlefield(P1, BOOTH);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(rolls(&t), 0);
}

#[test]
fn an_attraction_card_would_go_elsewhere_than_battlefield_exile_or_command_goes_to_the_junkyard() {
    cr!("717.6", "717.6a");
    let mut t = TestGame::new(2);
    let deck = attraction_deck_of(&mut t, P0, &["Log Flume"]);
    // Destroyed: it goes to the command zone instead of the graveyard, face up, apart
    // from the Attraction deck.
    let booth = t.battlefield(P0, BOOTH);
    run_effect(
        &mut t,
        P1,
        None,
        &[Entity::Object(booth)],
        Effect::Destroy {
            what: Sel::Target(0),
            no_regen: false,
        },
    );
    assert_eq!(t.graveyard_size(P0), 0);
    let junk = attraction_cards::junkyard(&t.g, P0);
    assert_eq!(junk.len(), 1);
    assert_eq!(t.obj(junk[0]).chars.name, BOOTH);
    assert!(!t.obj(junk[0]).face_down);
    assert_eq!(attraction_deck(&t.g, P0), deck);
    // Returned to its owner's hand: the command zone too.
    let booth = t.battlefield(P0, BOOTH);
    t.g.move_object(booth, Zone::Hand(P0), MoveCause::Effect, None);
    assert!(!t.in_hand(P0, BOOTH));
    assert_eq!(attraction_cards::junkyard(&t.g, P0).len(), 2);
    // Exiled: it stays in exile.
    let booth = t.battlefield(P0, BOOTH);
    run_effect(
        &mut t,
        P1,
        None,
        &[Entity::Object(booth)],
        Effect::Exile {
            what: Sel::Target(0),
            face_down: false,
            link: false,
        },
    );
    assert!(t.in_exile(BOOTH));
    // Opening an Attraction uses the deck, not the junkyard.
    t.enter(P0, "Seasoned Buttoneer");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Log Flume").len(), 1);
    assert_eq!(attraction_cards::junkyard(&t.g, P0).len(), 2);
}
