//! Shared helpers for the keyword action tests (CR 701).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::{CardDef, Layout};
use mtg_engine::decision::Answer;
use mtg_engine::eval::Ctx;
use mtg_engine::object::{Characteristics, ChosenMode};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// A card compiled from oracle text with the real compiler; panics if any of the text
/// isn't understood.
pub fn oracle_card(
    name: &str,
    type_line: &str,
    cost: &str,
    pt: Option<(i32, i32)>,
    text: &str,
) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let p = pt.map(|x| x.0.to_string());
    let tt = pt.map(|x| x.1.to_string());
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: p.as_deref(),
        toughness: tt.as_deref(),
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(
        compiled.unsupported.is_empty(),
        "{name}: unsupported text {:?}",
        compiled.unsupported
    );
    let m = mtg_engine::mana::ManaCost::parse(cost);
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        colors: m.as_ref().map_or(ColorSet::NONE, |m| m.colors()),
        mana_cost: m,
        supertypes: tl.supertypes,
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        abilities: compiled.abilities,
        power: pt.map(|x| x.0),
        toughness: pt.map(|x| x.1),
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

/// Asserts that the real card is fully supported by the oracle compiler.
pub fn supported(name: &str) {
    let d = mtg_engine::card::card(name);
    assert!(
        d.is_fully_supported(),
        "{name}: unsupported {:?}",
        d.unsupported_text()
    );
}

/// Performs `effect` as if a resolving ability controlled by `p` (with source `source`)
/// did, then checks for triggers.
pub fn run(t: &mut TestGame, p: PlayerId, source: Option<ObjectId>, effect: Effect) {
    let mut ctx = Ctx::new(source, p);
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// Like [`run`], with targets chosen for slot 0 (and more slots after it).
pub fn run_targeted(
    t: &mut TestGame,
    p: PlayerId,
    source: Option<ObjectId>,
    effect: Effect,
    targets: Vec<Vec<Entity>>,
) {
    let mut ctx = Ctx::new(source, p);
    ctx.targets = targets;
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// The top object of the stack.
pub fn top(t: &TestGame) -> ObjectId {
    *t.g.stack.last().expect("empty stack")
}

/// The modes and targets chosen for a spell or ability on the stack.
pub fn chosen(t: &TestGame, id: ObjectId) -> Vec<ChosenMode> {
    t.g.obj(id)
        .stack
        .as_deref()
        .map(|si| si.chosen.clone())
        .unwrap_or_default()
}

/// Declares attackers for the active player and advances to the declare attackers step,
/// with the active player holding priority.
pub fn attack_with(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    if t.g.turn.step != Step::BeginningOfCombat {
        t.set_step(ap, Step::BeginningOfCombat);
    }
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "attackers weren't declared");
}

/// Empties a player's library.
pub fn clear_library(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].library.clear();
}

/// A log shared between a test and a [`Spy`] agent.
pub type Probe = std::sync::Arc<std::sync::Mutex<Vec<String>>>;

type SpyFn = Box<
    dyn Fn(&mtg_engine::game::Game, PlayerId, &mtg_engine::decision::Decision) -> Option<String>
        + Send,
>;

/// An agent that records what it sees (via `f`) whenever it's asked something, then
/// answers like the test harness's scripted agent.
pub struct Spy {
    inner: ScriptedAgent,
    f: SpyFn,
    log: Probe,
}

impl mtg_engine::decision::Agent for Spy {
    fn decide(
        &mut self,
        g: &mtg_engine::game::Game,
        p: PlayerId,
        d: &mtg_engine::decision::Decision,
    ) -> Answer {
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
    f: impl Fn(&mtg_engine::game::Game, PlayerId, &mtg_engine::decision::Decision) -> Option<String>
        + Send
        + 'static,
) -> Probe {
    let log: Probe = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
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
