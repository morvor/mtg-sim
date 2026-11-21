//! Shared helpers for the tests of rulings batch P041 (`r_p041_*.rs`): damage prevention
//! (CR 615), the order of prevention and replacement effects (CR 616.1), damage that
//! can't be prevented (CR 615.12), and Gideon-style planeswalkers that become creatures.
//! (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Agent, Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

pub use crate::r_p125_common::{at_p1, exile_now, fight_it_out};
pub use crate::r_s01_common::{attack_with, block_and_finish, supported};
pub use crate::r_s20_common::to_beginning_of_combat;
pub use crate::r_s25_common::{cast_new, lands_for_cost};
pub use crate::r_s29_common::{damage_marked, divide, replacement_choosers};

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// `p` casts the real card `name` (with lands for its cost) at the targets, and
/// everything on the stack resolves.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let s = cast_new(t, p, name, targets);
    t.resolve_all();
    s
}

/// `source` deals `n` (noncombat) damage to `to`, and the game settles.
pub fn damage(t: &mut TestGame, source: ObjectId, to: Entity, n: u32) {
    let source = t.g.current(source);
    t.g.deal_damage(source, to, n, false);
    t.g.flush_events();
    t.settle();
}

/// Several damage events dealt simultaneously, and the game settles.
pub fn damage_at_once(t: &mut TestGame, events: &[(ObjectId, Entity, u32)], combat: bool) {
    let events = events
        .iter()
        .map(|(s, e, n)| (t.g.current(*s), *e, *n))
        .collect();
    t.g.deal_damage_batch(events, combat);
    t.g.flush_events();
    t.settle();
}

/// The damage events of this turn: (source, recipient, amount).
pub fn damage_events(t: &TestGame) -> Vec<(ObjectId, Entity, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                ..
            } => Some((*source, *target, *amount)),
            _ => None,
        })
        .collect()
}

/// The total damage dealt this turn to `to`.
pub fn damage_dealt_to(t: &TestGame, to: Entity) -> u32 {
    damage_events(t)
        .iter()
        .filter(|(_, e, _)| *e == to)
        .map(|(_, _, n)| *n)
        .sum()
}

/// Sets the planeswalker's loyalty.
pub fn set_loyalty(t: &mut TestGame, pw: ObjectId, n: u32) {
    let pw = t.g.current(pw);
    t.g.objects[pw.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), n);
    t.g.dirty = true;
    t.g.recompute();
}

/// Activates the `index`th activated ability of `source` (it stays on the stack).
pub fn activate(t: &mut TestGame, p: PlayerId, source: ObjectId, index: usize, targets: &[Entity]) {
    let source = t.g.current(source);
    t.activate(p, source, index, targets)
        .unwrap_or_else(|e| panic!("ability {index}: {e:?}"));
}

/// Activates the ability and resolves the stack.
pub fn activate_resolve(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    index: usize,
    targets: &[Entity],
) {
    activate(t, p, source, index, targets);
    t.resolve_all();
}

type Pick = Box<dyn FnMut(&Game, &Decision) -> Option<Answer> + Send>;

/// Replaces `p`'s agent by one that answers with `f` when it returns an answer, and
/// otherwise as before (scripted answers and defaults). Answers given by `f` are logged
/// in `t.asked()`.
pub fn respond_with(t: &mut TestGame, p: PlayerId, f: Pick) {
    struct Respond {
        inner: Box<dyn Agent>,
        f: Pick,
        script: std::sync::Arc<std::sync::Mutex<Script>>,
    }
    impl Agent for Respond {
        fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
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

/// `p` applies first, among replacement and prevention effects (CR 616.1), the one whose
/// description contains `needle` (else the first one offered). The options offered are
/// recorded in the returned log.
pub fn apply_first(t: &mut TestGame, p: PlayerId, needle: &'static str) -> OptionsLog {
    let log: OptionsLog = Default::default();
    let l = log.clone();
    respond_with(
        t,
        p,
        Box::new(move |_, d| match d {
            Decision::ChooseReplacement { options } => {
                l.lock().unwrap().push(options.clone());
                Some(Answer::Index(
                    options.iter().position(|o| o.contains(needle)).unwrap_or(0),
                ))
            }
            _ => None,
        }),
    );
    log
}

/// `p` applies first the replacement or prevention effect whose description doesn't
/// contain `needle`.
pub fn apply_first_not(t: &mut TestGame, p: PlayerId, needle: &'static str) -> OptionsLog {
    let log: OptionsLog = Default::default();
    let l = log.clone();
    respond_with(
        t,
        p,
        Box::new(move |_, d| match d {
            Decision::ChooseReplacement { options } => {
                l.lock().unwrap().push(options.clone());
                Some(Answer::Index(
                    options
                        .iter()
                        .position(|o| !o.contains(needle))
                        .unwrap_or(0),
                ))
            }
            _ => None,
        }),
    );
    log
}

/// The option lists of the replacement-order choices answered through `apply_first`.
pub type OptionsLog = std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>;

/// The number of replacement-order choices asked of `p` since decision `from`.
pub fn replacement_choices_of(t: &TestGame, p: PlayerId, from: usize) -> usize {
    replacement_choosers(t, from)
        .iter()
        .filter(|q| **q == p)
        .count()
}
