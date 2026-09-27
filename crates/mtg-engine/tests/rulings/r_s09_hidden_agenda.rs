//! Rulings batch S09 — hidden agenda (CR 702.106) and conspiracies (CR 315, 905.4): a
//! conspiracy with hidden agenda starts the game face down in the command zone with a
//! secretly chosen card name, and may be turned face up any time its controller has
//! priority.

use crate::r_s01_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::deck::{check_limited, DeckProblem};
use mtg_engine::events::Event;
use mtg_engine::facedown::REVEALED;
use mtg_engine::game::{Game, GameConfig};
use mtg_engine::kw::hidden_agenda::{as_put_into_command_zone, chosen_names};
use mtg_engine::match_play::Match;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn limited() -> GameConfig {
    GameConfig {
        limited: true,
        skip_mulligans: true,
        ..Default::default()
    }
}

fn scripted(n: usize) -> (Arc<Mutex<Script>>, Vec<Box<dyn Agent>>) {
    let script = Arc::new(Mutex::new(Script {
        queues: vec![VecDeque::new(); n],
        asked: vec![],
    }));
    let agents = (0..n)
        .map(|i| {
            Box::new(ScriptedAgent {
                player: PlayerId(i as u8),
                script: script.clone(),
            }) as Box<dyn Agent>
        })
        .collect();
    (script, agents)
}

/// A two-player limited game that hasn't started yet.
fn pregame() -> TestGame {
    let (script, agents) = scripted(2);
    let decks = (0..2).map(|_| vec![filler_card(); 40]).collect();
    let mut g = Game::new(limited(), decks, agents);
    g.logging = true;
    TestGame { g, script }
}

fn name(t: &mut TestGame, p: PlayerId, n: &str) {
    t.answer(p, DecisionKind::Name, Answer::Text(n.into()));
}

/// A conspiracy `p` put into the command zone as the game began, naming `n`.
fn agenda(t: &mut TestGame, p: PlayerId, conspiracy: &str, n: &str) -> ObjectId {
    supported(conspiracy);
    let c = t.custom(p, (*card(conspiracy)).clone(), Zone::Command);
    name(t, p, n);
    as_put_into_command_zone(&mut t.g, p, c);
    t.g.recompute();
    c
}

fn turn_face_up(t: &mut TestGame, p: PlayerId, c: ObjectId) {
    t.g.turn.priority = Some(p);
    let action = Action::Special(SpecialAction::TurnFaceUp { obj: c });
    assert!(t.g.legal_actions(p).contains(&action));
    t.g.take_action(p, action);
    t.g.recompute();
}

fn revealed(t: &TestGame, c: ObjectId) -> bool {
    t.g.turn_events.iter().any(|e| {
        matches!(e, Event::Custom { name, obj: Some(o), .. } if name == REVEALED && *o == c)
    })
}

fn copies(n: &str, k: usize) -> Vec<Arc<CardDef>> {
    (0..k).map(|_| card(n)).collect()
}

#[test]
fn a_conspiracy_doesnt_count_toward_the_minimum_deck_size() {
    cr!("100.2b", "315.3");
    ruling!(
        "Brago's Favor",
        "A conspiracy doesn't count as a card in your deck for purposes of meeting minimum deck size requirements."
    );
    let pool: Vec<Arc<CardDef>> = copies("Grizzly Bears", 40)
        .into_iter()
        .chain(std::iter::once(card("Brago's Favor")))
        .collect();
    // Thirty-nine cards plus a conspiracy: too few cards.
    let mut deck = copies("Grizzly Bears", 39);
    deck.push(card("Brago's Favor"));
    let problems = check_limited(&deck, &pool);
    assert!(problems.contains(&DeckProblem::TooFewCards { have: 39, min: 40 }));
    // Forty cards, with the conspiracy drafted alongside: legal.
    assert!(check_limited(&copies("Grizzly Bears", 40), &pool).is_empty());
}

#[test]
fn conspiracies_are_optional_and_only_put_into_the_command_zone_as_the_game_begins() {
    cr!("315.2", "905.4");
    ruling!(
        "Brago's Favor",
        "You don't have to play with any conspiracy you draft. However, you have only one opportunity to put conspiracies into the command zone, as the game begins."
    );
    let mut t = pregame();
    let side = t
        .g
        .add_to_sideboard(P0, vec![card("Brago's Favor"), card("Secret Summoning")]);
    // P0 plays with only one of them.
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    name(&mut t, P0, "Grizzly Bears");
    t.g.start();
    assert_eq!(t.zone(side[0]), Zone::Command);
    assert_eq!(t.zone(side[1]), Zone::Outside(P0));
    // Later, there's no way to put the other one into the command zone.
    t.g.run_until(10_000, |g| g.turn.number >= 3);
    assert_eq!(t.zone(side[1]), Zone::Outside(P0));
    let asked_about_it = t.asked().iter().filter(|(_, d)| {
        matches!(d, Decision::ChooseEntities { candidates, .. }
            if candidates.contains(&Entity::Object(side[1])))
    }).count();
    assert_eq!(asked_about_it, 1);
    // Playing with none of them is fine too.
    let mut t = pregame();
    let side = t.g.add_to_sideboard(P0, vec![card("Brago's Favor")]);
    t.answer_choose(P0, &[]);
    t.g.start();
    assert_eq!(t.zone(side[0]), Zone::Outside(P0));
    assert!(t.g.command.is_empty());
}

#[test]
fn face_down_conspiracies_are_revealed_at_the_end_of_the_game() {
    cr!("702.106e");
    ruling!(
        "Brago's Favor",
        "At the end of the game, you must reveal any face-down conspiracies you own in the command zone to all players."
    );
    let mut t = TestGame::new(2);
    let mine = agenda(&mut t, P0, "Brago's Favor", "Grizzly Bears");
    let theirs = agenda(&mut t, P1, "Secret Summoning", "Llanowar Elves");
    let face_up = agenda(&mut t, P0, "Muzzio's Preparations", "Hill Giant");
    turn_face_up(&mut t, P0, face_up);
    assert!(!revealed(&t, mine) && !revealed(&t, theirs));
    t.g.player_loses(P1);
    t.g.flush_events();
    assert!(t.g.is_over());
    assert!(revealed(&t, mine));
    assert!(revealed(&t, theirs));
    assert!(!revealed(&t, face_up));
}

#[test]
fn only_conspiracies_go_to_the_command_zone_and_only_hidden_agendas_face_down() {
    cr!("315.2", "702.106a", "702.106e");
    ruling!(
        "Brago's Favor",
        "Notably, you can't bluff conspiracies with hidden agenda by putting other cards into the command zone face down as the game starts."
    );
    supported("Power Play");
    let mut t = pregame();
    let side = t.g.add_to_sideboard(
        P0,
        vec![
            card("Grizzly Bears"),
            card("Power Play"),
            card("Brago's Favor"),
        ],
    );
    let (bears, power_play, favor) = (side[0], side[1], side[2]);
    // Trying to put all three into the command zone: the Bears aren't a conspiracy, so
    // the choice is invalid and the default (every conspiracy) is used.
    t.answer_choose(
        P0,
        &[
            Entity::Object(bears),
            Entity::Object(power_play),
            Entity::Object(favor),
        ],
    );
    name(&mut t, P0, "Grizzly Bears");
    t.g.start();
    let offered: Vec<Vec<Entity>> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert!(offered.iter().all(|c| !c.contains(&Entity::Object(bears))));
    assert_eq!(t.zone(bears), Zone::Outside(P0));
    assert_eq!(t.zone(power_play), Zone::Command);
    assert!(!t.obj(power_play).face_down);
    assert_eq!(t.zone(favor), Zone::Command);
    assert!(t.obj(favor).face_down);
    // At the end of the game the face-down conspiracy is revealed.
    t.g.player_loses(P1);
    t.g.flush_events();
    assert!(revealed(&t, favor));
}

#[test]
fn a_different_card_can_be_named_in_each_game_of_a_match() {
    cr!("100.6a", "702.106a");
    ruling!(
        "Brago's Favor",
        "If you play multiple games after the draft, you can name a different card in each new game."
    );
    let decks = vec![vec![filler_card(); 40], vec![filler_card(); 40]];
    let m = Match::new(limited(), decks, vec![vec![card("Brago's Favor")], vec![]]);
    let mut named = vec![];
    for (seed, n) in [(1, "Grizzly Bears"), (2, "Llanowar Elves")] {
        let (script, agents) = scripted(2);
        script.lock().unwrap().queues[0]
            .push_back((DecisionKind::Name, Answer::Text(n.into())));
        let mut g = m.next_game(agents, seed);
        g.start();
        let c = g.command.iter().copied().find(|c| g.obj(*c).face_down);
        let c = c.expect("the conspiracy is face down in the command zone");
        named.push(chosen_names(&g, c));
    }
    assert_eq!(named, vec![vec!["Grizzly Bears"], vec!["Llanowar Elves"]]);
}

#[test]
fn the_chosen_name_must_be_a_magic_card_not_a_token() {
    cr!("702.106a", "201.4");
    ruling!(
        "Brago's Favor",
        "You must name a Magic card. Notably, you can't name a token."
    );
    let mut t = TestGame::new(2);
    let token = agenda(&mut t, P0, "Brago's Favor", "Treasure");
    assert!(chosen_names(&t.g, token).is_empty());
    let other = agenda(&mut t, P0, "Brago's Favor", "Goblin");
    assert!(chosen_names(&t.g, other).is_empty());
    let real = agenda(&mut t, P0, "Brago's Favor", "Grizzly Bears");
    assert_eq!(chosen_names(&t.g, real), vec!["Grizzly Bears"]);
}

#[test]
fn a_triggered_ability_triggers_only_if_the_conspiracy_is_already_face_up() {
    cr!("315.5", "702.106c", "603.2");
    ruling!(
        "Secret Summoning",
        "A conspiracy with hidden agenda that has a triggered ability must be face up before that ability's trigger condition is met in order for it to trigger. Turning it face up afterward won't have any effect."
    );
    let mut t = TestGame::new(2);
    // "Whenever a creature you control with the chosen name enters, you may search your
    // library for any number of cards with that name, reveal them, put them into your
    // hand, then shuffle."
    let c = agenda(&mut t, P0, "Secret Summoning", "Grizzly Bears");
    t.library_top(P0, "Grizzly Bears");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Turned face up afterward: nothing happens.
    turn_face_up(&mut t, P0, c);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Face up, the next one triggers it.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "search your library"), 1);
}

#[test]
fn each_face_down_conspiracy_has_its_own_named_card() {
    cr!("702.106b", "702.106d");
    ruling!(
        "Muzzio's Preparations",
        "If you have multiple face-down conspiracies, you may name a different card for each one."
    );
    let mut t = TestGame::new(2);
    // Muzzio's Preparations: "Each creature you control with the chosen name enters with
    // an additional +1/+1 counter on it."
    let muzzio = agenda(&mut t, P0, "Muzzio's Preparations", "Grizzly Bears");
    let summoning = agenda(&mut t, P0, "Secret Summoning", "Llanowar Elves");
    assert!(t.obj(muzzio).face_down && t.obj(summoning).face_down);
    assert_eq!(chosen_names(&t.g, muzzio), vec!["Grizzly Bears"]);
    assert_eq!(chosen_names(&t.g, summoning), vec!["Llanowar Elves"]);
    turn_face_up(&mut t, P0, muzzio);
    turn_face_up(&mut t, P0, summoning);
    // Each applies to its own name only.
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.stack_len(), 0);
    let elves = t.enter(P0, "Llanowar Elves");
    t.settle();
    assert_eq!(t.counters(elves, counters::PLUS1), 0);
    assert_eq!(triggers_on_stack(&t, "search your library"), 1);
}

#[test]
fn the_card_is_named_as_the_game_begins_not_when_turned_face_up() {
    cr!("702.106a", "702.106c");
    ruling!(
        "Brago's Favor",
        "You name the card as the game begins, as you put the conspiracy into the command zone, not as you turn the face-down conspiracy face up."
    );
    let mut t = pregame();
    let side = t.g.add_to_sideboard(P0, vec![card("Brago's Favor")]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    name(&mut t, P0, "Grizzly Bears");
    t.g.start();
    let named_at_start = t
        .asked()
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::NameCard { .. }))
        .count();
    assert_eq!(named_at_start, 1);
    assert_eq!(chosen_names(&t.g, side[0]), vec!["Grizzly Bears"]);
    // Turning it face up asks for no name; it keeps the one chosen.
    t.g.run_until(10_000, |g| {
        g.turn.active == P0 && g.turn.priority == Some(P0) && g.turn.step.is_main()
    });
    let before = t.asked().len();
    name(&mut t, P0, "Llanowar Elves");
    turn_face_up(&mut t, P0, side[0]);
    assert!(!t.obj(side[0]).face_down);
    assert!(asked_since(&t, before)
        .iter()
        .all(|(_, d)| !matches!(d, Decision::NameCard { .. })));
    assert_eq!(chosen_names(&t.g, side[0]), vec!["Grizzly Bears"]);
}
