//! Shared helpers for the CR 114–123 tests (emblems, targets, special actions, priority,
//! costs, life, damage, drawing, counters, stickers).

#![allow(dead_code)]

pub use super::r600_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Agent, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::CardDef;
use std::sync::{Arc, Mutex};

/// A log shared between a test and a [`Spy`] agent.
pub type Probe = Arc<Mutex<Vec<String>>>;

type SpyFn = Box<dyn Fn(&Game, PlayerId, &Decision) -> Option<String> + Send>;

/// An agent that records what it sees (via `f`) whenever it's asked something, then
/// answers like the test harness's scripted agent.
pub struct Spy {
    inner: ScriptedAgent,
    f: SpyFn,
    log: Probe,
}

impl Agent for Spy {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if let Some(s) = (self.f)(g, p, d) {
            self.log.lock().unwrap().push(s);
        }
        self.inner.decide(g, p, d)
    }
}

/// Replaces player `p`'s agent with a [`Spy`] that logs `f`'s observations. Scripted
/// answers queued on the test game still apply.
pub fn spy(
    t: &mut TestGame,
    p: PlayerId,
    f: impl Fn(&Game, PlayerId, &Decision) -> Option<String> + Send + 'static,
) -> Probe {
    let log: Probe = Arc::new(Mutex::new(vec![]));
    let agent = Spy {
        inner: ScriptedAgent {
            player: p,
            script: t.script.clone(),
        },
        f: Box::new(f),
        log: log.clone(),
    };
    t.g.set_agent(p, Box::new(agent));
    log
}

pub fn probe_lines(p: &Probe) -> Vec<String> {
    p.lock().unwrap().clone()
}

/// Queues a priority action for a player.
pub fn queue_action(t: &mut TestGame, p: PlayerId, a: Action) {
    t.answer(p, DecisionKind::Priority, Answer::Action(a));
}

/// Mana-free filler spells for timing tests.
pub fn free_instant(name: &str) -> CardDef {
    CB::new(name)
        .instant()
        .cost("{0}")
        .spell(Body::effect(gain(1)))
        .build()
}

pub fn free_sorcery(name: &str) -> CardDef {
    CB::new(name)
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(gain(1)))
        .build()
}

/// All events of the current turn that match `f`.
pub fn events_matching(
    t: &TestGame,
    f: impl Fn(&mtg_engine::events::Event) -> bool,
) -> Vec<mtg_engine::events::Event> {
    t.turn_events
        .iter()
        .chain(t.events.iter())
        .filter(|e| f(e))
        .cloned()
        .collect()
}
