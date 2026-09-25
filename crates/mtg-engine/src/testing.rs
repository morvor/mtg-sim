//! Test harness for writing rules tests concisely.
//!
//! ```ignore
//! let mut t = TestGame::new(2);
//! let bear = t.battlefield(P1, "Grizzly Bears");
//! let bolt = t.hand(P0, "Lightning Bolt");
//! t.cast(P0, bolt).target(bear).go();
//! t.resolve();
//! assert!(t.in_graveyard(P1, "Grizzly Bears"));
//! ```
//!
//! Decisions are answered by a [`ScriptedAgent`] per player: queued answers are used in
//! order; when the queue is empty, a sensible default is used (pass priority, keep hand,
//! no attackers/blockers, engine defaults otherwise). Queued answers can be *matched* to
//! a kind of decision so that unrelated prompts in between don't consume them.

use crate::ability::*;
use crate::card::{card, CardDef};
use crate::decision::*;
use crate::game::*;
use crate::object::*;
use crate::turn::{Stage, Step, TurnState};
use crate::types::*;
use smol_str::SmolStr;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub const P0: PlayerId = PlayerId(0);
pub const P1: PlayerId = PlayerId(1);
pub const P2: PlayerId = PlayerId(2);
pub const P3: PlayerId = PlayerId(3);

/// Which decisions a queued answer applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionKind {
    Any,
    Priority,
    Targets,
    Entities,
    YesNo,
    Option,
    Modes,
    X,
    Attackers,
    Blockers,
    Damage,
    Order,
    Number,
    Replacement,
    Scry,
    Surveil,
    OptionalCost,
    Divide,
    Mulligan,
    Name,
}

fn kind_of(d: &Decision) -> DecisionKind {
    match d {
        Decision::Priority { .. } => DecisionKind::Priority,
        Decision::ChooseTargets { .. } => DecisionKind::Targets,
        Decision::ChooseEntities { .. } | Decision::PutOnBottom { .. } => DecisionKind::Entities,
        Decision::YesNo { .. } => DecisionKind::YesNo,
        Decision::ChooseOption { .. } | Decision::ChooseCastingMethod { .. } => {
            DecisionKind::Option
        }
        Decision::ChooseModes { .. } => DecisionKind::Modes,
        Decision::ChooseX { .. } => DecisionKind::X,
        Decision::DeclareAttackers { .. } => DecisionKind::Attackers,
        Decision::DeclareBlockers { .. } => DecisionKind::Blockers,
        Decision::AssignCombatDamage { .. } => DecisionKind::Damage,
        Decision::Order { .. } => DecisionKind::Order,
        Decision::ChooseNumber { .. } => DecisionKind::Number,
        Decision::ChooseReplacement { .. } => DecisionKind::Replacement,
        Decision::Scry { .. } => DecisionKind::Scry,
        Decision::Surveil { .. } => DecisionKind::Surveil,
        Decision::OptionalCost { .. } => DecisionKind::OptionalCost,
        Decision::Divide { .. } => DecisionKind::Divide,
        Decision::Mulligan { .. } => DecisionKind::Mulligan,
        Decision::NameCard { .. } => DecisionKind::Name,
    }
}

#[derive(Default)]
pub struct Script {
    pub queues: Vec<VecDeque<(DecisionKind, Answer)>>,
    /// Log of all decisions asked (player, decision) for assertions.
    pub asked: Vec<(PlayerId, Decision)>,
}

/// An agent that answers from a shared script.
pub struct ScriptedAgent {
    pub player: PlayerId,
    pub script: Arc<Mutex<Script>>,
}

impl Agent for ScriptedAgent {
    fn name(&self) -> &str {
        "scripted"
    }

    fn decide(&mut self, _g: &Game, p: PlayerId, d: &Decision) -> Answer {
        let mut s = self.script.lock().unwrap();
        s.asked.push((p, d.clone()));
        let k = kind_of(d);
        let q = &mut s.queues[p.idx()];
        if let Some(i) = q
            .iter()
            .position(|(kind, _)| *kind == k || *kind == DecisionKind::Any)
        {
            let (_, a) = q.remove(i).unwrap();
            return a;
        }
        match d {
            Decision::Priority { .. } => Answer::Action(Action::Pass),
            Decision::Mulligan { .. } => Answer::Bool(false),
            Decision::DeclareAttackers { .. } => Answer::Attackers(vec![]),
            Decision::DeclareBlockers { .. } => Answer::Blockers(vec![]),
            _ => Answer::Default,
        }
    }
}

/// A game plus helpers for tests.
pub struct TestGame {
    pub g: Game,
    pub script: Arc<Mutex<Script>>,
}

impl std::ops::Deref for TestGame {
    type Target = Game;
    fn deref(&self) -> &Game {
        &self.g
    }
}

impl std::ops::DerefMut for TestGame {
    fn deref_mut(&mut self) -> &mut Game {
        &mut self.g
    }
}

/// A card with no characteristics used to fill libraries so drawing doesn't lose.
pub fn filler_card() -> Arc<CardDef> {
    use std::sync::OnceLock;
    static F: OnceLock<Arc<CardDef>> = OnceLock::new();
    F.get_or_init(|| {
        Arc::new(CardDef::custom(Characteristics {
            name: SmolStr::new("Filler"),
            rules_text: Arc::from(""),
            ..Default::default()
        }))
    })
    .clone()
}

impl TestGame {
    /// A game with `n` players at 20 life, each with a 30-card library of fillers, in
    /// player 0's first precombat main phase with priority.
    pub fn new(n: usize) -> TestGame {
        Self::with_config(n, GameConfig::default())
    }

    pub fn with_config(n: usize, config: GameConfig) -> TestGame {
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
        let decks: Vec<Vec<Arc<CardDef>>> = (0..n).map(|_| vec![filler_card(); 30]).collect();
        let mut g = Game::new(config, decks, agents);
        g.logging = true;
        g.turn.number = 1;
        g.turn.starting_player = P0;
        g.turn.active = P0;
        let mut t = TestGame { g, script };
        t.set_step(P0, Step::PrecombatMain);
        t
    }

    /// Jumps to a step of `active`'s turn (no turn-based actions are performed for the
    /// step; the active player has priority).
    pub fn set_step(&mut self, active: PlayerId, step: Step) {
        let g = &mut self.g;
        g.turn.active = active;
        g.turn.step = step;
        g.turn.stage = Stage::Priority;
        g.turn.priority = Some(active);
        g.turn.passes = 0;
        let sched = TurnState::default_schedule();
        g.turn.schedule = match sched.iter().position(|s| *s == step) {
            Some(i) => sched[i + 1..].to_vec(),
            None => vec![],
        };
        if step.is_main() {
            g.turn.main_phases = 1;
        } else if matches!(step, Step::Untap | Step::Upkeep | Step::Draw) {
            // No main phase has begun yet this turn.
            g.turn.main_phases = 0;
        }
        if step.is_combat() && g.combat.is_none() {
            crate::combat::begin_combat(g);
        }
        g.recompute();
    }

    // ------------------------------------------------------------------
    // Creating cards
    // ------------------------------------------------------------------

    fn place(&mut self, p: PlayerId, def: Arc<CardDef>, zone: Zone) -> ObjectId {
        let id = self.g.create_card_object(def, p, zone);
        match zone {
            Zone::Library(q) => self.g.players[q.idx()].library.push(id),
            Zone::Hand(q) => self.g.players[q.idx()].hand.push(id),
            Zone::Graveyard(q) => self.g.players[q.idx()].graveyard.push(id),
            Zone::Battlefield => {
                self.g.battlefield.push(id);
                let o = &mut self.g.objects[id.0 as usize];
                o.controller = p;
                o.base_controller = p;
                o.summoning_sick = false;
            }
            Zone::Exile => self.g.exile.push(id),
            Zone::Command => self.g.command.push(id),
            _ => {}
        }
        self.g.recompute();
        if zone == Zone::Battlefield {
            // Planeswalkers/battles/sagas get their starting counters.
            let o = self.g.obj(id).clone();
            if o.is(CardType::Planeswalker) {
                if let Some(l) = o.chars.loyalty {
                    self.g.objects[id.0 as usize]
                        .counters
                        .insert(counters::LOYALTY.into(), l.max(0) as u32);
                }
            }
            if o.is(CardType::Battle) {
                if let Some(d) = o.chars.defense {
                    self.g.objects[id.0 as usize]
                        .counters
                        .insert(counters::DEFENSE.into(), d.max(0) as u32);
                }
            }
            self.g.recompute();
        }
        id
    }

    /// Puts a real card directly onto the battlefield (no ETB triggers, not summoning sick).
    pub fn battlefield(&mut self, p: PlayerId, name: &str) -> ObjectId {
        self.place(p, card(name), Zone::Battlefield)
    }
    /// Puts a card onto the battlefield summoning sick.
    pub fn battlefield_sick(&mut self, p: PlayerId, name: &str) -> ObjectId {
        let id = self.battlefield(p, name);
        self.g.objects[id.0 as usize].summoning_sick = true;
        id
    }
    pub fn hand(&mut self, p: PlayerId, name: &str) -> ObjectId {
        self.place(p, card(name), Zone::Hand(p))
    }
    pub fn graveyard(&mut self, p: PlayerId, name: &str) -> ObjectId {
        self.place(p, card(name), Zone::Graveyard(p))
    }
    pub fn exile(&mut self, p: PlayerId, name: &str) -> ObjectId {
        self.place(p, card(name), Zone::Exile)
    }
    pub fn library_top(&mut self, p: PlayerId, name: &str) -> ObjectId {
        self.place(p, card(name), Zone::Library(p))
    }
    pub fn command(&mut self, p: PlayerId, name: &str) -> ObjectId {
        self.place(p, card(name), Zone::Command)
    }
    /// Places a custom card definition.
    pub fn custom(&mut self, p: PlayerId, def: CardDef, zone: Zone) -> ObjectId {
        self.place(p, Arc::new(def), zone)
    }
    /// Puts a card onto the battlefield through a real zone change (ETB triggers and
    /// replacement effects apply).
    pub fn enter(&mut self, p: PlayerId, name: &str) -> ObjectId {
        let id = self.g.create_card_object(card(name), p, Zone::Nowhere);
        self.g
            .move_object_ev(crate::replacement::MoveEv {
                obj: id,
                to: Zone::Battlefield,
                pos: LibraryPosition::Top,
                cause: crate::events::MoveCause::Effect,
                by: Some(p),
                etb: crate::replacement::EtbInfo {
                    controller: Some(p),
                    ..Default::default()
                },
                source: None,
            })
            .expect("failed to enter the battlefield")
    }
    /// Adds basic lands (or any cards) to the battlefield for mana.
    pub fn lands(&mut self, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
        (0..n).map(|_| self.battlefield(p, name)).collect()
    }

    // ------------------------------------------------------------------
    // Scripting answers
    // ------------------------------------------------------------------

    pub fn answer(&mut self, p: PlayerId, kind: DecisionKind, a: Answer) -> &mut Self {
        self.script.lock().unwrap().queues[p.idx()].push_back((kind, a));
        self
    }
    pub fn answer_targets(&mut self, p: PlayerId, targets: &[Entity]) -> &mut Self {
        self.answer(p, DecisionKind::Targets, Answer::Entities(targets.to_vec()))
    }
    pub fn answer_yes(&mut self, p: PlayerId, yes: bool) -> &mut Self {
        self.answer(p, DecisionKind::YesNo, Answer::Bool(yes))
    }
    pub fn answer_choose(&mut self, p: PlayerId, entities: &[Entity]) -> &mut Self {
        self.answer(
            p,
            DecisionKind::Entities,
            Answer::Entities(entities.to_vec()),
        )
    }
    pub fn clear_answers(&mut self) {
        for q in self.script.lock().unwrap().queues.iter_mut() {
            q.clear();
        }
    }
    /// Decisions asked so far.
    pub fn asked(&self) -> Vec<(PlayerId, Decision)> {
        self.script.lock().unwrap().asked.clone()
    }

    // ------------------------------------------------------------------
    // Actions
    // ------------------------------------------------------------------

    /// Begins casting a spell; configure with `.target(..)` etc., then `.go()`.
    pub fn cast(&mut self, p: PlayerId, card: ObjectId) -> CastBuilder<'_> {
        CastBuilder {
            t: self,
            p,
            card,
            method: CastMethod::Normal,
        }
    }

    /// Casts a spell with the given targets (one per slot) and returns the spell's id.
    pub fn cast_with(
        &mut self,
        p: PlayerId,
        card: ObjectId,
        targets: &[Entity],
    ) -> Result<ObjectId, crate::casting::Illegal> {
        for t in targets {
            self.answer_targets(p, &[*t]);
        }
        self.g.turn.priority = Some(p);
        let r = self.g.cast_spell(p, card, CastMethod::Normal);
        self.g.flush_events();
        r
    }

    /// Activates the `index`th activated ability of `source`.
    pub fn activate(
        &mut self,
        p: PlayerId,
        source: ObjectId,
        index: usize,
        targets: &[Entity],
    ) -> Result<Option<ObjectId>, crate::casting::Illegal> {
        self.g.recompute();
        let uid = self
            .g
            .obj(source)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
            .nth(index)
            .map(|a| a.uid)
            .expect("no such activated ability");
        for t in targets {
            self.answer_targets(p, &[*t]);
        }
        self.g.turn.priority = Some(p);
        let r = self.g.activate_ability(p, source, uid);
        self.g.flush_events();
        r
    }

    /// Plays a land from hand.
    pub fn play_land(
        &mut self,
        p: PlayerId,
        card: ObjectId,
    ) -> Result<(), crate::casting::Illegal> {
        self.g.turn.priority = Some(p);
        self.g.play_land(p, card)
    }

    /// Checks SBAs and puts triggers on the stack (as happens before priority).
    pub fn settle(&mut self) {
        self.g.settle();
    }

    /// Resolves the top object of the stack (after settling SBAs and triggers).
    pub fn resolve(&mut self) {
        self.g.settle();
        self.g.resolve_top();
        self.g.settle();
    }

    /// Resolves everything on the stack (including triggers that get added).
    pub fn resolve_all(&mut self) {
        self.g.settle();
        let mut guard = 0;
        while !self.g.stack.is_empty() && guard < 200 {
            self.g.resolve_top();
            self.g.settle();
            guard += 1;
        }
    }

    /// Advances the game (both players passing by default) until the given step of the
    /// given player's turn begins and that player has priority.
    pub fn advance_to(&mut self, active: PlayerId, step: Step) {
        let ok = self.g.run_until(10_000, |g| {
            g.turn.active == active
                && g.turn.step == step
                && g.turn.stage == Stage::Priority
                && g.turn.priority == Some(active)
        });
        assert!(ok, "did not reach {step:?} of {active}'s turn");
    }

    /// Advances to the next occurrence of `step` (in any turn).
    pub fn advance_to_step(&mut self, step: Step) {
        let ok = self.g.run_until(10_000, |g| {
            g.turn.step == step && g.turn.stage == Stage::Priority
        });
        assert!(ok, "did not reach {step:?}");
    }

    /// Runs combat from the current point: queues the attack declaration (and optional
    /// blocks by the defending player), then advances to the end of combat step.
    pub fn attack(&mut self, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
        let ap = self.g.turn.active;
        self.answer(
            ap,
            DecisionKind::Attackers,
            Answer::Attackers(attackers.to_vec()),
        );
        if !blocks.is_empty() {
            let dp = self.g.obj(blocks[0].0).controller;
            self.answer(
                dp,
                DecisionKind::Blockers,
                Answer::Blockers(blocks.to_vec()),
            );
        }
        if !(self.g.turn.step.is_combat()) {
            self.advance_to(ap, Step::BeginningOfCombat);
        }
        self.advance_to(ap, Step::EndOfCombat);
    }

    // ------------------------------------------------------------------
    // Queries
    // ------------------------------------------------------------------

    pub fn life(&self, p: PlayerId) -> i32 {
        self.g.player(p).life
    }
    pub fn obj_now(&self, id: ObjectId) -> &GameObject {
        self.g.obj(self.g.current(id))
    }
    pub fn zone(&self, id: ObjectId) -> Zone {
        self.g.obj(self.g.current(id)).zone
    }
    pub fn on_battlefield(&self, id: ObjectId) -> bool {
        self.zone(id) == Zone::Battlefield
    }
    pub fn pt(&self, id: ObjectId) -> (i32, i32) {
        let o = self.obj_now(id);
        (o.power(), o.toughness())
    }
    pub fn named_on_battlefield(&self, name: &str) -> Vec<ObjectId> {
        self.g.find_in_zone(Zone::Battlefield, name)
    }
    pub fn in_graveyard(&self, p: PlayerId, name: &str) -> bool {
        !self.g.find_in_zone(Zone::Graveyard(p), name).is_empty()
    }
    pub fn in_hand(&self, p: PlayerId, name: &str) -> bool {
        !self.g.find_in_zone(Zone::Hand(p), name).is_empty()
    }
    pub fn in_exile(&self, name: &str) -> bool {
        !self.g.find_in_zone(Zone::Exile, name).is_empty()
    }
    pub fn hand_size(&self, p: PlayerId) -> usize {
        self.g.player(p).hand.len()
    }
    pub fn library_size(&self, p: PlayerId) -> usize {
        self.g.player(p).library.len()
    }
    pub fn graveyard_size(&self, p: PlayerId) -> usize {
        self.g.player(p).graveyard.len()
    }
    pub fn counters(&self, id: ObjectId, kind: &str) -> u32 {
        self.obj_now(id).counter(kind)
    }
    pub fn stack_len(&self) -> usize {
        self.g.stack.len()
    }
    pub fn has_lost(&self, p: PlayerId) -> bool {
        self.g.player(p).has_lost
    }
    pub fn dump_log(&self) -> String {
        self.g
            .log
            .iter()
            .map(|l| format!("[T{}] {}", l.turn, l.text))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Builder for casting a spell in tests.
pub struct CastBuilder<'a> {
    t: &'a mut TestGame,
    p: PlayerId,
    card: ObjectId,
    method: CastMethod,
}

impl<'a> CastBuilder<'a> {
    pub fn target(self, e: impl Into<Entity>) -> Self {
        let p = self.p;
        self.t.answer_targets(p, &[e.into()]);
        self
    }
    pub fn targets(self, es: &[Entity]) -> Self {
        let p = self.p;
        self.t.answer_targets(p, es);
        self
    }
    pub fn x(self, x: i64) -> Self {
        let p = self.p;
        self.t.answer(p, DecisionKind::X, Answer::Number(x));
        self
    }
    pub fn modes(self, modes: &[usize]) -> Self {
        let p = self.p;
        self.t
            .answer(p, DecisionKind::Modes, Answer::Indices(modes.to_vec()));
        self
    }
    pub fn kicked(self, yes: bool) -> Self {
        let p = self.p;
        self.t
            .answer(p, DecisionKind::OptionalCost, Answer::Bool(yes));
        self
    }
    pub fn method(mut self, m: CastMethod) -> Self {
        self.method = m;
        self
    }
    /// Casts the spell. Panics if casting is illegal.
    pub fn go(self) -> ObjectId {
        self.try_go().expect("casting failed")
    }
    pub fn try_go(self) -> Result<ObjectId, crate::casting::Illegal> {
        self.t.g.turn.priority = Some(self.p);
        let r = self.t.g.cast_spell(self.p, self.card, self.method);
        self.t.g.flush_events();
        r
    }
}
