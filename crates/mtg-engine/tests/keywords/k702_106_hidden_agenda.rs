//! CR 702.106 Hidden agenda (and double agenda).

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::untapped_lands;
use mtg_engine::card::card;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::events::Event;
use mtg_engine::facedown::{can_look_at, REVEALED};
use mtg_engine::game::{Game, GameConfig};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::hidden_agenda::{as_put_into_command_zone, chosen_names};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A game that hasn't started yet (so conspiracies can be put into the command zone).
fn pregame(n: usize) -> TestGame {
    let script = Arc::new(Mutex::new(Script {
        queues: vec![VecDeque::new(); n],
        asked: vec![],
    }));
    let agents: Vec<Box<dyn Agent>> = (0..n)
        .map(|i| {
            Box::new(ScriptedAgent {
                player: PlayerId(i as u8),
                script: script.clone(),
            }) as Box<dyn Agent>
        })
        .collect();
    let decks = (0..n).map(|_| vec![filler_card(); 40]).collect();
    let config = GameConfig {
        limited: true,
        skip_mulligans: true,
        ..Default::default()
    };
    let mut g = Game::new(config, decks, agents);
    g.logging = true;
    TestGame { g, script }
}

fn name(t: &mut TestGame, p: PlayerId, n: &str) {
    t.answer(p, DecisionKind::Name, Answer::Text(n.into()));
}

/// A conspiracy `p` put into the command zone as the game began, naming `names`.
fn agenda(t: &mut TestGame, p: PlayerId, conspiracy: &str, names: &[&str]) -> ObjectId {
    let c = t.custom(p, (*card(conspiracy)).clone(), Zone::Command);
    for n in names {
        name(t, p, n);
    }
    as_put_into_command_zone(&mut t.g, p, c);
    t.g.recompute();
    c
}

fn specials(t: &mut TestGame, p: PlayerId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s) => Some(s),
            _ => None,
        })
        .collect()
}

fn turn_face_up(t: &mut TestGame, p: PlayerId, c: ObjectId) {
    assert!(specials(t, p).contains(&SpecialAction::TurnFaceUp { obj: c }));
    t.g.take_action(p, Action::Special(SpecialAction::TurnFaceUp { obj: c }));
    t.g.recompute();
}

fn revealed(t: &TestGame, c: ObjectId) -> bool {
    t.g.turn_events.iter().any(|e| {
        matches!(e, Event::Custom { name, obj: Some(o), .. } if name == REVEALED && *o == c)
    })
}

#[test]
fn a_hidden_agenda_goes_face_down_with_a_secretly_chosen_name() {
    cr!("702.106", "702.106a", "702.106b");
    assert_supported("Brago's Favor");
    let mut t = pregame(2);
    let side = t.g.add_to_sideboard(P0, vec![card("Brago's Favor")]);
    let c = side[0];
    t.answer_choose(P0, &[Entity::Object(c)]);
    name(&mut t, P0, "Grizzly Bears");
    t.g.start();
    assert_eq!(t.zone(c), Zone::Command);
    assert!(t.obj(c).face_down);
    // The name is kept with the face-down card, which has no characteristics; only its
    // controller may look at it.
    assert_eq!(chosen_names(&t.g, c), vec!["Grizzly Bears"]);
    assert!(t.obj(c).chars.name.is_empty());
    assert!(can_look_at(&t.g, P0, c));
    assert!(!can_look_at(&t.g, P1, c));
}

#[test]
fn an_invalid_name_leaves_the_choice_undefined() {
    cr!("702.106a");
    let mut t = TestGame::new(2);
    let c = agenda(&mut t, P0, "Brago's Favor", &["Not A Real Card"]);
    assert!(t.obj(c).face_down);
    assert!(chosen_names(&t.g, c).is_empty());
}

#[test]
fn its_controller_may_turn_it_face_up_any_time_they_have_priority() {
    cr!("702.106c");
    let mut t = TestGame::new(2);
    // Brago's Favor: "Spells with the chosen name you cast cost {1} less to cast."
    let c = agenda(&mut t, P0, "Brago's Favor", &["Grizzly Bears"]);
    // Only its controller can; face down, it has no effect.
    assert!(!specials(&mut t, P1).contains(&SpecialAction::TurnFaceUp { obj: c }));
    t.lands(P0, "Forest", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // Even during an opponent's turn, with a spell on the stack.
    t.set_step(P1, Step::Upkeep);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P1).go();
    turn_face_up(&mut t, P0, c);
    assert!(!t.obj(c).face_down);
    assert_eq!(t.obj(c).chars.name, "Brago's Favor");
    assert!(can_look_at(&t.g, P1, c));
    assert_eq!(chosen_names(&t.g, c), vec!["Grizzly Bears"]);
    t.resolve_all();
    // Now the Bears cost {G}.
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, bears).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn the_chosen_name_belongs_to_that_conspiracys_abilities() {
    cr!("702.106d");
    assert_supported("Immediate Action");
    let mut t = TestGame::new(2);
    // Immediate Action: "Creatures you control with the chosen name have haste."
    let a = agenda(&mut t, P0, "Immediate Action", &["Llanowar Elves"]);
    let b = agenda(&mut t, P0, "Brago's Favor", &["Grizzly Bears"]);
    turn_face_up(&mut t, P0, a);
    turn_face_up(&mut t, P0, b);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let elves = t.battlefield_sick(P0, "Llanowar Elves");
    assert!(t.obj(elves).chars.has_keyword(KeywordKind::Haste));
    assert!(!t.obj(bears).chars.has_keyword(KeywordKind::Haste));
    // Brago's Favor makes the Bears (not Hill Giant) cheaper.
    t.lands(P0, "Forest", 3);
    let b2 = t.hand(P0, "Grizzly Bears");
    t.cast(P0, b2).go();
    assert_eq!(untapped_lands(&t, P0), 2);
    t.resolve_all();
    // Hill Giant ({3}{R}) isn't reduced: three lands can't pay for it (the hasty Elves
    // are tapped).
    t.g.objects[elves.0 as usize].tapped = true;
    t.lands(P0, "Mountain", 1);
    let giant = t.hand(P0, "Hill Giant");
    assert!(t.cast(P0, giant).try_go().is_err());
}

#[test]
fn a_chosen_name_replacement_effect_applies_once_face_up() {
    cr!("702.106c", "702.106d");
    assert_supported("Muzzio's Preparations");
    let mut t = TestGame::new(2);
    // Muzzio's Preparations: "Each creature you control with the chosen name enters with
    // an additional +1/+1 counter on it."
    let c = agenda(&mut t, P0, "Muzzio's Preparations", &["Grizzly Bears"]);
    let first = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(first, types::counters::PLUS1), 0);
    turn_face_up(&mut t, P0, c);
    let second = t.enter(P0, "Grizzly Bears");
    let elves = t.enter(P0, "Llanowar Elves");
    let theirs = t.enter(P1, "Grizzly Bears");
    assert_eq!(t.counters(second, types::counters::PLUS1), 1);
    assert_eq!(t.counters(elves, types::counters::PLUS1), 0);
    assert_eq!(t.counters(theirs, types::counters::PLUS1), 0);
}

#[test]
fn face_down_conspiracies_are_revealed_when_their_controller_leaves_the_game() {
    cr!("702.106e");
    let mut t = TestGame::new(3);
    let mine = agenda(&mut t, P0, "Brago's Favor", &["Grizzly Bears"]);
    let theirs = agenda(&mut t, P1, "Immediate Action", &["Llanowar Elves"]);
    let other = agenda(&mut t, P2, "Brago's Favor", &["Hill Giant"]);
    t.g.player_loses(P1);
    t.g.flush_events();
    assert!(revealed(&t, theirs));
    assert!(!revealed(&t, mine));
    assert!(!revealed(&t, other));
    // At the end of the game, all of them are revealed.
    t.g.player_loses(P2);
    t.g.flush_events();
    assert!(t.g.is_over());
    assert!(revealed(&t, mine));
    assert!(revealed(&t, other));
}

#[test]
fn double_agenda_secretly_chooses_two_different_names() {
    cr!("702.106f");
    assert_supported("Summoner's Bond");
    let mut t = TestGame::new(2);
    let c = agenda(
        &mut t,
        P0,
        "Summoner's Bond",
        &["Grizzly Bears", "Llanowar Elves"],
    );
    assert!(t.obj(c).face_down);
    assert_eq!(
        chosen_names(&t.g, c),
        vec!["Grizzly Bears", "Llanowar Elves"]
    );
    // The same name twice counts once.
    let d = agenda(
        &mut t,
        P0,
        "Summoner's Bond",
        &["Grizzly Bears", "Grizzly Bears"],
    );
    assert_eq!(chosen_names(&t.g, d), vec!["Grizzly Bears"]);
}

#[test]
fn summoners_bond_uses_both_chosen_names() {
    cr!("702.106d", "702.106f");
    let mut t = TestGame::new(2);
    // "Whenever you cast a creature spell with one of the chosen names, you may search
    // your library for a creature card with the other chosen name, reveal it, put it into
    // your hand, then shuffle."
    let c = agenda(
        &mut t,
        P0,
        "Summoner's Bond",
        &["Grizzly Bears", "Llanowar Elves"],
    );
    turn_face_up(&mut t, P0, c);
    t.library_top(P0, "Llanowar Elves");
    t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Llanowar Elves"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}
