//! Every engine decision becomes a request that survives a JSON round trip, and a valid
//! JSON answer to it converts to the engine answer it stands for.

mod common;

use mtg_api::request::{decision_kind, prepare, PresentOptions};
use mtg_api::{AnswerSpec, JsonAnswer, Request};
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::collections::BTreeSet;

/// One decision of every kind, with a valid answer and the engine answer it means.
fn samples(t: &mut TestGame) -> Vec<(Decision, JsonAnswer, Answer)> {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    t.lands(P0, "Mountain", 2);
    let o = |id: ObjectId| Entity::Object(id);
    let actions = t.g.legal_actions(P0);
    let cast_bolt = Action::Cast {
        card: bolt,
        method: Default::default(),
    };
    assert!(actions.contains(&cast_bolt));
    vec![
        (
            Decision::Priority {
                actions: actions.clone(),
            },
            // Option 0 is always "pass".
            JsonAnswer::index(0),
            Answer::Action(Action::Pass),
        ),
        (
            Decision::Mulligan { mulligans_taken: 1 },
            JsonAnswer::yes_no(true),
            Answer::Bool(true),
        ),
        (
            Decision::PutOnBottom {
                cards: vec![bolt, shock],
                n: 1,
            },
            JsonAnswer::indices(vec![1]),
            Answer::Entities(vec![o(shock)]),
        ),
        (
            Decision::ChooseModes {
                source: bolt,
                modes: vec!["a".into(), "b".into(), "c".into()],
                min: 1,
                max: 2,
                allow_repeat: false,
                available: vec![0, 2],
                pawprint_budget: None,
            },
            // Option 1 is the second available mode, mode 2.
            JsonAnswer::indices(vec![1, 0]),
            Answer::Indices(vec![2, 0]),
        ),
        (
            Decision::ChooseX {
                source: bolt,
                min: 0,
                max: 5,
            },
            JsonAnswer::number(3),
            Answer::Number(3),
        ),
        (
            Decision::ChooseCastingMethod {
                card: bolt,
                options: vec!["normal".into(), "flashback".into()],
            },
            JsonAnswer::index(1),
            Answer::Index(1),
        ),
        (
            Decision::OptionalCost {
                source: bolt,
                name: "kicker".into(),
                repeatable: false,
            },
            JsonAnswer::index(1),
            Answer::Bool(true),
        ),
        (
            Decision::OptionalCost {
                source: bolt,
                name: "multikicker".into(),
                repeatable: true,
            },
            JsonAnswer::number(2),
            Answer::Number(2),
        ),
        (
            Decision::ChooseTargets {
                source: bolt,
                text: "any target".into(),
                candidates: vec![Entity::Player(P1), o(giant), o(bears)],
                min: 1,
                max: 1,
            },
            JsonAnswer::indices(vec![1]),
            Answer::Entities(vec![o(giant)]),
        ),
        (
            Decision::Divide {
                source: bolt,
                total: 3,
                recipients: vec![o(giant), Entity::Player(P1)],
                min_each: 1,
            },
            JsonAnswer::numbers(vec![2, 1]),
            Answer::Numbers(vec![2, 1]),
        ),
        (
            Decision::YesNo {
                source: Some(bolt),
                prompt: "Draw a card?".into(),
            },
            JsonAnswer::yes_no(false),
            Answer::Bool(false),
        ),
        (
            Decision::ChooseEntities {
                source: None,
                prompt: "Sacrifice a creature".into(),
                candidates: vec![o(bears)],
                min: 1,
                max: 1,
            },
            JsonAnswer::indices(vec![0]),
            Answer::Entities(vec![o(bears)]),
        ),
        (
            Decision::Order {
                prompt: "Order triggers".into(),
                items: vec!["x".into(), "y".into(), "z".into()],
            },
            JsonAnswer::indices(vec![2, 0, 1]),
            Answer::Indices(vec![2, 0, 1]),
        ),
        (
            Decision::ChooseOption {
                source: None,
                prompt: "Choose a color".into(),
                options: vec!["white".into(), "blue".into()],
            },
            JsonAnswer::index(1),
            Answer::Index(1),
        ),
        (
            Decision::ChooseNumber {
                source: None,
                prompt: "Choose a number".into(),
                min: 0,
                max: 10,
            },
            JsonAnswer::number(7),
            Answer::Number(7),
        ),
        (
            Decision::NameCard {
                source: None,
                prompt: "Name a card".into(),
            },
            JsonAnswer::text("Lightning Bolt"),
            Answer::Text("Lightning Bolt".into()),
        ),
        (
            Decision::AssignCombatDamage {
                creature: giant,
                amount: 3,
                recipients: vec![o(bears)],
                lethal: vec![2],
                trample: false,
            },
            JsonAnswer::numbers(vec![3]),
            Answer::Numbers(vec![3]),
        ),
        (
            Decision::Scry {
                cards: vec![bolt, shock],
            },
            JsonAnswer::split(vec![1], vec![0]),
            Answer::Split(vec![shock], vec![bolt]),
        ),
        (
            Decision::Surveil {
                cards: vec![bolt, shock],
            },
            JsonAnswer::split(vec![], vec![0, 1]),
            Answer::Split(vec![], vec![bolt, shock]),
        ),
        (
            Decision::ChooseReplacement {
                options: vec!["first".into(), "second".into()],
            },
            JsonAnswer::index(0),
            Answer::Index(0),
        ),
    ]
}

fn round_trip(req: &Request) {
    let json = serde_json::to_string(req).expect("serialize");
    assert!(!json.contains('\n'), "one request per line");
    let back: Request = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(&back, req, "request round trip: {json}");
}

#[test]
fn every_decision_round_trips_through_json() {
    let mut t = TestGame::new(2);
    let mut cases = samples(&mut t);
    // Combat decisions need a combat.
    let mut c = TestGame::new(2);
    let attacker = c.battlefield(P0, "Grizzly Bears");
    let blocker = c.battlefield(P1, "Hill Giant");
    c.set_step(P0, Step::DeclareAttackers);
    let attack_options = mtg_engine::combat::attack_options(&c.g);
    assert!(!attack_options.is_empty());
    let mut kinds = BTreeSet::new();
    for (d, _, _) in &cases {
        kinds.insert(decision_kind(d));
    }
    let combat_cases = vec![
        (
            Decision::DeclareAttackers {
                options: attack_options.clone(),
            },
            JsonAnswer::indices(vec![0]),
            Answer::Attackers(vec![(attacker, attack_options[0].1[0])]),
        ),
        (
            Decision::DeclareBlockers {
                options: vec![(blocker, vec![attacker])],
            },
            JsonAnswer::indices(vec![]),
            Answer::Blockers(vec![]),
        ),
    ];
    for (d, _, _) in &combat_cases {
        kinds.insert(decision_kind(d));
    }
    // Every variant of `Decision` is covered (`decision_kind` matches them all).
    assert_eq!(kinds.len(), 21, "{kinds:?}");
    let opts = PresentOptions::default();
    for (i, (d, a, expected)) in cases.drain(..).enumerate() {
        // The engine's own decision type round-trips too.
        let dj = serde_json::to_string(&d).unwrap();
        let _: Decision = serde_json::from_str(&dj).unwrap();
        let p = prepare(&t.g, P0, P0, &d, i as u64 + 1, &opts);
        round_trip(&p.request);
        let aj = serde_json::to_string(&a).unwrap();
        let parsed = JsonAnswer::parse(&aj).unwrap();
        assert_eq!(parsed, a);
        assert_eq!(
            p.convert(&t.g, &parsed),
            Ok(expected),
            "{}: {aj}",
            p.request.kind
        );
        // `{"default": true}` is always accepted.
        assert_eq!(
            p.convert(&t.g, &JsonAnswer::default_choice()),
            Ok(Answer::Default)
        );
    }
    for (i, (d, a, expected)) in combat_cases.into_iter().enumerate() {
        let p = prepare(&c.g, P0, P0, &d, 100 + i as u64, &opts);
        round_trip(&p.request);
        let parsed = JsonAnswer::parse(&serde_json::to_string(&a).unwrap()).unwrap();
        assert_eq!(p.convert(&c.g, &parsed), Ok(expected), "{}", p.request.kind);
    }
}

#[test]
fn answer_specs_name_their_shape() {
    let mut t = TestGame::new(2);
    let cases = samples(&mut t);
    let opts = PresentOptions::default();
    for (i, (d, _, _)) in cases.iter().enumerate() {
        let p = prepare(&t.g, P0, P0, d, i as u64, &opts);
        let v: serde_json::Value = serde_json::to_value(&p.request).unwrap();
        assert_eq!(v["type"], "decision");
        assert_eq!(v["version"], mtg_api::PROTOCOL_VERSION);
        assert!(v["answer"]["type"].is_string());
        // Options carry their own index.
        for (k, o) in p.request.options.iter().enumerate() {
            assert_eq!(o.index, k);
            assert!(!o.text.is_empty());
        }
        match (&p.request.answer, d) {
            (AnswerSpec::ChooseOne, Decision::Priority { .. }) => {
                assert!(p.request.options[0].action.as_ref().unwrap().kind == "pass")
            }
            (AnswerSpec::YesNo, _) => assert_eq!(p.request.options.len(), 2),
            _ => {}
        }
    }
}
