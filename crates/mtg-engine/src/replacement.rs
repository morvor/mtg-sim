//! Replacement and prevention effects (CR 614–616).
//!
//! Events that can be replaced are first proposed as a [`ReplEvent`]. The pipeline
//! repeatedly finds applicable replacement effects, lets the affected player choose one
//! (respecting the ordering of CR 616.1a–e), applies it, and repeats on the modified
//! event(s) until none apply. Each effect gets only one opportunity per event (CR 614.5).

use crate::ability::*;
use crate::card::CardDef;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::types::*;
use std::sync::Arc;

/// Modifications to how a permanent enters the battlefield (CR 614.1c–d, 614.12).
#[derive(Clone, Debug, Default)]
pub struct EtbInfo {
    pub tapped: bool,
    pub counters: Vec<(CounterKind, u32)>,
    pub controller: Option<PlayerId>,
    /// Enters as a copy of this object's copiable values (CR 707.9).
    pub copy_of: Option<ObjectId>,
    pub copy_exceptions: Vec<Modification>,
    /// Modifications to its copiable values from "as this enters" abilities that set
    /// power and toughness (CR 707.2), applied after any copy effect it enters with.
    pub copiable_mods: Vec<Modification>,
    pub face_down: Option<KeywordKind>,
    pub transformed: bool,
    pub attacking: Option<Entity>,
    pub blocking: Option<ObjectId>,
    pub attach_to: Option<Entity>,
    /// Effects to perform right after it is put onto the battlefield, before its
    /// zone-change event is emitted: (source ability ctx, effect). ("As this enters"
    /// replacement effects run earlier, while the replacement applies; see
    /// [`ReplacementAction::AsEnters`].)
    pub as_enters: Vec<(Ctx, Effect)>,
    /// Cast info carried from the stack.
    pub cast: Option<CastInfo>,
    /// Face to put onto the battlefield (for MDFCs played/cast as back face).
    pub face: Option<FaceState>,
    /// For a move to exile caused by a replacement effect: the link id of the ability
    /// whose replacement effect exiles the object, so the object is "exiled with" that
    /// ability's source (CR 607.2b, 614.14).
    pub link: Option<u16>,
    /// A continuous effect of the resolving spell or ability that puts the permanent onto
    /// the battlefield and says it "is" something: (source, controller, modifications).
    /// It applies as the permanent enters (CR 611.2e), with a later timestamp than the
    /// permanent's own static abilities (CR 613.7n).
    pub with_mods: Option<(Option<ObjectId>, PlayerId, Vec<Modification>)>,
}

#[derive(Clone, Debug)]
pub struct MoveEv {
    pub obj: ObjectId,
    pub to: Zone,
    pub pos: LibraryPosition,
    pub cause: MoveCause,
    pub by: Option<PlayerId>,
    pub etb: EtbInfo,
    /// The source that caused the move (for "if a spell or ability an opponent controls
    /// causes you to discard" style checks and linked abilities).
    pub source: Option<ObjectId>,
}

/// A token about to be created.
#[derive(Clone, Debug)]
pub struct TokenCreate {
    pub chars: Characteristics,
    pub card: Option<Arc<CardDef>>,
    pub tapped: bool,
    pub attacking: Option<Entity>,
    /// A token created as a copy of this object (CR 707.2, 111.10).
    pub copy_of: Option<ObjectId>,
    pub copy_exceptions: Vec<Modification>,
}

/// A proposed event that replacement effects may modify.
#[derive(Clone, Debug)]
pub enum ReplEvent {
    Move(MoveEv),
    Damage {
        source: ObjectId,
        target: Entity,
        amount: u32,
        combat: bool,
    },
    Draw {
        player: PlayerId,
    },
    GainLife {
        player: PlayerId,
        amount: u32,
    },
    LoseLife {
        player: PlayerId,
        amount: u32,
    },
    AddCounters {
        target: Entity,
        kind: CounterKind,
        n: u32,
        source: Option<ObjectId>,
    },
    CreateTokens {
        controller: PlayerId,
        spec: Box<TokenCreate>,
        count: u32,
        source: Option<ObjectId>,
    },
    Destroy {
        obj: ObjectId,
        source: Option<ObjectId>,
    },
    LoseGame {
        player: PlayerId,
    },
}

/// Identifies a replacement effect for the "only once per event" rule (CR 614.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReplKey {
    Static(ObjectId, u64),
    Instance(u32),
}

#[derive(Clone, Debug)]
struct Candidate {
    key: ReplKey,
    source: Option<ObjectId>,
    /// The link of the ability generating the effect (CR 607, 614.14).
    link: u16,
    controller: PlayerId,
    def: ReplacementDef,
    class: u8,
    text: String,
    instance: Option<u32>,
}

/// Which replacement effects to look for (see [`Game::replacement_candidates`]).
#[derive(Clone, Copy, PartialEq, Eq)]
enum CandScope {
    All,
    /// Effects that modify how a permanent enters the battlefield.
    EntryOnly,
    /// All others.
    NotEntry,
}

impl Game {
    /// Runs the replacement pipeline on a proposed event and returns the event(s) that
    /// actually happen.
    pub fn replace(&mut self, ev: ReplEvent) -> Vec<ReplEvent> {
        let applied: Vec<ReplKey> = self.repl_context.last().cloned().unwrap_or_default();
        self.replace_rec(ev, applied, 0, false)
    }

    /// Runs only self-replacement effects on an event that can't happen (CR 614.17c).
    pub fn replace_self_only(&mut self, ev: ReplEvent) -> Vec<ReplEvent> {
        let applied: Vec<ReplKey> = self.repl_context.last().cloned().unwrap_or_default();
        self.replace_rec(ev, applied, 0, true)
    }

    fn replace_rec(
        &mut self,
        ev: ReplEvent,
        applied: Vec<ReplKey>,
        depth: u32,
        self_only: bool,
    ) -> Vec<ReplEvent> {
        if depth > 32 {
            return vec![ev];
        }
        if self.dirty {
            self.recompute();
        }
        let mut cands = match &ev {
            // CR 614.12: which effects modify how a permanent enters, and how, is
            // determined from the permanent as it would exist on the battlefield.
            ReplEvent::Move(m) if m.to == Zone::Battlefield => {
                let mut v = self.replacement_candidates(&ev, &applied, CandScope::NotEntry);
                let m = m.clone();
                let ev2 = ev.clone();
                v.extend(self.with_hypothetical_entry(&m, |g| {
                    g.replacement_candidates(&ev2, &applied, CandScope::EntryOnly)
                }));
                v
            }
            _ => self.replacement_candidates(&ev, &applied, CandScope::All),
        };
        if self_only {
            cands.retain(|c| c.def.self_replacement);
        }
        if cands.is_empty() {
            return vec![ev];
        }
        // CR 616.1a–e: self-replacement, then control, then copy, then back-face, then any.
        let min_class = cands.iter().map(|c| c.class).min().unwrap();
        cands.retain(|c| c.class == min_class);
        let chooser = self.affected_player(&ev);
        let pick = if cands.len() == 1 {
            0
        } else {
            let options = cands.iter().map(|c| c.text.clone()).collect();
            match self.ask(chooser, Decision::ChooseReplacement { options }) {
                Answer::Index(i) if i < cands.len() => i,
                _ => 0,
            }
        };
        let cand = cands.swap_remove(pick);
        let mut applied = applied;
        applied.push(cand.key);
        // Optional ("you may") replacements.
        if cand.def.optional {
            let who = cand.controller;
            if !self.ask_yes_no(who, cand.source, &format!("Apply: {}?", cand.text), true) {
                return self.replace_rec(ev, applied, depth + 1, self_only);
            }
        }
        let results = self.apply_replacement(&cand, ev, &applied);
        let mut out = Vec::new();
        for r in results {
            out.extend(self.replace_rec(r, applied.clone(), depth + 1, self_only));
        }
        out
    }

    /// The player who chooses among replacement effects for an event (CR 616.1).
    fn affected_player(&self, ev: &ReplEvent) -> PlayerId {
        match ev {
            ReplEvent::Move(m) => {
                let o = self.obj(m.obj);
                match o.zone {
                    Zone::Battlefield | Zone::Stack => o.controller,
                    _ => o.owner,
                }
            }
            ReplEvent::Damage { target, .. } => match target {
                Entity::Player(p) => *p,
                Entity::Object(o) => self.obj(*o).controller,
            },
            ReplEvent::Draw { player }
            | ReplEvent::GainLife { player, .. }
            | ReplEvent::LoseLife { player, .. }
            | ReplEvent::LoseGame { player } => *player,
            ReplEvent::AddCounters { target, .. } => match target {
                Entity::Player(p) => *p,
                Entity::Object(o) => self.obj(*o).controller,
            },
            ReplEvent::CreateTokens { controller, .. } => *controller,
            ReplEvent::Destroy { obj, .. } => self.obj(*obj).controller,
        }
    }

    fn replacement_candidates(
        &self,
        ev: &ReplEvent,
        applied: &[ReplKey],
        scope: CandScope,
    ) -> Vec<Candidate> {
        let mut out = Vec::new();
        let in_scope = |d: &ReplacementDef| match scope {
            CandScope::All => true,
            CandScope::EntryOnly => matches!(d.event, ReplacementEvent::EntersBattlefield(_)),
            CandScope::NotEntry => !matches!(d.event, ReplacementEvent::EntersBattlefield(_)),
        };
        // Static abilities of objects (CR 614.12: include the entering object's own
        // abilities for ETB replacements).
        let mut sources: Vec<(ObjectId, PlayerId, Ability, ReplacementDef)> = self
            .statics
            .replacements
            .iter()
            .map(|(s, c, _, a, d)| (*s, *c, a.clone(), d.clone()))
            .collect();
        if let ReplEvent::Move(m) = ev {
            if m.to == Zone::Battlefield && scope != CandScope::NotEntry {
                let o = self.obj(m.obj);
                // CR 614.12, 707.9: once it's entering as a copy, the copied object's
                // "as this enters" / "enters with" abilities apply instead of its own.
                // A card entering with another face up (a modal DFC's back face played
                // as a land, or entering transformed) has that face's abilities.
                let face = if m.etb.transformed {
                    Some(FaceState::Back)
                } else {
                    m.etb.face
                };
                let face_chars = match (face, &o.card) {
                    (Some(f), Some(card)) if f != o.face => Some(card.characteristics(f)),
                    _ => None,
                };
                let abilities = match (m.etb.copy_of, &face_chars) {
                    (Some(c), _) => &self.obj(c).copiable.abilities,
                    (None, Some(fc)) => &fc.abilities,
                    (None, None) => &o.chars.abilities,
                };
                for a in abilities {
                    if let AbilityKind::Static(s) = &a.kind {
                        // CR 614.12: only effects that affect just that permanent apply
                        // from the permanent itself ("Permanents enter tapped" doesn't
                        // affect the permanent that has it).
                        if let StaticEffect::Replacement(d) = &s.effect {
                            if matches!(
                                d.event,
                                ReplacementEvent::EntersBattlefield(Filter::Source)
                            ) && !sources
                                .iter()
                                .any(|(src, _, ab, _)| *src == m.obj && ab.uid == a.uid)
                            {
                                sources.push((
                                    m.obj,
                                    m.by.unwrap_or(o.controller),
                                    a.clone(),
                                    d.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }
        for (src, ctl, a, d) in sources {
            let key = ReplKey::Static(src, a.uid);
            if applied.contains(&key) || !in_scope(&d) {
                continue;
            }
            let mut ctx = Ctx::new(Some(src), ctl);
            ctx.link = a.link;
            if self.repl_event_matches(&d.event, &ctx, ev, None) {
                out.push(Candidate {
                    key,
                    source: Some(src),
                    link: a.link,
                    controller: ctl,
                    class: repl_class(&d, ev),
                    text: format!("{}: {}", self.obj(src).chars.name, a.text),
                    def: d,
                    instance: None,
                });
            }
        }
        for inst in &self.replacements {
            let key = ReplKey::Instance(inst.id);
            if applied.contains(&key) {
                continue;
            }
            if inst.uses == Some(0) || !in_scope(&inst.def) {
                continue;
            }
            let ctx = Ctx::new(inst.source, inst.controller);
            if self.repl_event_matches(&inst.def.event, &ctx, ev, inst.objects.as_deref()) {
                out.push(Candidate {
                    key,
                    source: inst.source,
                    link: 0,
                    controller: inst.controller,
                    class: repl_class(&inst.def, ev),
                    text: format!("effect #{}", inst.id),
                    def: inst.def.clone(),
                    instance: Some(inst.id),
                });
            }
        }
        // Built-in rules replacement: commander to hand/library (CR 903.9b).
        if let ReplEvent::Move(m) = ev {
            let o = self.obj(m.obj);
            let key = ReplKey::Static(m.obj, 0);
            if o.is_commander
                && self.config.variant == Variant::Commander
                && matches!(m.to, Zone::Hand(_) | Zone::Library(_))
                && !applied.contains(&key)
                && scope != CandScope::EntryOnly
            {
                out.push(Candidate {
                    key,
                    source: None,
                    link: 0,
                    controller: o.owner,
                    class: 4,
                    text: "Commander: put into command zone instead".into(),
                    def: ReplacementDef {
                        event: ReplacementEvent::ZoneChange {
                            filter: Filter::Any,
                            from: None,
                            to: None,
                        },
                        action: ReplacementAction::MoveInstead(Destination::zone(
                            ZoneKind::Command,
                        )),
                        self_replacement: false,
                        optional: true,
                    },
                    instance: None,
                });
            }
        }
        out
    }

    /// Whether the resolved replacement/prevention effect with this id would apply to
    /// the event.
    pub(crate) fn instance_matches(&self, id: u32, ev: &ReplEvent) -> bool {
        self.replacements.iter().any(|inst| {
            inst.id == id
                && inst.uses != Some(0)
                && self.repl_event_matches(
                    &inst.def.event,
                    &Ctx::new(inst.source, inst.controller),
                    ev,
                    inst.objects.as_deref(),
                )
        })
    }

    /// Evaluates `f` with the object of a move onto the battlefield temporarily put there
    /// as it would exist (CR 614.12, 614.17d): under the player who'll control it, with
    /// the face, copy effect, counters, and tapped status from replacement effects already
    /// applied, its own static abilities, and continuous effects that already exist. The
    /// replacement and restriction effects considered are those that exist now, not ones
    /// the entering permanent's abilities would create.
    pub(crate) fn with_hypothetical_entry<R>(
        &mut self,
        m: &MoveEv,
        f: impl FnOnce(&Game) -> R,
    ) -> R {
        if self.dirty {
            self.recompute();
        }
        let id = m.obj;
        let saved_obj = self.objects[id.0 as usize].clone();
        let saved_bf = self.battlefield.clone();
        let saved_effects = self.effects.len();
        let saved_statics = self.statics.clone();
        let saved_next_ts = self.next_timestamp;
        let saved_events = self.events.len();
        let from = self.obj(id).zone;
        let saved_list = self.zone_list_mut(from).map(|l| l.clone());
        let controller = self.entry_controller(m);
        if let Some(l) = self.zone_list_mut(from) {
            l.retain(|x| *x != id);
        }
        {
            let o = &mut self.objects[id.0 as usize];
            o.zone = Zone::Battlefield;
            o.controller = controller;
            o.base_controller = controller;
            o.tapped = m.etb.tapped;
            o.phased_out = false;
            o.timestamp = saved_next_ts;
            o.face = if m.etb.transformed {
                FaceState::Back
            } else if let Some(fc) = m.etb.face {
                fc
            } else {
                match o.face {
                    FaceState::Half(_) | FaceState::Fused | FaceState::Flipped => FaceState::Front,
                    f => f,
                }
            };
            if let Some(card) = o.card.clone() {
                o.base = card.characteristics(o.face);
            }
            if m.etb.face_down.is_some() {
                o.face_down = true;
            }
            for (k, n) in &m.etb.counters {
                *o.counters.entry(k.clone()).or_insert(0) += n;
            }
        }
        self.battlefield.push(id);
        if let Some(src) = m.etb.copy_of {
            let values = Box::new(self.obj(src).copiable.clone());
            self.effects.push(ContinuousEffect {
                id: 0,
                source: Some(id),
                controller,
                timestamp: saved_next_ts,
                duration: Duration::Permanent,
                affected: Affected::Objects(vec![id]),
                mods: vec![],
                layer1: Some(Layer1::Copy {
                    values,
                    exceptions: m.etb.copy_exceptions.clone(),
                }),
                created_turn: self.turn.number,
            });
        }
        self.compute_characteristics(false);
        self.statics.replacements = saved_statics.replacements.clone();
        self.statics.restrictions = saved_statics.restrictions.clone();
        let r = f(self);
        self.objects[id.0 as usize] = saved_obj;
        self.battlefield = saved_bf;
        if let (Some(l), Some(saved)) = (self.zone_list_mut(from), saved_list) {
            *l = saved;
        }
        self.effects.truncate(saved_effects);
        self.next_timestamp = saved_next_ts;
        self.events.truncate(saved_events);
        self.compute_characteristics(false);
        r
    }

    /// Whether a "can't enter the battlefield" effect stops this move (CR 614.17d),
    /// checking the object as it would exist on the battlefield.
    pub(crate) fn cant_enter(&mut self, m: &MoveEv) -> bool {
        let any = self.statics.restrictions.iter().any(|(_, _, r)| {
            matches!(
                r,
                Restriction::CantEnter(_) | Restriction::CantEnterBattlefield(_)
            )
        }) || self.rule_effects.iter().any(|e| {
            matches!(
                e.restriction,
                Restriction::CantEnter(_) | Restriction::CantEnterBattlefield(_)
            )
        });
        if !any {
            return false;
        }
        self.with_hypothetical_entry(m, |g| {
            let id = m.obj;
            g.statics.restrictions.iter().any(|(s, c, r)| match r {
                Restriction::CantEnter(f) | Restriction::CantEnterBattlefield(f) => {
                    g.matches(id, f, &Ctx::new(Some(*s), *c))
                }
                _ => false,
            }) || g.rule_effects.iter().any(|e| match &e.restriction {
                Restriction::CantEnter(f) | Restriction::CantEnterBattlefield(f) => {
                    e.objects.as_ref().is_none_or(|v| v.contains(&id))
                        && g.matches(id, f, &Ctx::new(e.source, e.controller))
                }
                _ => false,
            })
        })
    }

    /// Whether a replacement effect's event pattern matches the proposed event.
    fn repl_event_matches(
        &self,
        pat: &ReplacementEvent,
        ctx: &Ctx,
        ev: &ReplEvent,
        locked: Option<&[ObjectId]>,
    ) -> bool {
        let locked_ok = |o: ObjectId| locked.is_none_or(|v| v.contains(&o));
        match (pat, ev) {
            (ReplacementEvent::EntersBattlefield(f), ReplEvent::Move(m)) => {
                m.to == Zone::Battlefield && locked_ok(m.obj) && self.matches_entering(m, f, ctx)
            }
            (ReplacementEvent::ZoneChange { filter, from, to }, ReplEvent::Move(m)) => {
                let o = self.obj(m.obj);
                from.is_none_or(|z| o.zone.kind() == Some(z))
                    && to.is_none_or(|z| m.to.kind() == Some(z))
                    && locked_ok(m.obj)
                    && self.matches(m.obj, filter, ctx)
            }
            (ReplacementEvent::Dies(f), ReplEvent::Move(m)) => {
                let o = self.obj(m.obj);
                o.zone == Zone::Battlefield
                    && matches!(m.to, Zone::Graveyard(_))
                    && o.is_creature()
                    && locked_ok(m.obj)
                    && self.matches(m.obj, f, ctx)
            }
            (ReplacementEvent::Draw(pf), ReplEvent::Draw { player }) => {
                self.player_filter_matches(pf, *player, ctx)
            }
            (
                ReplacementEvent::Damage {
                    source,
                    to_players,
                    to_objects,
                    combat_only,
                },
                ReplEvent::Damage {
                    source: s,
                    target,
                    combat,
                    amount,
                },
            ) => {
                if *amount == 0 || (*combat_only && !combat) {
                    return false;
                }
                if !self.matches(*s, source, ctx) {
                    return false;
                }
                match target {
                    Entity::Player(p) => {
                        to_players
                            .as_ref()
                            .is_some_and(|f| self.player_filter_matches(f, *p, ctx))
                            && locked.is_none_or(|_| true)
                    }
                    Entity::Object(o) => {
                        to_objects
                            .as_ref()
                            .is_some_and(|f| self.matches(*o, f, ctx))
                            && locked_ok(*o)
                    }
                }
            }
            (ReplacementEvent::GainLife(pf), ReplEvent::GainLife { player, amount }) => {
                *amount > 0 && self.player_filter_matches(pf, *player, ctx)
            }
            (ReplacementEvent::LoseLife(pf), ReplEvent::LoseLife { player, amount }) => {
                *amount > 0 && self.player_filter_matches(pf, *player, ctx)
            }
            (
                ReplacementEvent::PutCounters {
                    on_objects,
                    on_players,
                    kind,
                },
                ReplEvent::AddCounters {
                    target, kind: k, n, ..
                },
            ) => {
                if *n == 0 || kind.as_ref().is_some_and(|x| x != k) {
                    return false;
                }
                match target {
                    Entity::Object(o) => on_objects
                        .as_ref()
                        .is_some_and(|f| self.matches(*o, f, ctx)),
                    Entity::Player(p) => on_players
                        .as_ref()
                        .is_some_and(|f| self.player_filter_matches(f, *p, ctx)),
                }
            }
            (
                ReplacementEvent::CreateTokens(pf),
                ReplEvent::CreateTokens {
                    controller, count, ..
                },
            ) => *count > 0 && self.player_filter_matches(pf, *controller, ctx),
            (ReplacementEvent::Destroy(f), ReplEvent::Destroy { obj, .. }) => {
                locked_ok(*obj) && self.matches(*obj, f, ctx)
            }
            (ReplacementEvent::LoseGame(pf), ReplEvent::LoseGame { player }) => {
                self.player_filter_matches(pf, *player, ctx)
            }
            _ => false,
        }
    }

    /// Checks an entering object against a filter "as it would exist on the battlefield"
    /// (CR 614.12). We approximate with its current characteristics plus the
    /// modifications already made to how it enters, and the player who will control it.
    fn matches_entering(&self, m: &MoveEv, f: &Filter, ctx: &Ctx) -> bool {
        let view = EnteringView {
            obj: m.obj,
            controller: self.entering_controller(m),
        };
        self.matches_view(&view, m.obj, f, ctx)
    }

    /// The player who will control a permanent entering the battlefield (as in
    /// `perform_move`).
    pub fn entering_controller(&self, m: &MoveEv) -> PlayerId {
        let o = self.obj(m.obj);
        m.etb
            .controller
            .or(if o.zone == Zone::Stack {
                Some(o.controller)
            } else {
                None
            })
            .or(m.by)
            .unwrap_or(o.owner)
    }

    fn apply_replacement(
        &mut self,
        cand: &Candidate,
        ev: ReplEvent,
        applied: &[ReplKey],
    ) -> Vec<ReplEvent> {
        let mut ctx = Ctx::new(cand.source, cand.controller);
        ctx.link = cand.link;
        // CR 615.12: prevention effects applied to damage that can't be prevented prevent
        // nothing (their other effects still happen), and a shield that prevents nothing
        // isn't used up (CR 609.7b).
        let unpreventable = matches!(ev, ReplEvent::Damage { .. })
            && crate::prevention::is_prevention(&cand.def.action)
            && crate::prevention::damage_cant_be_prevented(self);
        // Use up one application of limited-use effects.
        if let Some(id) = cand.instance {
            if !unpreventable {
                if let Some(inst) = self.replacements.iter_mut().find(|r| r.id == id) {
                    if let Some(u) = inst.uses.as_mut() {
                        *u = u.saturating_sub(1);
                    }
                }
            }
        }
        let key = match cand.key {
            ReplKey::Static(o, u) => (o.0 as u64) << 32 ^ u,
            ReplKey::Instance(i) => (1u64 << 63) | i as u64,
        };
        let action = cand.def.action.clone();
        match (action, ev) {
            (
                ReplacementAction::Prevent
                | ReplacementAction::PreventAmount(_)
                | ReplacementAction::PreventAndThen(..),
                ReplEvent::Damage {
                    source,
                    target,
                    amount,
                    combat,
                },
            ) => {
                // Prevention (CR 615): all of the damage, or a shield of N.
                let (limit, then) = match &cand.def.action {
                    ReplacementAction::Prevent => (None, None),
                    ReplacementAction::PreventAmount(v) => (Some(v.clone()), None),
                    ReplacementAction::PreventAndThen(v, e) => (v.clone(), Some((**e).clone())),
                    _ => (None, None),
                };
                let inst = cand
                    .instance
                    .and_then(|id| self.replacements.iter().position(|r| r.id == id));
                let shield = match &limit {
                    None => None,
                    Some(v) => Some(match inst.and_then(|i| self.replacements[i].remaining) {
                        Some(r) => r,
                        None => self.eval_value(v, &ctx).max(0) as u32,
                    }),
                };
                let prevented = if unpreventable {
                    0
                } else {
                    shield.map_or(amount, |s| s.min(amount))
                };
                if let (Some(i), Some(s)) = (inst, shield) {
                    if !unpreventable {
                        let rem = s - prevented;
                        self.replacements[i].remaining = Some(rem);
                        if rem == 0 {
                            self.replacements.remove(i);
                        }
                    }
                }
                crate::prevention::damage_prevented(
                    self,
                    key,
                    cand.source,
                    source,
                    target,
                    prevented,
                );
                if let Some(e) = then {
                    // CR 615.5: the rest of the effect happens immediately afterward.
                    let mut c = ctx.clone();
                    let mut info = event_info_of(&ReplEvent::Damage {
                        source,
                        target,
                        amount,
                        combat,
                    });
                    info.amount = prevented as i32;
                    c.event = Some(info);
                    self.post_replacement_effects.push((c, e));
                }
                let left = amount - prevented;
                if left == 0 {
                    vec![]
                } else {
                    vec![ReplEvent::Damage {
                        source,
                        target,
                        amount: left,
                        combat,
                    }]
                }
            }
            (ReplacementAction::EnterTransformed, ReplEvent::Move(mut m)) => {
                m.etb.transformed = true;
                vec![ReplEvent::Move(m)]
            }
            (ReplacementAction::EnterTapped, ReplEvent::Move(mut m)) => {
                m.etb.tapped = true;
                vec![ReplEvent::Move(m)]
            }
            (ReplacementAction::EnterWithCounters(k, v), ReplEvent::Move(mut m)) => {
                let mut c = ctx.clone();
                // The amount is computed for the replacement effect's source: the entering
                // permanent itself, or e.g. "each other creature you control enters with X
                // additional counters, where X is the number of counters on ~".
                c.source = cand.source.or(Some(m.obj));
                c.cast = m.etb.cast.clone();
                c.x = m.etb.cast.as_ref().and_then(|ci| ci.x).unwrap_or(0);
                let n = self.eval_value(&v, &c).max(0) as u32;
                if n > 0 {
                    m.etb.counters.push((k, n));
                }
                vec![ReplEvent::Move(m)]
            }
            (ReplacementAction::AsEnters(e), ReplEvent::Move(mut m)) => {
                // CR 614.12a: choices required by a replacement effect that modifies how a
                // permanent enters are made before it enters. The effect runs now, with
                // the entering object as its source; choices are stored on that object
                // and carried onto the permanent (see `perform_move`), and entry
                // modifications ("it enters tapped") are applied to this event.
                let mut c = ctx.clone();
                c.source = Some(m.obj);
                c.controller = m.etb.controller.unwrap_or(cand.controller);
                c.cast = m.etb.cast.clone();
                c.x = m.etb.cast.as_ref().and_then(|ci| ci.x).unwrap_or(0);
                c.entering = Some(crate::eval::EntryMods::default());
                self.exec(&e, &mut c);
                if let Some(em) = c.entering.take() {
                    m.etb.tapped |= em.tapped;
                    m.etb.counters.extend(em.counters);
                    m.etb.copy_exceptions.extend(em.copy_exceptions);
                    m.etb.copiable_mods.extend(em.copiable);
                    // CR 614.1c: "it enters with haste" — performed on the permanent as
                    // it's put onto the battlefield.
                    for e in em.on_entry {
                        m.etb.as_enters.push((c.clone(), e));
                    }
                    if em.prepared {
                        // CR 722.3a/c: it gains the designation (and its prepare-spell
                        // copy is created) as it's put onto the battlefield.
                        m.etb.as_enters.push((
                            c.clone(),
                            Effect::SetPrepared {
                                what: Sel::This,
                                prepared: true,
                            },
                        ));
                    }
                }
                vec![ReplEvent::Move(m)]
            }
            (ReplacementAction::EnterAsCopy { filter, optional }, ReplEvent::Move(mut m)) => {
                // "enter as a copy of [this creature]" refers to the effect's source.
                let c = ctx.clone();
                let cands: Vec<ObjectId> = self
                    .objects_matching(&filter, &c)
                    .into_iter()
                    .filter(|o| *o != m.obj)
                    .collect();
                let chooser = m.by.unwrap_or(cand.controller);
                let min = if optional { 0 } else { 1 };
                let chosen = self.ask_objects(
                    chooser,
                    Some(m.obj),
                    "Choose an object to copy",
                    cands,
                    min,
                    1,
                );
                if let Some(o) = chosen.first() {
                    m.etb.copy_of = Some(*o);
                }
                vec![ReplEvent::Move(m)]
            }
            (ReplacementAction::EnterUnderControl(r), ReplEvent::Move(mut m)) => {
                if let Some(p) = self.eval_player(&r, &ctx) {
                    m.etb.controller = Some(p);
                }
                vec![ReplEvent::Move(m)]
            }
            (ReplacementAction::MoveInstead(dest), ReplEvent::Move(mut m)) => {
                let owner = self.obj(m.obj).owner;
                m.to = Zone::of_kind(dest.zone, owner);
                m.pos = dest.position;
                // CR 607.2b, 614.14: a card exiled by a replacement effect is exiled with
                // (linked to) the effect's source.
                if dest.zone == ZoneKind::Exile && cand.source.is_some() {
                    m.source = cand.source;
                    m.etb.link = Some(cand.link);
                }
                vec![ReplEvent::Move(m)]
            }
            (ReplacementAction::MoveInstead(dest), ReplEvent::Destroy { obj, .. }) => {
                let owner = self.obj(obj).owner;
                vec![ReplEvent::Move(MoveEv {
                    obj,
                    to: Zone::of_kind(dest.zone, owner),
                    pos: dest.position,
                    cause: MoveCause::Destroy,
                    by: None,
                    etb: EtbInfo::default(),
                    source: cand.source,
                })]
            }
            (ReplacementAction::Prevent, _) => vec![],
            (ReplacementAction::Regenerate, ReplEvent::Destroy { obj, .. }) => {
                // CR 701.19a: remove all damage, tap it, remove it from combat.
                let o = &mut self.objects[obj.0 as usize];
                o.damage = 0;
                o.deathtouch_damage = false;
                self.tap(obj);
                crate::combat::remove_from_combat(self, obj);
                if let Some(id) = cand.instance {
                    self.replacements.retain(|r| r.id != id);
                }
                vec![]
            }
            (ReplacementAction::Multiply(k), ev) => {
                vec![scale_event(ev, |n| n.saturating_mul(k.max(0) as u32))]
            }
            (ReplacementAction::Add(v), ev) => {
                let d = self.eval_value(&v, &ctx).max(0) as u32;
                vec![scale_event(ev, |n| n + d)]
            }
            (ReplacementAction::Subtract(v), ev) => {
                let d = self.eval_value(&v, &ctx).max(0) as u32;
                let e = scale_event(ev, |n| n.saturating_sub(d));
                if event_amount(&e) == Some(0) {
                    vec![]
                } else {
                    vec![e]
                }
            }
            (
                ReplacementAction::Redirect(sel),
                ReplEvent::Damage {
                    source,
                    target,
                    amount,
                    combat,
                },
            ) => {
                let new_target = self.eval_sel(&sel, &ctx).into_iter().next();
                match new_target {
                    // CR 614.9: redirection to something no longer valid does nothing.
                    Some(t) if self.valid_damage_recipient(t) => {
                        vec![ReplEvent::Damage {
                            source,
                            target: t,
                            amount,
                            combat,
                        }]
                    }
                    _ => vec![ReplEvent::Damage {
                        source,
                        target,
                        amount,
                        combat,
                    }],
                }
            }
            (ReplacementAction::Instead(effect), ev) => {
                let mut c = ctx.clone();
                c.event = Some(event_info_of(&ev));
                self.repl_context.push(applied.to_vec());
                self.exec(&effect, &mut c);
                self.repl_context.pop();
                vec![]
            }
            (ReplacementAction::Also(effect), ev) => {
                let mut c = ctx.clone();
                c.event = Some(event_info_of(&ev));
                self.post_replacement_effects.push((c, *effect));
                vec![ev]
            }
            (_, ev) => vec![ev],
        }
    }

    pub fn valid_damage_recipient(&self, e: Entity) -> bool {
        match e {
            Entity::Player(p) => self.player(p).in_game(),
            Entity::Object(o) => {
                let ob = self.obj(o);
                self.is_live(o)
                    && ob.zone == Zone::Battlefield
                    && (ob.is_creature()
                        || ob.is(CardType::Planeswalker)
                        || ob.is(CardType::Battle))
            }
        }
    }
}

/// Characteristics of an object about to enter the battlefield, with the controller it
/// will have there (CR 614.12).
struct EnteringView {
    obj: ObjectId,
    controller: PlayerId,
}

impl crate::eval::View for EnteringView {
    fn chars<'a>(&'a self, g: &'a Game, id: ObjectId) -> &'a Characteristics {
        &g.obj(id).chars
    }
    fn controller(&self, g: &Game, id: ObjectId) -> PlayerId {
        if id == self.obj {
            self.controller
        } else {
            g.obj(id).controller
        }
    }
    fn controller_override(&self, id: ObjectId) -> Option<PlayerId> {
        (id == self.obj).then_some(self.controller)
    }
}

fn repl_class(d: &ReplacementDef, ev: &ReplEvent) -> u8 {
    if d.self_replacement {
        return 0;
    }
    if let ReplEvent::Move(m) = ev {
        if m.to == Zone::Battlefield {
            match &d.action {
                ReplacementAction::EnterUnderControl(_) => return 1,
                ReplacementAction::EnterAsCopy { .. } => return 2,
                ReplacementAction::EnterTransformed => return 3,
                _ => {}
            }
        }
    }
    4
}

fn scale_event(ev: ReplEvent, f: impl Fn(u32) -> u32) -> ReplEvent {
    match ev {
        ReplEvent::Damage {
            source,
            target,
            amount,
            combat,
        } => ReplEvent::Damage {
            source,
            target,
            amount: f(amount),
            combat,
        },
        ReplEvent::GainLife { player, amount } => ReplEvent::GainLife {
            player,
            amount: f(amount),
        },
        ReplEvent::LoseLife { player, amount } => ReplEvent::LoseLife {
            player,
            amount: f(amount),
        },
        ReplEvent::AddCounters {
            target,
            kind,
            n,
            source,
        } => ReplEvent::AddCounters {
            target,
            kind,
            n: f(n),
            source,
        },
        ReplEvent::CreateTokens {
            controller,
            spec,
            count,
            source,
        } => ReplEvent::CreateTokens {
            controller,
            spec,
            count: f(count),
            source,
        },
        other => other,
    }
}

fn event_amount(ev: &ReplEvent) -> Option<u32> {
    match ev {
        ReplEvent::Damage { amount, .. }
        | ReplEvent::GainLife { amount, .. }
        | ReplEvent::LoseLife { amount, .. } => Some(*amount),
        ReplEvent::AddCounters { n, .. } => Some(*n),
        ReplEvent::CreateTokens { count, .. } => Some(*count),
        _ => None,
    }
}

pub fn event_info_of(ev: &ReplEvent) -> EventInfo {
    let mut e = EventInfo::default();
    match ev {
        ReplEvent::Move(m) => {
            e.object = Some(m.obj);
        }
        ReplEvent::Damage {
            source,
            target,
            amount,
            ..
        } => {
            e.other = Some(*source);
            match target {
                Entity::Player(p) => e.player = Some(*p),
                Entity::Object(o) => e.object = Some(*o),
            }
            e.amount = *amount as i32;
        }
        ReplEvent::Draw { player } | ReplEvent::LoseGame { player } => e.player = Some(*player),
        ReplEvent::GainLife { player, amount } | ReplEvent::LoseLife { player, amount } => {
            e.player = Some(*player);
            e.amount = *amount as i32;
        }
        ReplEvent::AddCounters { target, n, .. } => {
            match target {
                Entity::Player(p) => e.player = Some(*p),
                Entity::Object(o) => e.object = Some(*o),
            }
            e.amount = *n as i32;
        }
        ReplEvent::CreateTokens {
            controller, count, ..
        } => {
            e.player = Some(*controller);
            e.amount = *count as i32;
        }
        ReplEvent::Destroy { obj, .. } => e.object = Some(*obj),
    }
    e
}
