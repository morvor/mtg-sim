//! Primitive game actions. Each goes through the replacement pipeline (where the rules
//! say it can be replaced) and emits events.

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::{Event, LookbackSnapshot, MoveCause};
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::replacement::*;
use crate::types::*;
use std::sync::Arc;

impl Game {
    // ------------------------------------------------------------------
    // Zone changes (CR 400.7)
    // ------------------------------------------------------------------

    /// Moves an object to a zone (replacement effects apply). Returns the new object id
    /// if it ended up in *some* zone.
    pub fn move_object(
        &mut self,
        obj: ObjectId,
        to: Zone,
        cause: MoveCause,
        by: Option<PlayerId>,
    ) -> Option<ObjectId> {
        self.move_objects(vec![MoveEv {
            obj,
            to,
            pos: LibraryPosition::Top,
            cause,
            by,
            etb: EtbInfo::default(),
            source: None,
        }])
        .into_iter()
        .next()
        .flatten()
    }

    pub fn move_object_ev(&mut self, m: MoveEv) -> Option<ObjectId> {
        self.move_objects(vec![m]).into_iter().next().flatten()
    }

    /// Moves several objects simultaneously. Returns, for each proposed move, the new id
    /// of that object after the (possibly replaced) move.
    pub fn move_objects(&mut self, moves: Vec<MoveEv>) -> Vec<Option<ObjectId>> {
        if self.dirty {
            self.recompute();
        }
        // Apply replacement effects to each move individually.
        let mut finals: Vec<(usize, ReplEvent)> = Vec::new();
        for (i, m) in moves.iter().enumerate() {
            if !self.can_move(m.obj) {
                continue;
            }
            for e in self.replace(ReplEvent::Move(m.clone())) {
                // An object that can't enter the battlefield stays where it is.
                if let ReplEvent::Move(mv) = &e {
                    if mv.to == Zone::Battlefield && self.cant_enter_battlefield(mv.obj) {
                        continue;
                    }
                }
                finals.push((i, e));
            }
        }
        // Look back in time for leaves-the-battlefield triggers (CR 603.10a).
        let leaving = finals.iter().any(|(_, e)| match e {
            ReplEvent::Move(m) => {
                self.obj(m.obj).zone == Zone::Battlefield && m.to != Zone::Battlefield
            }
            _ => false,
        });
        let lookback = if leaving {
            Some(Arc::new(self.lookback_snapshot()))
        } else {
            None
        };
        let mut out: Vec<Option<ObjectId>> = vec![None; moves.len()];
        for (i, e) in finals {
            match e {
                ReplEvent::Move(m) => {
                    let r = self.perform_move(m, lookback.clone());
                    if out[i].is_none() {
                        out[i] = r;
                    }
                }
                other => self.execute_repl_event(other),
            }
        }
        self.run_post_replacement_effects();
        self.recompute();
        out
    }

    /// Whether a "can't enter the battlefield" effect applies to an object.
    pub fn cant_enter_battlefield(&self, obj: ObjectId) -> bool {
        self.statics.restrictions.iter().any(|(s, c, r)| match r {
            Restriction::CantEnterBattlefield(f) => self.matches(obj, f, &Ctx::new(Some(*s), *c)),
            _ => false,
        })
    }

    /// Whether an object can be moved to a zone: it's a current object in some zone, or a
    /// newly created object (a token or card) that hasn't been put anywhere yet.
    pub fn can_move(&self, id: ObjectId) -> bool {
        let o = self.obj(id);
        self.is_live(id)
            || (o.zone == Zone::Nowhere
                && o.next.is_none()
                && o.prev.is_none()
                && o.kind != ObjKind::StackAbility)
    }

    /// Snapshot of triggered abilities of permanents on the battlefield right now.
    pub fn lookback_snapshot(&self) -> LookbackSnapshot {
        let mut snap = LookbackSnapshot::default();
        for o in self.permanents() {
            for a in &o.chars.abilities {
                if matches!(a.kind, AbilityKind::Triggered(_))
                    || crate::triggers::keyword_has_trigger(a)
                {
                    snap.sources.push((o.id, o.controller, a.clone()));
                }
            }
        }
        snap
    }

    /// Performs a (final, already-replaced) zone change.
    pub(crate) fn perform_move(
        &mut self,
        m: MoveEv,
        lookback: Option<Arc<LookbackSnapshot>>,
    ) -> Option<ObjectId> {
        let old_id = m.obj;
        if !self.can_move(old_id) {
            return None;
        }
        let from = self.obj(old_id).zone;
        let kind = self.obj(old_id).kind;
        if kind == ObjKind::StackAbility {
            self.stack.retain(|x| *x != old_id);
            self.objects[old_id.0 as usize].zone = Zone::Nowhere;
            return None;
        }
        if let Some(list) = self.zone_list_mut(from) {
            list.retain(|x| *x != old_id);
        }
        if from == Zone::Battlefield {
            crate::combat::remove_from_combat(self, old_id);
            // Unpair soulbond partners.
            if let Some(p) = self.obj(old_id).paired_with {
                self.objects[p.0 as usize].paired_with = None;
            }
        }
        let old_controller = self.obj(old_id).controller;
        let old_was_creature = self.obj(old_id).is_creature();
        let new_id = self.create_incarnation(old_id, m.to);
        {
            let face = m.etb.face;
            let n = &mut self.objects[new_id.0 as usize];
            n.zone = m.to;
            if let Some(f) = face {
                n.face = f;
            }
            if m.etb.transformed {
                n.face = FaceState::Back;
            }
            if let Some(card) = n.card.clone() {
                n.base = card.characteristics(n.face);
            }
        }
        match m.to {
            Zone::Battlefield => {
                let controller = m
                    .etb
                    .controller
                    .or(if from == Zone::Stack {
                        Some(old_controller)
                    } else {
                        None
                    })
                    .or(m.by)
                    .unwrap_or(self.obj(new_id).owner);
                {
                    let n = &mut self.objects[new_id.0 as usize];
                    n.controller = controller;
                    n.base_controller = controller;
                    n.tapped = m.etb.tapped;
                    n.summoning_sick = true;
                    n.cast = m.etb.cast.clone().map(Box::new);
                    if let Some(k) = m.etb.face_down {
                        n.face_down = true;
                        n.choices.text = Some(k.name().into());
                    }
                    if let Some(to) = m.etb.attach_to {
                        n.attached_to = Some(to);
                    }
                }
                if let Some(src) = m.etb.copy_of {
                    let values = Box::new(self.obj(src).copiable.clone());
                    let id = self.new_effect_id();
                    let ts = self.obj(new_id).timestamp;
                    self.effects.push(ContinuousEffect {
                        id,
                        source: Some(new_id),
                        controller: controller,
                        timestamp: ts,
                        duration: Duration::Permanent,
                        affected: Affected::Objects(vec![new_id]),
                        mods: vec![],
                        layer1: Some(Layer1::Copy {
                            values,
                            exceptions: m.etb.copy_exceptions.clone(),
                        }),
                        created_turn: self.turn.number,
                    });
                }
                self.battlefield.push(new_id);
                self.recompute();
                // Counters it enters with (CR 122.6): planeswalker loyalty (CR 306.5b),
                // battle defense (CR 310.4), Saga lore (CR 714.3a), plus effects.
                let mut counters_to_add = m.etb.counters.clone();
                let o = self.obj(new_id);
                if o.is(CardType::Planeswalker) {
                    if let Some(l) = o.chars.loyalty {
                        if l > 0 {
                            counters_to_add.insert(0, (counters::LOYALTY.into(), l as u32));
                        }
                    }
                }
                if o.is(CardType::Battle) {
                    if let Some(d) = o.chars.defense {
                        if d > 0 {
                            counters_to_add.insert(0, (counters::DEFENSE.into(), d as u32));
                        }
                    }
                }
                if o.chars.has_subtype("Saga") && crate::saga::has_chapters(o) {
                    counters_to_add.push((counters::LORE.into(), 1));
                }
                for (k, n) in counters_to_add {
                    // Counters placed as it enters are part of the ETB event; replacement
                    // effects on counters still apply (CR 614.16).
                    for e in self.replace(ReplEvent::AddCounters {
                        target: Entity::Object(new_id),
                        kind: k.clone(),
                        n,
                        source: None,
                    }) {
                        if let ReplEvent::AddCounters {
                            target: Entity::Object(t),
                            kind,
                            n,
                            ..
                        } = e
                        {
                            if t == new_id {
                                *self.objects[t.0 as usize].counters.entry(kind).or_insert(0) += n;
                            }
                        }
                    }
                }
                // "As this enters" effects (CR 614.1c).
                for (mut c, e) in m.etb.as_enters.clone() {
                    c.source = Some(new_id);
                    c.controller = controller;
                    self.exec(&e, &mut c);
                }
                if let Some(target) = m.etb.attacking {
                    crate::combat::put_onto_battlefield_attacking(self, new_id, target);
                }
            }
            Zone::Library(p) => {
                let lib = &mut self.players[p.idx()].library;
                match m.pos {
                    LibraryPosition::Top => lib.push(new_id),
                    LibraryPosition::Bottom => lib.insert(0, new_id),
                    LibraryPosition::FromTop(n) => {
                        let idx = lib.len().saturating_sub(n as usize);
                        lib.insert(idx, new_id);
                    }
                    LibraryPosition::Shuffled => {
                        lib.push(new_id);
                        self.shuffle_library(p);
                    }
                }
            }
            Zone::Exile => {
                if m.etb.face_down.is_some() {
                    self.objects[new_id.0 as usize].face_down = true;
                }
                self.exile.push(new_id);
            }
            Zone::Nowhere => {}
            other => {
                if let Some(list) = self.zone_list_mut(other) {
                    list.push(new_id);
                }
            }
        }
        // Linked exile (CR 607): remember cards exiled by a source.
        if let (Some(src), Zone::Exile) = (m.source, m.to) {
            if self.is_live(src) {
                let link = self.current_link;
                self.objects[src.0 as usize]
                    .linked
                    .entry(link)
                    .or_default()
                    .push(new_id);
            }
        }
        if from == Zone::Battlefield {
            self.history.permanents_left.push(old_id);
            if old_was_creature && matches!(m.to, Zone::Graveyard(_)) {
                self.history.creatures_died.push(old_id);
            }
        }
        if matches!(from, Zone::Graveyard(_)) {
            *self
                .history
                .cards_left_graveyard
                .entry(self.obj(old_id).owner)
                .or_insert(0) += 1;
        }
        self.dirty = true;
        self.log(|g| {
            format!(
                "{} moves {:?} -> {:?} ({:?})",
                g.describe(old_id),
                from,
                m.to,
                m.cause
            )
        });
        self.emit(Event::ZoneChange {
            old: old_id,
            new: new_id,
            from,
            to: m.to,
            cause: m.cause,
            by: m.by,
            lookback,
        });
        Some(new_id)
    }

    /// Executes a final (post-replacement) event of any kind.
    pub fn execute_repl_event(&mut self, e: ReplEvent) {
        match e {
            ReplEvent::Move(m) => {
                self.perform_move(m, None);
            }
            ReplEvent::Damage {
                source,
                target,
                amount,
                combat,
            } => self.perform_damage(source, target, amount, combat),
            ReplEvent::Draw { player } => {
                self.perform_draw(player);
            }
            ReplEvent::GainLife { player, amount } => self.perform_gain_life(player, amount),
            ReplEvent::LoseLife { player, amount } => self.perform_lose_life(player, amount),
            ReplEvent::AddCounters {
                target, kind, n, ..
            } => self.perform_add_counters(target, kind, n),
            ReplEvent::CreateTokens {
                controller,
                spec,
                count,
                ..
            } => {
                for _ in 0..count {
                    self.perform_create_token(controller, &spec);
                }
            }
            ReplEvent::Destroy { obj, .. } => {
                let owner = self.obj(obj).owner;
                if self
                    .perform_move(
                        MoveEv {
                            obj,
                            to: Zone::Graveyard(owner),
                            pos: LibraryPosition::Top,
                            cause: MoveCause::Destroy,
                            by: None,
                            etb: EtbInfo::default(),
                            source: None,
                        },
                        Some(Arc::new(self.lookback_snapshot())),
                    )
                    .is_some()
                {
                    self.emit(Event::Destroyed { obj });
                }
            }
            ReplEvent::LoseGame { player } => self.player_loses(player),
        }
    }

    fn run_post_replacement_effects(&mut self) {
        while let Some((mut c, e)) = self.post_replacement_effects.pop() {
            self.exec(&e, &mut c);
        }
    }

    // ------------------------------------------------------------------
    // Drawing (CR 121)
    // ------------------------------------------------------------------

    /// Draws without replacement effects (used for opening hands, CR 103.4).
    pub fn draw_card_raw(&mut self, p: PlayerId) -> Option<ObjectId> {
        let top = self.players[p.idx()].library.pop()?;
        let new = self.create_incarnation(top, Zone::Hand(p));
        self.objects[new.0 as usize].zone = Zone::Hand(p);
        self.players[p.idx()].hand.push(new);
        self.dirty = true;
        Some(new)
    }

    /// Draws `n` cards one at a time (CR 121.2). Returns the cards drawn.
    pub fn draw_cards(&mut self, p: PlayerId, n: u32) -> Vec<ObjectId> {
        let mut out = Vec::new();
        for _ in 0..n {
            if !self.player(p).in_game() {
                break;
            }
            if self.draw_restricted(p) {
                break;
            }
            let before = self.player(p).hand.clone();
            for e in self.replace(ReplEvent::Draw { player: p }) {
                self.execute_repl_event(e);
            }
            for c in self.player(p).hand.clone() {
                if !before.contains(&c) {
                    out.push(c);
                }
            }
        }
        self.run_post_replacement_effects();
        out
    }

    fn draw_restricted(&self, p: PlayerId) -> bool {
        let drawn = self.history.cards_drawn.get(&p).copied().unwrap_or(0);
        self.statics.restrictions.iter().any(|(s, c, r)| match r {
            Restriction::MaxDrawsPerTurn(pf, n) => {
                self.player_filter_matches(pf, p, &Ctx::new(Some(*s), *c)) && drawn >= *n
            }
            _ => false,
        })
    }

    fn perform_draw(&mut self, p: PlayerId) -> Option<ObjectId> {
        let Some(top) = self.players[p.idx()].library.last().copied() else {
            // CR 121.4 / 704.5b
            self.players[p.idx()].drew_from_empty_library = true;
            return None;
        };
        let new = self.perform_move(
            MoveEv {
                obj: top,
                to: Zone::Hand(p),
                pos: LibraryPosition::Top,
                cause: MoveCause::Draw,
                by: Some(p),
                etb: EtbInfo::default(),
                source: None,
            },
            None,
        )?;
        let nth = {
            let c = self.history.cards_drawn.entry(p).or_insert(0);
            *c += 1;
            *c
        };
        self.emit(Event::Drew {
            player: p,
            card: new,
            nth,
        });
        Some(new)
    }

    // ------------------------------------------------------------------
    // Discard / mill
    // ------------------------------------------------------------------

    /// Discards a specific card from a player's hand (CR 701.9).
    pub fn discard(
        &mut self,
        p: PlayerId,
        card: ObjectId,
        source: Option<ObjectId>,
    ) -> Option<ObjectId> {
        if self.obj(card).zone != Zone::Hand(p) {
            return None;
        }
        let owner = self.obj(card).owner;
        let new = self.move_object_ev(MoveEv {
            obj: card,
            to: Zone::Graveyard(owner),
            pos: LibraryPosition::Top,
            cause: MoveCause::Discard,
            by: Some(p),
            etb: EtbInfo::default(),
            source,
        });
        // The card is discarded even if a replacement effect puts it elsewhere (madness).
        if let Some(n) = new {
            self.emit(Event::Discarded { player: p, card: n });
        }
        new
    }

    /// Mills `n` cards (CR 701.17): puts the top N cards into the graveyard simultaneously.
    pub fn mill(&mut self, p: PlayerId, n: u32) -> Vec<ObjectId> {
        let lib = &self.players[p.idx()].library;
        let k = (n as usize).min(lib.len());
        let top: Vec<ObjectId> = lib[lib.len() - k..].iter().rev().copied().collect();
        let owner = p;
        let moves = top
            .iter()
            .map(|c| MoveEv {
                obj: *c,
                to: Zone::Graveyard(owner),
                pos: LibraryPosition::Top,
                cause: MoveCause::Mill,
                by: Some(p),
                etb: EtbInfo::default(),
                source: None,
            })
            .collect();
        let res: Vec<ObjectId> = self.move_objects(moves).into_iter().flatten().collect();
        let milled: Vec<ObjectId> = res
            .iter()
            .copied()
            .filter(|c| matches!(self.obj(*c).zone, Zone::Graveyard(_)))
            .collect();
        if !milled.is_empty() {
            self.emit(Event::Milled {
                player: p,
                cards: milled.clone(),
            });
        }
        milled
    }

    // ------------------------------------------------------------------
    // Tap / untap (CR 701.26)
    // ------------------------------------------------------------------

    pub fn tap(&mut self, obj: ObjectId) -> bool {
        self.tap_ex(obj, false)
    }

    pub fn tap_ex(&mut self, obj: ObjectId, for_mana: bool) -> bool {
        let o = self.obj(obj);
        if o.zone != Zone::Battlefield || o.tapped {
            return false;
        }
        self.objects[obj.0 as usize].tapped = true;
        self.dirty = true;
        self.emit(Event::Tapped { obj, for_mana });
        true
    }

    pub fn untap(&mut self, obj: ObjectId) -> bool {
        let o = self.obj(obj);
        if o.zone != Zone::Battlefield || !o.tapped {
            return false;
        }
        self.objects[obj.0 as usize].tapped = false;
        self.dirty = true;
        self.emit(Event::Untapped { obj });
        true
    }

    /// Whether a permanent doesn't untap during its controller's untap step (CR 502.3).
    pub fn doesnt_untap(&self, obj: ObjectId) -> bool {
        let o = self.obj(obj);
        if o.exerted {
            return false;
        }
        self.statics.restrictions.iter().any(|(s, c, r)| match r {
            Restriction::DoesntUntap(f) => self.matches(obj, f, &Ctx::new(Some(*s), *c)),
            _ => false,
        }) || self.rule_effects.iter().any(|e| match &e.restriction {
            Restriction::DoesntUntap(f) => {
                e.objects.as_ref().is_none_or(|v| v.contains(&obj))
                    && self.matches(obj, f, &Ctx::new(e.source, e.controller))
            }
            _ => false,
        })
    }

    // ------------------------------------------------------------------
    // Destroy / sacrifice / exile
    // ------------------------------------------------------------------

    /// Destroys a permanent (CR 701.8). Returns true if it left the battlefield.
    pub fn destroy(&mut self, obj: ObjectId, source: Option<ObjectId>) -> bool {
        if self.dirty {
            self.recompute();
        }
        if !self.is_live(obj) || self.obj(obj).zone != Zone::Battlefield {
            return false;
        }
        // CR 702.12b: indestructible permanents can't be destroyed.
        if self.obj(obj).has_keyword(KeywordKind::Indestructible) {
            return false;
        }
        let evs = self.replace(ReplEvent::Destroy { obj, source });
        for e in evs {
            self.execute_repl_event(e);
        }
        self.run_post_replacement_effects();
        self.recompute();
        !self.is_live(obj)
    }

    /// Destroys several permanents simultaneously ("destroy all creatures").
    pub fn destroy_all(
        &mut self,
        objs: Vec<ObjectId>,
        source: Option<ObjectId>,
        no_regen: bool,
    ) -> Vec<ObjectId> {
        if self.dirty {
            self.recompute();
        }
        let mut moves = Vec::new();
        for obj in objs {
            if !self.is_live(obj) || self.obj(obj).zone != Zone::Battlefield {
                continue;
            }
            if self.obj(obj).has_keyword(KeywordKind::Indestructible) {
                continue;
            }
            let evs = if no_regen {
                vec![ReplEvent::Destroy { obj, source }]
            } else {
                self.replace(ReplEvent::Destroy { obj, source })
            };
            for e in evs {
                match e {
                    ReplEvent::Destroy { obj, .. } => {
                        let owner = self.obj(obj).owner;
                        moves.push(MoveEv {
                            obj,
                            to: Zone::Graveyard(owner),
                            pos: LibraryPosition::Top,
                            cause: MoveCause::Destroy,
                            by: None,
                            etb: EtbInfo::default(),
                            source,
                        });
                    }
                    other => self.execute_repl_event(other),
                }
            }
        }
        let ids: Vec<ObjectId> = moves.iter().map(|m| m.obj).collect();
        let res = self.move_objects(moves);
        for (old, new) in ids.iter().zip(res.iter()) {
            if new.is_some() {
                self.emit(Event::Destroyed { obj: *old });
            }
        }
        res.into_iter().flatten().collect()
    }

    /// Sacrifices a permanent (CR 701.21). Only the controller can sacrifice.
    pub fn sacrifice(&mut self, obj: ObjectId, by: PlayerId) -> Option<ObjectId> {
        if self.dirty {
            self.recompute();
        }
        let o = self.obj(obj);
        if !self.is_live(obj) || o.zone != Zone::Battlefield || o.controller != by {
            return None;
        }
        if self.cant_be_sacrificed(obj) {
            return None;
        }
        let owner = o.owner;
        let new = self.move_object_ev(MoveEv {
            obj,
            to: Zone::Graveyard(owner),
            pos: LibraryPosition::Top,
            cause: MoveCause::Sacrifice,
            by: Some(by),
            etb: EtbInfo::default(),
            source: None,
        });
        self.history.sacrificed.push((by, obj));
        self.emit(Event::Sacrificed { obj, player: by });
        new
    }

    pub fn cant_be_sacrificed(&self, obj: ObjectId) -> bool {
        self.statics.restrictions.iter().any(|(s, c, r)| match r {
            Restriction::CantBeSacrificed(f) => self.matches(obj, f, &Ctx::new(Some(*s), *c)),
            _ => false,
        })
    }

    /// Exiles an object (CR 701.13).
    pub fn exile_object(&mut self, obj: ObjectId, source: Option<ObjectId>) -> Option<ObjectId> {
        self.move_object_ev(MoveEv {
            obj,
            to: Zone::Exile,
            pos: LibraryPosition::Top,
            cause: MoveCause::Exile,
            by: None,
            etb: EtbInfo::default(),
            source,
        })
    }

    // ------------------------------------------------------------------
    // Counters (CR 122)
    // ------------------------------------------------------------------

    /// Puts counters on an object or player. Returns the number actually placed.
    pub fn add_counters(
        &mut self,
        target: Entity,
        kind: &str,
        n: u32,
        source: Option<ObjectId>,
    ) -> u32 {
        if n == 0 {
            return 0;
        }
        if self.dirty {
            self.recompute();
        }
        let mut placed = 0;
        for e in self.replace(ReplEvent::AddCounters {
            target,
            kind: kind.into(),
            n,
            source,
        }) {
            if let ReplEvent::AddCounters { n, .. } = &e {
                placed += *n;
            }
            self.execute_repl_event(e);
        }
        self.run_post_replacement_effects();
        placed
    }

    fn perform_add_counters(&mut self, target: Entity, kind: CounterKind, n: u32) {
        if n == 0 {
            return;
        }
        match target {
            Entity::Object(o) => {
                if !self.is_live(o) {
                    return;
                }
                *self.objects[o.0 as usize]
                    .counters
                    .entry(kind.clone())
                    .or_insert(0) += n;
            }
            Entity::Player(p) => {
                *self.players[p.idx()]
                    .counters
                    .entry(kind.clone())
                    .or_insert(0) += n;
            }
        }
        self.history.counters_put += n;
        self.dirty = true;
        self.emit(Event::CountersAdded { target, kind, n });
    }

    /// Removes up to `n` counters of a kind. Returns the number removed.
    pub fn remove_counters(&mut self, target: Entity, kind: &str, n: u32) -> u32 {
        let have = match target {
            Entity::Object(o) => self.obj(o).counter(kind),
            Entity::Player(p) => self.player(p).counter(kind),
        };
        let k = have.min(n);
        if k == 0 {
            return 0;
        }
        match target {
            Entity::Object(o) => {
                let c = self.objects[o.0 as usize].counters.get_mut(kind).unwrap();
                *c -= k;
                if *c == 0 {
                    self.objects[o.0 as usize].counters.remove(kind);
                }
            }
            Entity::Player(p) => {
                let c = self.players[p.idx()].counters.get_mut(kind).unwrap();
                *c -= k;
                if *c == 0 {
                    self.players[p.idx()].counters.remove(kind);
                }
            }
        }
        self.dirty = true;
        self.emit(Event::CountersRemoved {
            target,
            kind: kind.into(),
            n: k,
        });
        k
    }

    // ------------------------------------------------------------------
    // Life (CR 119)
    // ------------------------------------------------------------------

    pub fn gain_life(&mut self, p: PlayerId, n: u32) -> u32 {
        if n == 0 || !self.player(p).in_game() || self.cant_gain_life(p) {
            return 0;
        }
        let before = self.player(p).life;
        for e in self.replace(ReplEvent::GainLife {
            player: p,
            amount: n,
        }) {
            self.execute_repl_event(e);
        }
        self.run_post_replacement_effects();
        (self.player(p).life - before).max(0) as u32
    }

    fn perform_gain_life(&mut self, p: PlayerId, n: u32) {
        if n == 0 || self.cant_gain_life(p) {
            return;
        }
        self.players[p.idx()].life += n as i32;
        *self.history.life_gained.entry(p).or_insert(0) += n;
        self.emit(Event::LifeGained {
            player: p,
            amount: n,
        });
    }

    pub fn lose_life(&mut self, p: PlayerId, n: u32) -> u32 {
        if n == 0 || !self.player(p).in_game() || self.cant_lose_life(p) {
            return 0;
        }
        let before = self.player(p).life;
        for e in self.replace(ReplEvent::LoseLife {
            player: p,
            amount: n,
        }) {
            self.execute_repl_event(e);
        }
        self.run_post_replacement_effects();
        (before - self.player(p).life).max(0) as u32
    }

    fn perform_lose_life(&mut self, p: PlayerId, n: u32) {
        if n == 0 || self.cant_lose_life(p) {
            return;
        }
        self.players[p.idx()].life -= n as i32;
        *self.history.life_lost.entry(p).or_insert(0) += n;
        self.emit(Event::LifeLost {
            player: p,
            amount: n,
        });
    }

    /// Paying life (CR 119.4): a player can pay life only if their life total is at least
    /// that amount; paying 0 is always possible.
    pub fn can_pay_life(&self, p: PlayerId, n: u32) -> bool {
        n == 0 || (self.player(p).life >= n as i32 && !self.cant_lose_life(p))
    }

    pub fn pay_life(&mut self, p: PlayerId, n: u32) -> bool {
        if !self.can_pay_life(p, n) {
            return false;
        }
        if n > 0 {
            self.perform_lose_life(p, n);
        }
        true
    }

    pub fn cant_gain_life(&self, p: PlayerId) -> bool {
        self.player_restricted(p, |r| matches!(r, Restriction::CantGainLife(_)))
    }

    pub fn cant_lose_life(&self, p: PlayerId) -> bool {
        self.player_restricted(p, |r| matches!(r, Restriction::CantLoseLife(_)))
    }

    /// Checks player-filter restrictions from statics and resolved rule effects.
    pub fn player_restricted(&self, p: PlayerId, which: impl Fn(&Restriction) -> bool) -> bool {
        let check = |r: &Restriction, ctx: &Ctx| -> bool {
            if !which(r) {
                return false;
            }
            match r {
                Restriction::CantGainLife(f)
                | Restriction::CantLoseLife(f)
                | Restriction::CantLoseGame(f)
                | Restriction::CantWinGame(f)
                | Restriction::CantSearch(f)
                | Restriction::SorcerySpeedOnly(f)
                | Restriction::CantPlayLands(f)
                | Restriction::MaxDrawsPerTurn(f, _)
                | Restriction::MaxSpellsPerTurn(f, _) => self.player_filter_matches(f, p, ctx),
                _ => false,
            }
        };
        self.statics
            .restrictions
            .iter()
            .any(|(s, c, r)| check(r, &Ctx::new(Some(*s), *c)))
            || self
                .rule_effects
                .iter()
                .any(|e| check(&e.restriction, &Ctx::new(e.source, e.controller)))
    }

    // ------------------------------------------------------------------
    // Damage (CR 120)
    // ------------------------------------------------------------------

    /// Deals damage from a source to a single recipient.
    pub fn deal_damage(&mut self, source: ObjectId, target: Entity, amount: u32, combat: bool) {
        self.deal_damage_batch(vec![(source, target, amount)], combat);
    }

    /// Deals several damage events simultaneously (e.g. combat damage, CR 510.2).
    pub fn deal_damage_batch(&mut self, events: Vec<(ObjectId, Entity, u32)>, combat: bool) {
        if self.dirty {
            self.recompute();
        }
        let mut finals = Vec::new();
        for (s, t, a) in events {
            // CR 120.8 / 614.7a: 0 damage isn't dealt.
            if a == 0 || !self.valid_damage_recipient(t) {
                continue;
            }
            if self.damage_cant_be_prevented() {
                // Prevention effects don't apply, but other replacements still do.
            }
            finals.extend(self.replace(ReplEvent::Damage {
                source: s,
                target: t,
                amount: a,
                combat,
            }));
        }
        let mut lifelink_gains: Vec<(PlayerId, u32)> = Vec::new();
        for e in finals {
            match e {
                ReplEvent::Damage {
                    source,
                    target,
                    amount,
                    combat,
                } => {
                    if self.valid_damage_recipient(target) && amount > 0 {
                        self.perform_damage(source, target, amount, combat);
                        // CR 120.3f: lifelink — damage causes the source's controller to gain life.
                        if self.obj(source).has_keyword(KeywordKind::Lifelink) {
                            lifelink_gains.push((self.obj(source).controller, amount));
                        }
                    }
                }
                other => self.execute_repl_event(other),
            }
        }
        for (p, n) in lifelink_gains {
            self.gain_life(p, n);
        }
        self.run_post_replacement_effects();
        self.recompute();
    }

    fn damage_cant_be_prevented(&self) -> bool {
        self.statics
            .restrictions
            .iter()
            .any(|(_, _, r)| matches!(r, Restriction::DamageCantBePrevented))
            || self
                .rule_effects
                .iter()
                .any(|e| matches!(e.restriction, Restriction::DamageCantBePrevented))
    }

    /// Applies the results of damage (CR 120.3). Lifelink is handled by the caller.
    pub(crate) fn perform_damage(
        &mut self,
        source: ObjectId,
        target: Entity,
        amount: u32,
        combat: bool,
    ) {
        let src = self.obj(source).clone();
        let infect = src.has_keyword(KeywordKind::Infect);
        let wither = src.has_keyword(KeywordKind::Wither);
        let deathtouch = src.has_keyword(KeywordKind::Deathtouch);
        match target {
            Entity::Player(p) => {
                if infect {
                    // CR 120.3b
                    self.perform_add_counters(Entity::Player(p), counters::POISON.into(), amount);
                } else {
                    // CR 120.3a (life loss can be replaced as "lose life")
                    for e in self.replace(ReplEvent::LoseLife { player: p, amount }) {
                        self.execute_repl_event(e);
                    }
                }
                *self.history.damage_dealt_to_players.entry(p).or_insert(0) += amount;
                if combat && src.is_commander {
                    // CR 903.10a
                    *self.players[p.idx()]
                        .commander_damage
                        .entry(src.chars.name.clone())
                        .or_insert(0) += amount;
                }
            }
            Entity::Object(o) => {
                let obj = self.obj(o).clone();
                if obj.is(CardType::Planeswalker) {
                    // CR 120.3c
                    self.remove_counters(Entity::Object(o), counters::LOYALTY, amount);
                }
                if obj.is(CardType::Battle) {
                    // CR 120.3h
                    self.remove_counters(Entity::Object(o), counters::DEFENSE, amount);
                }
                if obj.is_creature() {
                    if wither || infect {
                        // CR 120.3d
                        self.perform_add_counters(
                            Entity::Object(o),
                            counters::MINUS1.into(),
                            amount,
                        );
                    } else {
                        // CR 120.3e
                        self.objects[o.0 as usize].damage += amount;
                    }
                    if deathtouch {
                        self.objects[o.0 as usize].deathtouch_damage = true;
                    }
                }
                self.history.objects_dealt_damage.insert(o);
            }
        }
        self.dirty = true;
        self.log(|g| {
            format!(
                "{} deals {} damage to {:?}",
                g.describe(source),
                amount,
                target
            )
        });
        self.emit(Event::Damage {
            source,
            target,
            amount,
            combat,
        });
        crate::keyword_impls::after_damage(self, source, target, amount, combat);
    }

    // ------------------------------------------------------------------
    // Tokens (CR 111)
    // ------------------------------------------------------------------

    /// Creates `count` tokens. Returns the ids of tokens created.
    pub fn create_tokens(
        &mut self,
        controller: PlayerId,
        spec: TokenCreate,
        count: u32,
        source: Option<ObjectId>,
    ) -> Vec<ObjectId> {
        if count == 0 {
            return vec![];
        }
        if self.dirty {
            self.recompute();
        }
        let before = self.objects.len();
        for e in self.replace(ReplEvent::CreateTokens {
            controller,
            spec: Box::new(spec),
            count,
            source,
        }) {
            self.execute_repl_event(e);
        }
        self.run_post_replacement_effects();
        self.recompute();
        (before..self.objects.len())
            .map(|i| ObjectId(i as u32))
            .filter(|id| {
                let o = self.obj(*id);
                o.kind == ObjKind::Token && o.zone == Zone::Battlefield && o.next.is_none()
            })
            .collect()
    }

    fn perform_create_token(
        &mut self,
        controller: PlayerId,
        spec: &TokenCreate,
    ) -> Option<ObjectId> {
        let tok = self.create_token_object(spec.chars.clone(), controller);
        if let Some(card) = &spec.card {
            self.objects[tok.0 as usize].card = Some(card.clone());
        }
        let mut etb = EtbInfo {
            tapped: spec.tapped,
            controller: Some(controller),
            ..Default::default()
        };
        if let Some(src) = spec.copy_of {
            etb.copy_of = Some(src);
            etb.copy_exceptions = spec.copy_exceptions.clone();
        }
        etb.attacking = spec.attacking;
        let new = self.move_object_ev(MoveEv {
            obj: tok,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(controller),
            etb,
            source: None,
        })?;
        *self.history.tokens_created.entry(controller).or_insert(0) += 1;
        self.emit(Event::TokenCreated {
            obj: new,
            controller,
        });
        Some(new)
    }

    // ------------------------------------------------------------------
    // Attaching (CR 701.3)
    // ------------------------------------------------------------------

    /// Attaches `obj` to `to`. Returns false if it couldn't be attached.
    pub fn attach(&mut self, obj: ObjectId, to: Entity) -> bool {
        if self.dirty {
            self.recompute();
        }
        if !self.is_live(obj) || self.obj(obj).zone != Zone::Battlefield {
            return false;
        }
        if self.obj(obj).attached_to == Some(to) {
            // CR 701.3b: already attached — nothing happens.
            return false;
        }
        if !crate::attach::can_attach(self, obj, to) {
            return false;
        }
        if let Some(prev) = self.obj(obj).attached_to {
            self.emit(Event::Unattached { obj, from: prev });
        }
        let ts = self.new_timestamp();
        let o = &mut self.objects[obj.0 as usize];
        o.attached_to = Some(to);
        // CR 613.7e: new timestamp when attached.
        o.timestamp = ts;
        self.dirty = true;
        self.emit(Event::Attached { obj, to });
        true
    }

    pub fn unattach(&mut self, obj: ObjectId) {
        if let Some(prev) = self.obj(obj).attached_to {
            self.objects[obj.0 as usize].attached_to = None;
            self.dirty = true;
            self.emit(Event::Unattached { obj, from: prev });
        }
    }

    // ------------------------------------------------------------------
    // Mana
    // ------------------------------------------------------------------

    pub fn add_mana(
        &mut self,
        p: PlayerId,
        mana: Vec<crate::mana::Mana>,
        source: Option<ObjectId>,
    ) {
        let produced: Vec<crate::mana::ManaType> = mana.iter().map(|m| m.ty).collect();
        for m in mana {
            self.players[p.idx()].mana_pool.add(m);
        }
        self.emit(Event::ManaAdded { player: p, source });
        // CR 106.12a: the permanent whose {T} mana ability is resolving was tapped for mana.
        if let Some(obj) = source.filter(|s| self.mana_ability_resolving == Some(*s)) {
            if !produced.is_empty() {
                self.emit(Event::TappedForMana {
                    obj,
                    player: p,
                    produced,
                });
            }
        }
    }
}
