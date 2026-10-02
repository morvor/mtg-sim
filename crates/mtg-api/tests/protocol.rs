//! The protocol end to end: invalid answers, the session API, and complete games played
//! by a child process through newline-delimited JSON.

mod common;

use mtg_api::agent::{AgentStats, Closed, GameOver, ProtocolAgent, ProtocolOptions, Transport};
use mtg_api::session::{Seat, Session, SessionError};
use mtg_api::{JsonAnswer, Request};
use mtg_engine::agents::RandomAgent;
use mtg_engine::decision::{Action, Agent, Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A transport answering from a script, recording the requests.
#[derive(Clone, Default)]
struct Scripted {
    answers: Arc<Mutex<VecDeque<String>>>,
    seen: Arc<Mutex<Vec<Request>>>,
}

impl Transport for Scripted {
    fn ask(&mut self, req: &Request) -> Result<Result<JsonAnswer, String>, Closed> {
        self.seen.lock().unwrap().push(req.clone());
        match self.answers.lock().unwrap().pop_front() {
            Some(line) => Ok(JsonAnswer::parse(&line)),
            None => Err(Closed("script over".into())),
        }
    }
}

fn scripted(lines: &[&str]) -> Scripted {
    Scripted {
        answers: Arc::new(Mutex::new(lines.iter().map(|s| s.to_string()).collect())),
        seen: Arc::default(),
    }
}

#[test]
fn invalid_answers_get_an_error_and_are_asked_again() {
    cr!("117.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    let actions = t.g.legal_actions(P0);
    let s = scripted(&[
        "not json",
        r#"{"index": 99}"#,
        r#"{"indices": [1]}"#,
        r#"{"index": 0, "bool": true}"#,
        r#"{"id": 12345, "index": 0}"#,
        r#"{"index": 1}"#,
    ]);
    let mut agent = ProtocolAgent::new(
        P0,
        s.clone(),
        ProtocolOptions {
            max_retries: 5,
            ..Default::default()
        },
    );
    let answer = agent.decide(&t.g, P0, &Decision::Priority { actions });
    let seen = s.seen.lock().unwrap();
    assert_eq!(seen.len(), 6);
    assert_eq!(seen[0].error, None);
    assert_eq!(seen[0].attempt, 0);
    let errors: Vec<&str> = seen[1..]
        .iter()
        .map(|r| r.error.as_deref().unwrap())
        .collect();
    assert!(errors[0].contains("invalid answer JSON"), "{errors:?}");
    assert!(errors[1].contains("out of range"), "{errors:?}");
    assert!(errors[2].contains("expects"), "{errors:?}");
    assert!(errors[3].contains("more than one"), "{errors:?}");
    assert!(errors[4].contains("doesn't match"), "{errors:?}");
    assert!(seen.iter().all(|r| r.id == seen[0].id));
    assert_eq!(seen[5].attempt, 5);
    // Option 1 is the first action after "pass".
    let first = seen[0].options[1].action.clone().unwrap();
    assert!(matches!(answer, Answer::Action(_)));
    assert_ne!(answer, Answer::Action(Action::Pass));
    assert_eq!(
        agent.stats,
        AgentStats {
            decisions: 1,
            requests: 6,
            invalid_answers: 5,
            fallbacks: 0
        }
    );
    let _ = (first, bolt);
}

#[test]
fn too_many_invalid_answers_fall_back_to_the_default() {
    let t = TestGame::new(2);
    let s = scripted(&[r#"{"index": 7}"#, r#"{"index": 8}"#, r#"{"index": 9}"#]);
    let mut agent = ProtocolAgent::new(
        P0,
        s.clone(),
        ProtocolOptions {
            max_retries: 2,
            ..Default::default()
        },
    );
    let d = Decision::YesNo {
        source: None,
        prompt: "Scry 1?".into(),
    };
    assert_eq!(agent.decide(&t.g, P0, &d), Answer::Default);
    assert_eq!(s.seen.lock().unwrap().len(), 3);
    assert_eq!(agent.stats.fallbacks, 1);
    // A closed transport: defaults from then on, without asking.
    assert_eq!(agent.decide(&t.g, P0, &d), Answer::Default);
    assert!(agent.closed().is_some());
    assert_eq!(agent.decide(&t.g, P0, &d), Answer::Default);
    assert_eq!(s.seen.lock().unwrap().len(), 4);
}

#[test]
fn illegal_attacks_are_rejected_with_the_reason() {
    cr!("508.1d");
    let mut t = TestGame::new(2);
    // Must attack each combat if able.
    let berserker = t.battlefield(P0, "Bloodrock Cyclops");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, mtg_engine::turn::Step::DeclareAttackers);
    let options = mtg_engine::combat::attack_options(&t.g);
    let d = Decision::DeclareAttackers { options };
    let p = mtg_api::prepare(&t.g, P0, P0, &d, 1, &Default::default());
    let idx = |c: ObjectId| {
        p.request
            .options
            .iter()
            .find(|o| o.group == Some(c.0))
            .unwrap()
            .index
    };
    // The cyclops's option is marked as obeying a requirement, the bears' isn't.
    assert_eq!(p.request.options[idx(berserker)].required, Some(true));
    assert_eq!(p.request.options[idx(bears)].required, None);
    // Attacking with the bears alone leaves the cyclops home: illegal (CR 508.1d).
    let err = p
        .convert(&t.g, &JsonAnswer::indices(vec![idx(bears)]))
        .unwrap_err();
    assert!(err.contains("508.1"), "{err}");
    assert!(p
        .convert(&t.g, &JsonAnswer::indices(vec![idx(berserker)]))
        .is_ok());
    // The same creature twice: one option per creature.
    let both: Vec<usize> = p
        .request
        .options
        .iter()
        .filter(|o| o.group == Some(bears.0))
        .map(|o| o.index)
        .collect();
    if both.len() > 1 {
        assert!(p.convert(&t.g, &JsonAnswer::indices(both)).is_err());
    }
}

/// Plays a session to its end, answering every external decision at random.
fn drive(s: &mut Session, seed: u64) -> (u32, u32) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let (mut asked, mut errors) = (0, 0);
    let mut last_id = 0;
    while let Some(req) = s.next() {
        asked += 1;
        if req.error.is_some() {
            errors += 1;
        } else {
            assert_ne!(req.id, last_id, "a new decision has a new id");
        }
        last_id = req.id;
        assert!(req.observation.is_some());
        let a = common::random_answer(&req, &mut rng);
        s.answer(PlayerId(req.seat), a).unwrap();
        assert!(asked < 200_000, "game doesn't end");
    }
    (asked, errors)
}

#[test]
fn a_session_drives_a_whole_game() {
    let config = GameConfig {
        seed: 3,
        max_turns: 30,
        ..Default::default()
    };
    let mut s = Session::start(
        config,
        vec![common::simple_deck(), common::simple_deck()],
        vec![Seat::External, Seat::External],
        ProtocolOptions::default(),
    );
    // Answering for the wrong seat is refused.
    let first = s.next().unwrap();
    let other = PlayerId(1 - first.seat);
    assert!(matches!(
        s.answer(other, JsonAnswer::index(0)),
        Err(SessionError::WrongSeat { .. })
    ));
    // The same pending request until it's answered.
    assert_eq!(s.next().unwrap().id, first.id);
    // An unparsable answer comes back as an error on the same decision.
    s.answer_json(PlayerId(first.seat), "{nope").unwrap();
    let again = s.next().unwrap();
    assert_eq!(again.id, first.id);
    assert!(again.error.as_deref().unwrap().contains("JSON"));
    let (asked, _) = drive(&mut s, 1);
    assert!(asked > 50);
    assert!(s.is_over());
    assert!(s.result().is_some());
    assert_eq!(s.game_over().len(), 2);
    assert!(s.next().is_none());
    assert_eq!(
        s.answer(PlayerId(0), JsonAnswer::index(0)),
        Err(SessionError::GameOver)
    );
}

#[test]
fn a_session_mixes_external_and_in_process_seats() {
    let decks = vec![common::random_deck(7), common::random_deck(8)];
    let mut s = Session::start(
        GameConfig {
            seed: 9,
            max_turns: 20,
            ..Default::default()
        },
        decks,
        vec![Seat::Agent(Box::new(RandomAgent::new(1))), Seat::External],
        ProtocolOptions {
            max_retries: 2,
            ..Default::default()
        },
    );
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let mut n = 0;
    let mut saw_events = false;
    while let Some(req) = s.next() {
        assert_eq!(req.seat, 1);
        saw_events |= req.events.iter().any(|e| e.kind == "turn");
        n += 1;
        s.answer(PlayerId(1), common::random_answer(&req, &mut rng))
            .unwrap();
    }
    assert!(n > 10);
    assert!(saw_events, "the event feed reports turns");
    assert!(s.result().is_some());
}

#[test]
fn dropping_a_session_stops_its_game() {
    let mut s = Session::start(
        GameConfig::default(),
        vec![common::simple_deck(), common::simple_deck()],
        vec![Seat::External, Seat::External],
        ProtocolOptions::default(),
    );
    assert!(s.next().is_some());
    drop(s);
}

/// The example client, if Python is available.
fn python() -> Option<String> {
    for p in ["python3", "python"] {
        if std::process::Command::new(p)
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
        {
            return Some(p.to_string());
        }
    }
    None
}

fn client_path() -> String {
    format!(
        "{}/../../examples/random_client.py",
        env!("CARGO_MANIFEST_DIR")
    )
}

/// Records each seat's agent stats after the game.
struct Shared<T: Transport>(Arc<Mutex<ProtocolAgent<T>>>);

impl<T: Transport> Agent for Shared<T> {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        self.0.lock().unwrap().decide(g, p, d)
    }
}

#[test]
fn child_processes_play_complete_games_through_json() {
    let Some(py) = python() else {
        eprintln!("python3 not found: skipping the child process test");
        return;
    };
    let dir = std::env::temp_dir().join(format!("mtg-api-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let transcript = dir.join("transcript.txt");
    for game in 0..2u64 {
        let mut agents = Vec::new();
        let mut conns = Vec::new();
        for seat in 0..2u8 {
            let cmd = format!(
                "MTG_CLIENT_SEED={} {py} {}",
                game * 2 + seat as u64,
                client_path()
            );
            let (agent, conn) =
                mtg_api::spawn_external(&cmd, PlayerId(seat), ProtocolOptions::default()).unwrap();
            conn.transcript(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&transcript)
                    .unwrap(),
            );
            let shared = Arc::new(Mutex::new(agent));
            agents.push(Box::new(Shared(shared.clone())) as Box<dyn Agent>);
            conns.push((shared, conn));
        }
        let decks = if game == 0 {
            vec![common::simple_deck(), common::simple_deck()]
        } else {
            vec![common::random_deck(40), common::random_deck(41)]
        };
        let mut g = Game::new(
            GameConfig {
                seed: 50 + game,
                max_turns: 25,
                ..Default::default()
            },
            decks,
            agents,
        );
        mtg_api::prepare_game(&mut g);
        let result = g.run();
        for (seat, (agent, mut conn)) in conns.into_iter().enumerate() {
            let a = agent.lock().unwrap();
            assert!(a.closed().is_none(), "seat {seat}: {:?}", a.closed());
            assert!(a.stats.requests > 10);
            // Random legal options are almost always accepted; rule-checked choices
            // (blocking requirements) can be refused now and then.
            assert!(
                a.stats.invalid_answers * 20 <= a.stats.requests,
                "seat {seat}: {:?}",
                a.stats
            );
            drop(a);
            conn.game_over(&GameOver::new(&g, PlayerId(seat as u8)));
            conn.shut_down();
        }
        let _ = result;
    }
    // Every line of the transcript is one JSON message.
    let text = std::fs::read_to_string(&transcript).unwrap();
    let mut sent = 0;
    let mut over = 0;
    for line in text.lines() {
        let (tag, json) = line.split_once(' ').unwrap();
        let v: serde_json::Value = serde_json::from_str(json).unwrap();
        if tag.starts_with('>') {
            sent += 1;
            if v["type"] == "game_over" {
                over += 1;
            } else {
                assert_eq!(v["type"], "decision");
                let _: Request = serde_json::from_value(v).unwrap();
            }
        } else {
            let _: JsonAnswer = serde_json::from_value(v).unwrap();
        }
    }
    assert_eq!(over, 4);
    assert!(sent > 100);
    let _ = std::fs::remove_dir_all(&dir);
}
