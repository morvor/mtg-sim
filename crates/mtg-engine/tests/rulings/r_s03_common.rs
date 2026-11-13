//! Shared helpers for the tests of rulings batch S03 (`r_s03_*.rs`): cipher, clash, cleave,
//! cloak, cohort, collect evidence, companion, compleated, conjure, connive, constellation,
//! converge, convert, convoke, council's dilemma, coven, craft. (The helpers of batches S01
//! and S02, `r_s01_common` and `r_s02_common`, are used too.)

#![allow(dead_code)]

use crate::r_s01_common::give_mana_for;
use mtg_engine::ability::Effect;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// Executes `effect` as a resolving spell or ability controlled by `controller` (with
/// `source`) would, with `targets` as its first target slot.
pub fn run_effect(
    t: &mut TestGame,
    source: Option<ObjectId>,
    controller: PlayerId,
    effect: Effect,
    targets: &[Entity],
) {
    let mut ctx = mtg_engine::eval::Ctx::new(source, controller);
    ctx.targets = vec![targets.to_vec()];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Puts the real card `name` into `p`'s hand with the lands to pay its mana cost, and
/// returns it.
pub fn in_hand_with_mana(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    give_mana_for(t, p, name);
    t.hand(p, name)
}

/// The active player attacks with `attackers`, the defending player declares `blocks`,
/// and the game stops in the declare blockers step with the active player holding
/// priority (blocks declared, triggers on the stack).
pub fn to_blockers(
    t: &mut TestGame,
    attackers: &[(ObjectId, Entity)],
    blocks: &[(ObjectId, ObjectId)],
) {
    let ap = t.g.turn.active;
    if t.g.turn.step != Step::BeginningOfCombat {
        t.set_step(ap, Step::BeginningOfCombat);
    }
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    if !blocks.is_empty() {
        let dp = t.g.obj(blocks[0].0).controller;
        t.answer(
            dp,
            DecisionKind::Blockers,
            Answer::Blockers(blocks.to_vec()),
        );
    }
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareBlockers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "blockers not declared");
    t.settle();
}

/// Wraps `p`'s agent: decisions for which `f` returns an answer get that answer (and are
/// logged like the others); the rest go to the scripted agent.
pub fn respond(
    t: &mut TestGame,
    p: PlayerId,
    f: fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>,
) {
    struct Respond {
        inner: Box<dyn mtg_engine::decision::Agent>,
        f: fn(&mtg_engine::game::Game, &Decision) -> Option<Answer>,
        script: std::sync::Arc<std::sync::Mutex<Script>>,
    }
    impl mtg_engine::decision::Agent for Respond {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            match (self.f)(g, d) {
                Some(a) => {
                    self.script.lock().unwrap().asked.push((p, d.clone()));
                    a
                }
                None => self.inner.decide(g, p, d),
            }
        }
    }
    let script = t.script.clone();
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(Respond { inner, f, script });
}

/// Answers `d` with its first candidate if it's an entity choice whose prompt contains
/// "cipher" (choosing the creature to encode a card on).
pub fn first_cipher_candidate(_g: &mtg_engine::game::Game, d: &Decision) -> Option<Answer> {
    match d {
        Decision::ChooseEntities {
            prompt, candidates, ..
        } if prompt.contains("cipher") => Some(Answer::Entities(
            candidates.first().copied().into_iter().collect(),
        )),
        _ => None,
    }
}

/// The players asked for priority from decision `from` on.
pub fn priority_asked_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Priority { .. }))
        .map(|(p, _)| *p)
        .collect()
}

/// The candidates of the entity choices whose prompt contains `prompt`, asked since
/// decision `from`.
pub fn choice_candidates(t: &TestGame, from: usize, prompt: &str) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt: p,
                candidates,
                ..
            } if p.contains(prompt) => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}
