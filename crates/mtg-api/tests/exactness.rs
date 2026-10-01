//! The priority options offered to an agent are complete and exact: every listed action
//! can be taken, and no action that isn't listed can be (CR 117.1).

mod common;

use mtg_api::legal::sandbox;
use mtg_api::request::{prepare, PresentOptions};
use mtg_engine::ability::AbilityKind;
use mtg_engine::agents::RandomAgent;
use mtg_engine::decision::{Action, Agent, Answer, Decision, PassiveAgent, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

/// Whether `p` can take `action` in some way: tried on copies of the game with the
/// engine's default choices and with random choices (other seeds than the ones the
/// option list was checked with).
fn takeable(g: &Game, p: PlayerId, a: &Action, random_tries: u64) -> bool {
    if matches!(a, Action::Pass | Action::Concede) {
        return true;
    }
    let mut copy = sandbox(g, |_| Box::new(PassiveAgent));
    if copy.perform_action(p, a.clone()).is_ok() {
        return true;
    }
    (0..random_tries).any(|seed| {
        let mut copy = sandbox(g, |q| {
            Box::new(RandomAgent::new(9_000 + seed * 7 + q.0 as u64))
        });
        copy.perform_action(p, a.clone()).is_ok()
    })
}

/// Actions a player might try: casting each card they could see with each way of
/// casting it, playing each card as a land, activating each activated ability of each
/// object, and the special actions for each card.
fn candidates(g: &Game) -> Vec<Action> {
    let mut out = Vec::new();
    for (i, o) in g.objects.iter().enumerate() {
        let id = ObjectId(i as u32);
        if !g.is_live(id) {
            continue;
        }
        if matches!(o.zone, Zone::Library(_)) && g.library_top(o.owner) != Some(id) {
            continue;
        }
        if o.zone != Zone::Battlefield && o.zone != Zone::Stack {
            let mut methods = vec![
                CastMethod::Normal,
                CastMethod::Free,
                CastMethod::FaceDown(KeywordKind::Morph),
            ];
            for k in o.chars.keywords() {
                methods.push(CastMethod::Keyword(k.kind));
            }
            if let Some(c) = &o.card {
                for f in 0..c.faces.len() {
                    methods.push(CastMethod::Half(f as u8));
                }
            }
            for m in methods {
                out.push(Action::Cast {
                    card: id,
                    method: m,
                });
            }
            out.push(Action::PlayLand { card: id });
            out.push(Action::Special(SpecialAction::Suspend { card: id }));
            out.push(Action::Special(SpecialAction::Foretell { card: id }));
            out.push(Action::Special(SpecialAction::Plot { card: id }));
        }
        if o.zone == Zone::Battlefield {
            out.push(Action::Special(SpecialAction::TurnFaceUp { obj: id }));
        }
        for a in &o.chars.abilities {
            if matches!(a.kind, AbilityKind::Activated(_)) {
                out.push(Action::Activate {
                    source: id,
                    ability: a.uid,
                });
            }
        }
    }
    out
}

/// Checks the options a priority request offers `p`; returns the problems found.
fn check(g: &Game, p: PlayerId, listed: &[Action]) -> Vec<String> {
    let req = prepare(
        g,
        p,
        p,
        &Decision::Priority {
            actions: listed.to_vec(),
        },
        1,
        &PresentOptions::default(),
    );
    let offered: Vec<Action> = req
        .request
        .options
        .iter()
        .map(
            |o| match req.convert(g, &mtg_api::JsonAnswer::index(o.index)) {
                Ok(Answer::Action(a)) => a,
                other => panic!("option {} isn't an action: {other:?}", o.index),
            },
        )
        .collect();
    let mut problems = Vec::new();
    for (a, o) in offered.iter().zip(&req.request.options) {
        if !takeable(g, p, a, 6) {
            problems.push(format!("listed but can't be taken: {} ({a:?})", o.text));
        }
    }
    for a in candidates(g) {
        if offered.contains(&a) {
            continue;
        }
        if takeable(g, p, &a, 2) {
            problems.push(format!(
                "can be taken but isn't listed: {} ({a:?})",
                mtg_api::describe::action_text(g, Some(p), &a)
            ));
        }
    }
    // Pass is always there, first; conceding last.
    assert!(matches!(offered.first(), Some(Action::Pass)));
    assert!(matches!(offered.last(), Some(Action::Concede)));
    problems
}

fn check_now(t: &mut TestGame, p: PlayerId) -> Vec<Action> {
    let listed = t.g.legal_actions(p);
    let problems = check(&t.g, p, &listed);
    assert!(problems.is_empty(), "{problems:#?}");
    let req = prepare(
        &t.g,
        p,
        p,
        &Decision::Priority { actions: listed },
        1,
        &PresentOptions::default(),
    );
    req.request
        .options
        .iter()
        .map(
            |o| match req.convert(&t.g, &mtg_api::JsonAnswer::index(o.index)) {
                Ok(Answer::Action(a)) => a,
                _ => unreachable!(),
            },
        )
        .collect()
}

#[test]
fn main_phase_options_are_exact() {
    cr!("117.1a", "117.1b", "117.1d", "305.2", "104.3a");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Mountain", 1);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let bolt = t.hand(P0, "Lightning Bolt");
    let bears = t.hand(P0, "Grizzly Bears");
    let wurm = t.hand(P0, "Craw Wurm");
    let forest = t.hand(P0, "Forest");
    t.battlefield(P1, "Grizzly Bears");
    t.hand(P1, "Shock");
    let opts = check_now(&mut t, P0);
    let has_cast = |c: ObjectId| {
        opts.iter()
            .any(|a| matches!(a, Action::Cast { card, .. } if *card == c))
    };
    assert!(has_cast(bolt) && has_cast(bears));
    assert!(!has_cast(wurm), "Craw Wurm costs 6");
    assert!(opts.contains(&Action::PlayLand { card: forest }));
    // Mana abilities (CR 117.1d) and the pyromancer's ability (CR 117.1b).
    assert!(opts
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == lands[0])));
    assert!(opts
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == pyro)));
    // The elves are summoning sick: their mana ability needs {T} (CR 302.6).
    let _ = elves;
    // After a land play, no more lands (CR 305.2).
    t.g.players[0].lands_played_this_turn = 1;
    let opts = check_now(&mut t, P0);
    assert!(!opts.iter().any(|a| matches!(a, Action::PlayLand { .. })));
}

#[test]
fn instant_speed_options_on_an_opponents_turn_are_exact() {
    cr!("117.1a", "307.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bears = t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Mountain");
    t.set_step(P1, Step::Upkeep);
    t.g.turn.priority = Some(P0);
    let opts = check_now(&mut t, P0);
    assert!(opts
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == bolt)));
    assert!(!opts
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == bears)));
    assert!(!opts.iter().any(|a| matches!(a, Action::PlayLand { .. })));
}

#[test]
fn alternative_casts_and_special_actions_are_exact() {
    cr!("116.2b", "702.34a", "702.143a", "702.62a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Mountain", 2);
    let twice = t.graveyard(P0, "Think Twice");
    t.hand(P0, "Saw It Coming");
    t.hand(P0, "Rift Bolt");
    t.hand(P0, "Bonecrusher Giant");
    t.hand(P0, "Lonely Sandbar");
    let den = t.battlefield(P0, "Den Protector");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, den));
    t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Liliana of the Veil");
    t.g.recompute();
    let opts = check_now(&mut t, P0);
    assert!(opts.iter().any(
        |a| matches!(a, Action::Cast { card, method: CastMethod::Keyword(KeywordKind::Flashback) } if *card == twice)
    ));
    assert!(opts
        .iter()
        .any(|a| matches!(a, Action::Special(SpecialAction::Foretell { .. }))));
}

/// Wraps a random agent; on a sample of its priority decisions, checks the options.
struct Checker {
    inner: RandomAgent,
    every: u32,
    seen: u32,
    problems: Arc<Mutex<Vec<String>>>,
    checked: Arc<Mutex<u32>>,
}

impl Agent for Checker {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if let Decision::Priority { actions } = d {
            self.seen += 1;
            if self.seen % self.every == 0 {
                *self.checked.lock().unwrap() += 1;
                let found = check(g, p, actions);
                self.problems.lock().unwrap().extend(
                    found
                        .into_iter()
                        .map(|f| format!("turn {}: {f}", g.turn.number)),
                );
            }
        }
        self.inner.decide(g, p, d)
    }
}

#[test]
fn options_are_exact_in_random_games() {
    cr!("117.1", "733.1");
    let problems: Arc<Mutex<Vec<String>>> = Arc::default();
    let checked: Arc<Mutex<u32>> = Arc::default();
    for game in 0..3u64 {
        let decks = vec![
            common::random_deck(100 + game * 2),
            common::random_deck(101 + game * 2),
        ];
        let agents: Vec<Box<dyn Agent>> = (0..2)
            .map(|i| {
                Box::new(Checker {
                    inner: RandomAgent::new(game * 10 + i),
                    every: 7,
                    seen: 0,
                    problems: problems.clone(),
                    checked: checked.clone(),
                }) as Box<dyn Agent>
            })
            .collect();
        let mut g = Game::new(
            GameConfig {
                seed: game,
                max_turns: 8,
                ..Default::default()
            },
            decks,
            agents,
        );
        g.run();
    }
    let problems = problems.lock().unwrap();
    assert!(
        *checked.lock().unwrap() >= 10,
        "{}",
        checked.lock().unwrap()
    );
    assert!(problems.is_empty(), "{problems:#?}");
}
