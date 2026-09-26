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
        // CR 400.3: objects go to their owner's library, hand, or graveyard.
        let moves: Vec<MoveEv> = moves
            .into_iter()
            .map(|mut m| {
                m.to = crate::zones::owners_zone(self, m.obj, m.to);
                m
            })
            .collect();
        // Apply replacement effects to each move individually.
        let mut finals: Vec<(usize, ReplEvent)> = Vec::new();
        // CR 614.13a, 614.13c: while effects that modify how these objects enter are
        // applied, the objects entering simultaneously can't be chosen or moved by them.
        let prev_entering = std::mem::replace(
            &mut self.entering,
            moves
                .iter()
                .filter(|m| m.to == Zone::Battlefield)
                .map(|m| m.obj)
                .collect(),
        );
        // CR 616.1: when several players choose among replacement effects for
        // simultaneous events, they do so in APNAP order.
        let apnap = self.apnap();
        let mut order: Vec<usize> = (0..moves.len()).collect();
        order.sort_by_key(|i| {
            let o = self.obj(moves[*i].obj);
            let p = match o.zone {
                Zone::Battlefield | Zone::Stack => o.controller,
                _ => o.owner,
            };
            apnap.iter().position(|x| *x == p).unwrap_or(usize::MAX)
        });
        for (i, m) in order.into_iter().map(|i| (i, &moves[i])) {
            if !self.can_move(m.obj) {
                continue;
            }
            // CR 614.17d: a "can't enter" effect stops the event; it isn't replaced
            // (CR 614.17c).
            if m.to == Zone::Battlefield && self.cant_enter(m) || self.move_forbidden(m) {
                continue;
            }
            for e in self.replace(ReplEvent::Move(m.clone())) {
                // An object that can't enter the battlefield stays where it is. This also
                // covers moves a replacement effect redirected to the battlefield, and
                // entries a replacement modified (CR 614.17d: check the permanent as it
                // would exist, taking those replacements into account).
                let e = match e {
                    ReplEvent::Move(mut mv) => {
                        mv.to = crate::zones::owners_zone(self, mv.obj, mv.to);
                        if mv.to == Zone::Battlefield && self.cant_enter(&mv)
                            || self.move_forbidden(&mv)
                        {
                            continue;
                        }
                        // What it enters attached to (CR 301.5e, 303.4f–i, 310.10). An Aura
                        // with nothing it can enchant stays where it is.
                        if mv.to == Zone::Battlefield
                            && !crate::attach::entry_attachment(self, &mut mv)
                        {
                            for e in crate::attach::aura_left_on_stack(self, &mv) {
                                finals.push((i, e));
                            }
                            continue;
                        }
                        ReplEvent::Move(mv)
                    }
                    other => other,
                };
                finals.push((i, e));
            }
        }
        // CR 613.7m: objects entering the battlefield simultaneously get timestamps in
        // APNAP order.
        self.order_simultaneous_entries(&mut finals);
        // CR 401.4, 404.3: the owner arranges cards put into a library position or a
        // graveyard at the same time.
        crate::zones::order_simultaneous(self, &mut finals);
        // CR 701.24g: a position in a library that's shuffled at the same time.
        crate::shuffle_rules::positions_after_shuffles(self, &mut finals);
        // Look back in time for leaves-the-battlefield triggers and other zone-change
        // triggers that look back (CR 603.10a): leaving the battlefield, a graveyard, or
        // the stack, or a public object being put into a hand or library.
        let leaving = finals.iter().any(|(_, e)| match e {
            ReplEvent::Move(m) => {
                let from = self.obj(m.obj).zone;
                from != m.to
                    && (matches!(from, Zone::Battlefield | Zone::Graveyard(_) | Zone::Stack)
                        || (from.is_public()
                            && from != Zone::Nowhere
                            && matches!(m.to, Zone::Hand(_) | Zone::Library(_))))
            }
            _ => false,
        });
        let lookback = if leaving {
            Some(Arc::new(self.lookback_snapshot()))
        } else {
            None
        };
        let mut out: Vec<Option<ObjectId>> = vec![None; moves.len()];
        let entering: Vec<ObjectId> = finals
            .iter()
            .filter_map(|(_, e)| match e {
                ReplEvent::Move(m) if m.to == Zone::Battlefield => Some(m.obj),
                _ => None,
            })
            .collect();
        self.entering = entering;
        // CR 406.4: cards exiled face down together form a pile.
        let face_down_exiles: Vec<(usize, Option<ObjectId>)> = finals
            .iter()
            .filter_map(|(i, e)| match e {
                ReplEvent::Move(m) if m.to == Zone::Exile && m.etb.face_down.is_some() => {
                    Some((*i, m.source))
                }
                _ => None,
            })
            .collect();
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
        self.entering = prev_entering;
        crate::zones::face_down_exiled(
            self,
            face_down_exiles
                .into_iter()
                .filter_map(|(i, src)| out[i].map(|o| (o, src)))
                .collect(),
        );
        self.run_post_replacement_effects();
        self.recompute();
        let entered: Vec<ObjectId> = out.iter().flatten().copied().collect();
        self.entered_simultaneously(&entered);
        out
    }

    /// Whether a "can't enter the battlefield" effect from a static ability applies to an
    /// object as it currently exists. Moves use [`Game::cant_enter`], which checks the
    /// object as it would exist on the battlefield (CR 614.17d).
    pub fn cant_enter_battlefield(&self, obj: ObjectId) -> bool {
        self.statics.restrictions.iter().any(|(s, c, r)| match r {
            Restriction::CantEnterBattlefield(f) | Restriction::CantEnter(f) => {
                self.matches(obj, f, &Ctx::new(Some(*s), *c))
            }
            _ => false,
        })
    }

    /// Zone changes the rules forbid outright; the object stays where it is. Instant and
    /// sorcery cards (and token copies of them) can't enter the battlefield unless they
    /// enter face down (CR 110.4, 111.5, 708.2), and nontraditional cards can't be brought
    /// into the game from outside it (CR 108.5) — a dungeon card only by venturing into
    /// the dungeon (CR 309.2a, 309.2d).
    pub fn move_forbidden(&self, mv: &MoveEv) -> bool {
        let o = self.obj(mv.obj);
        if mv.to == Zone::Battlefield && mv.etb.face_down.is_none() {
            let face = if mv.etb.transformed {
                Some(FaceState::Back)
            } else {
                mv.etb.face
            };
            let types = match (face, &o.card) {
                (Some(f), Some(card)) if o.kind == ObjKind::Card => {
                    card.characteristics(f).card_types
                }
                _ => o.chars.card_types,
            };
            if types.contains(CardType::Instant) || types.contains(CardType::Sorcery) {
                return true;
            }
        }
        if matches!(o.zone, Zone::Outside(_)) && !matches!(mv.to, Zone::Outside(_)) {
            if let Some(card) = &o.card {
                let venture = mv.cause == MoveCause::Venture
                    && mv.to == Zone::Command
                    && card.front().chars.card_types.contains(CardType::Dungeon);
                if crate::variants::is_nontraditional(card) && !venture {
                    return true;
                }
            }
        }
        // CR 400.4b, 407.3, 407.4: the command zone and the ante zone; CR 309.2c, 315.3:
        // dungeon and conspiracy cards.
        crate::zones::move_forbidden(self, mv) || crate::variants::stays_in_command_zone(self, mv)
    }

    /// Whether an object can be moved to a zone: it's a current object in some zone, or a
    /// newly created object (a token or card) that hasn't been put anywhere yet.
    pub fn can_move(&self, id: ObjectId) -> bool {
        let o = self.obj(id);
        // A token that has left the battlefield stays where it is (CR 111.8). (A copy of a
        // permanent spell on the stack becomes a token as it resolves, CR 111.13.)
        if o.kind == ObjKind::Token
            && !matches!(o.zone, Zone::Battlefield | Zone::Nowhere | Zone::Stack)
        {
            return false;
        }
        self.is_live(id)
            || (o.zone == Zone::Nowhere
                && o.next.is_none()
                && o.prev.is_none()
                && o.kind != ObjKind::StackAbility)
    }

    /// The player who will control a permanent entering the battlefield with this move.
    pub fn entry_controller(&self, m: &MoveEv) -> PlayerId {
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

    /// Reorders simultaneous moves onto the battlefield so they receive timestamps in
    /// APNAP order (CR 613.7m): each player's objects in the order that player chooses,
    /// the active player's first. Other moves keep their places.
    fn order_simultaneous_entries(&mut self, finals: &mut [(usize, ReplEvent)]) {
        let slots: Vec<usize> = finals
            .iter()
            .enumerate()
            .filter(|(_, (_, e))| matches!(e, ReplEvent::Move(m) if m.to == Zone::Battlefield))
            .map(|(i, _)| i)
            .collect();
        if slots.len() < 2 {
            return;
        }
        let mut entries: Vec<Option<(usize, ReplEvent)>> =
            slots.iter().map(|i| Some(finals[*i].clone())).collect();
        let controller = |g: &Game, e: &ReplEvent| match e {
            ReplEvent::Move(m) => g.entry_controller(m),
            _ => g.turn.active,
        };
        let mut ordered: Vec<(usize, ReplEvent)> = Vec::new();
        for p in self.apnap() {
            let mine: Vec<usize> = entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.as_ref().is_some_and(|(_, ev)| controller(self, ev) == p))
                .map(|(k, _)| k)
                .collect();
            let order: Vec<usize> = if mine.len() >= 2 {
                let names = mine
                    .iter()
                    .map(|k| match &entries[*k] {
                        Some((_, ReplEvent::Move(m))) => self.describe(m.obj),
                        _ => String::new(),
                    })
                    .collect();
                self.ask_order(
                    p,
                    "Order objects entering the battlefield (first = oldest)",
                    names,
                )
            } else {
                (0..mine.len()).collect()
            };
            for i in order {
                if let Some(e) = entries[mine[i]].take() {
                    ordered.push(e);
                }
            }
        }
        ordered.extend(entries.into_iter().flatten());
        for (k, i) in slots.iter().enumerate() {
            finals[*i] = ordered[k].clone();
        }
    }

    /// Snapshot of the triggered abilities that function right now, with their sources and
    /// controllers, for abilities that look back in time (CR 603.10).
    pub fn lookback_snapshot(&self) -> LookbackSnapshot {
        LookbackSnapshot {
            sources: self.current_trigger_sources(),
        }
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
        // CR 708.9: a face-down permanent or spell is revealed as it leaves.
        crate::facedown::moving(self, old_id, m.to);
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
        if from == Zone::Exile && kind == ObjKind::CardCopy {
            crate::designations::prepared_copy_left_exile(self, old_id);
        }
        let old_controller = self.obj(old_id).controller;
        let old_was_creature = self.obj(old_id).is_creature();
        // An Aura, Equipment, or Fortification leaving the battlefield becomes unattached.
        let was_attached_to = if from == Zone::Battlefield {
            self.obj(old_id).attached_to
        } else {
            None
        };
        let new_id = self.create_incarnation(old_id, m.to);
        // CR 704.6d: a commander put into a graveyard or exile since the last check.
        if self.obj(new_id).is_commander && matches!(m.to, Zone::Graveyard(_) | Zone::Exile) {
            self.commander_moved_since_last_sba.insert(new_id);
        }
        if from == Zone::Stack && m.to == Zone::Battlefield {
            // Effects of resolved spells and abilities that changed a permanent spell
            // continue to apply to the permanent it becomes (CR 112.4, 110.2b, 400.7a).
            // (Prevention effects for damage from it follow it through `Filter::Objects`,
            // CR 400.7c.)
            for e in self.effects.iter_mut() {
                if let Affected::Objects(v) = &mut e.affected {
                    for x in v.iter_mut().filter(|x| **x == old_id) {
                        *x = new_id;
                    }
                }
            }
        }
        if m.to == Zone::Battlefield {
            // Choices made as it entered (CR 614.12a) or while it was cast are the
            // permanent's choices (CR 607.2d), still linked to the abilities that made
            // them.
            let ch = self.obj(old_id).choices.clone();
            let linked = self.obj(old_id).linked_choices.clone();
            let n = &mut self.objects[new_id.0 as usize];
            n.choices = ch;
            for (link, c) in linked {
                n.linked_choices.entry(link).or_insert(c);
            }
        }
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
                let mut copy_effect = None;
                if let Some(src) = m.etb.copy_of {
                    let values = Box::new(self.obj(src).copiable.clone());
                    let id = self.new_effect_id();
                    copy_effect = Some(id);
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
                            exceptions: m
                                .etb
                                .copy_exceptions
                                .iter()
                                .chain(&m.etb.copiable_mods)
                                .cloned()
                                .collect(),
                        }),
                        created_turn: self.turn.number,
                    });
                } else if !m.etb.copiable_mods.is_empty() {
                    // CR 707.2, 613.2a: an "as this enters" ability that sets power and
                    // toughness modifies its copiable values (its own, when it isn't a
                    // copy).
                    let id = self.new_effect_id();
                    let ts = self.obj(new_id).timestamp;
                    self.effects.push(ContinuousEffect {
                        id,
                        source: Some(new_id),
                        controller,
                        timestamp: ts,
                        duration: Duration::Permanent,
                        affected: Affected::Objects(vec![new_id]),
                        mods: vec![],
                        layer1: Some(Layer1::Copiable(m.etb.copiable_mods.clone())),
                        created_turn: self.turn.number,
                    });
                }
                self.battlefield.push(new_id);
                if from == Zone::Stack {
                    // A text change made to a permanent spell continues to apply to the
                    // permanent it becomes (Sleight of Mind rulings).
                    for e in self.effects.iter_mut() {
                        if e.mods
                            .iter()
                            .any(|x| matches!(x, Modification::ChangeText { .. }))
                            || self.carried_effects.contains(&e.id)
                        {
                            if let Affected::Objects(v) = &mut e.affected {
                                if v.contains(&old_id) {
                                    v.push(new_id);
                                }
                            }
                        }
                    }
                }
                if let Some((source, ctl, mods)) = m.etb.with_mods.clone() {
                    // CR 611.2e, 613.7n.
                    let id = self.new_effect_id();
                    let ts = self.new_timestamp();
                    self.effects.push(ContinuousEffect {
                        id,
                        source,
                        controller: ctl,
                        timestamp: ts,
                        duration: Duration::Permanent,
                        affected: Affected::Objects(vec![new_id]),
                        mods,
                        layer1: None,
                        created_turn: self.turn.number,
                    });
                }
                self.recompute();
                // Counters it enters with (CR 122.6): planeswalker loyalty (CR 306.5b),
                // battle defense (CR 310.4), Saga lore (CR 714.3a), plus effects.
                let mut counters_to_add = m.etb.counters.clone();
                // Copy exceptions that are additional effects or conditional
                // (CR 707.9e–707.9g).
                let copy_extras = crate::copy_rules::apply_copy_extras(
                    self,
                    new_id,
                    copy_effect,
                    &m.etb.copy_extras,
                    &mut counters_to_add,
                );
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
                                let ts = self.new_timestamp();
                                let ob = &mut self.objects[t.0 as usize];
                                *ob.counters.entry(kind.clone()).or_insert(0) += n;
                                ob.counter_timestamps.insert(kind.clone(), ts);
                                // Counters it's given as it enters are "put" on it (CR 122.6),
                                // e.g. a Saga's first lore counter triggers chapter I (714.3a).
                                if n > 0 {
                                    self.history.counters_put += n;
                                    self.emit(Event::CountersAdded {
                                        target: Entity::Object(t),
                                        kind,
                                        n,
                                    });
                                }
                            }
                        }
                    }
                }
                // CR 310.9a: as a battle enters, its controller chooses its protector.
                crate::battle::choose_protector_as_it_enters(self, new_id);
                // "As this enters" effects (CR 614.1c).
                for (mut c, e) in m.etb.as_enters.clone().into_iter().chain(copy_extras) {
                    c.source = Some(new_id);
                    c.controller = controller;
                    let before = self.effects.len();
                    self.exec(&e, &mut c);
                    crate::layers::as_enters_copiable(self, new_id, before);
                }
                if let Some(eid) = copy_effect {
                    // CR 607.2d, 707.9: choices made as it entered by the abilities it has
                    // through the copy effect belong to those abilities as copied.
                    let n = &mut self.objects[new_id.0 as usize];
                    let copied: Vec<(u16, crate::object::Choices)> = n
                        .linked_choices
                        .iter()
                        .map(|(k, c)| (crate::layers::copied_link(*k, eid), c.clone()))
                        .collect();
                    for (k, c) in copied {
                        n.linked_choices.entry(k).or_insert(c);
                    }
                    self.dirty = true;
                }
                if let Some(target) = m.etb.attacking {
                    crate::combat::put_onto_battlefield_attacking(self, new_id, target);
                }
                if let Some(attacker) = m.etb.blocking {
                    crate::combat::put_onto_battlefield_blocking(self, new_id, attacker);
                }
            }
            Zone::Library(p) => {
                let lib = &mut self.players[p.idx()].library;
                match m.pos {
                    LibraryPosition::Top => lib.push(new_id),
                    LibraryPosition::Bottom | LibraryPosition::BottomRandom => {
                        lib.insert(0, new_id)
                    }
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
                let link = m.etb.link.unwrap_or(self.current_link);
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
        if let Some(host) = was_attached_to {
            // CR 603.10c: looks back in time (to the snapshot taken for this move).
            self.emit(Event::Unattached {
                obj: old_id,
                from: host,
            });
        }
        if from == Zone::Battlefield {
            // CR 730.3: the other components of a merged or melded permanent.
            crate::merge::after_leaving(self, old_id, new_id, &m);
        }
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
            ReplEvent::LoseLife { player, amount, .. } => self.perform_lose_life(player, amount),
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
                // The move to the graveyard is itself subject to replacement effects
                // ("if it would die, exile it instead"), as in `destroy_all`.
                if self
                    .move_object_ev(MoveEv {
                        obj,
                        to: Zone::Graveyard(owner),
                        pos: LibraryPosition::Top,
                        cause: MoveCause::Destroy,
                        by: None,
                        etb: EtbInfo::default(),
                        source: None,
                    })
                    .is_some()
                {
                    self.emit(Event::Destroyed { obj });
                }
            }
            ReplEvent::LoseGame { player } => self.player_loses(player),
        }
    }

    pub(crate) fn run_post_replacement_effects(&mut self) {
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
        // CR 121.2a: effects referring to the number of cards drawn apply first.
        if let Some(out) = crate::draw_rules::replace_multiple_draws(self, p, n) {
            self.run_post_replacement_effects();
            return out;
        }
        let mut out = Vec::new();
        for _ in 0..n {
            if !self.player(p).in_game() {
                break;
            }
            if self.draw_restricted(p) {
                break;
            }
            // CR 614.11b: cards drawn because a replacement effect replaced the draw aren't
            // the card this draw drew.
            for e in self.replace(ReplEvent::Draw { player: p }) {
                match e {
                    ReplEvent::Draw { player } if player == p => {
                        if let Some(c) = self.perform_draw(player) {
                            out.push(c);
                        }
                    }
                    other => self.execute_repl_event(other),
                }
            }
        }
        self.run_post_replacement_effects();
        out
    }

    /// CR 121.2b: "can't draw more than N cards each turn" applies to individual draws.
    fn draw_restricted(&self, p: PlayerId) -> bool {
        crate::draw_rules::draws_left(self, p) == Some(0)
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
        // "As you draw it" abilities (CR 121.8, 121.9).
        crate::draw_rules::card_drawn(self, p, new, nth);
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
        // CR 701.17d: replacement effects may change how many cards are milled.
        let n = crate::mill_rules::replaced_count(self, p, n);
        // CR 614.13c: cards entering the battlefield from the library aren't milled.
        let lib: Vec<ObjectId> = self.players[p.idx()]
            .library
            .iter()
            .copied()
            .filter(|c| !self.entering.contains(c))
            .collect();
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
        // CR 701.17c: a milled card is found in the zone it moved to, if that's a public
        // zone (it may have been exiled instead of put into the graveyard).
        let milled: Vec<ObjectId> = res
            .iter()
            .copied()
            .filter(|c| {
                !matches!(
                    self.obj(*c).zone,
                    Zone::Library(_) | Zone::Hand(_) | Zone::Outside(_) | Zone::Nowhere
                )
            })
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
        // CR 122.1d: a stun counter's replacement effect.
        if crate::counter_rules::stun_instead_of_untap(self, obj) {
            return false;
        }
        self.objects[obj.0 as usize].tapped = false;
        self.dirty = true;
        self.emit(Event::Untapped { obj });
        true
    }

    /// Whether a permanent doesn't untap during its controller's untap step (CR 502.3).
    pub fn doesnt_untap(&self, obj: ObjectId) -> bool {
        // CR 701.43a: an exerted permanent doesn't untap during its controller's next
        // untap step.
        if crate::kwa::exert::keeps_tapped(self, obj) {
            return true;
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
                // CR 701.19c: "can't be regenerated" makes regeneration shields not
                // apply; other replacement effects still do.
                let mut skip = self.repl_context.last().cloned().unwrap_or_default();
                skip.extend(self.regeneration_keys());
                self.repl_context.push(skip);
                let r = self.replace(ReplEvent::Destroy { obj, source });
                self.repl_context.pop();
                r
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

    /// Keys of all regeneration replacement effects (shields and static regeneration).
    fn regeneration_keys(&self) -> Vec<ReplKey> {
        let regen = |d: &ReplacementDef| matches!(d.action, ReplacementAction::Regenerate);
        let mut keys: Vec<ReplKey> = self
            .replacements
            .iter()
            .filter(|r| regen(&r.def))
            .map(|r| ReplKey::Instance(r.id))
            .collect();
        keys.extend(
            self.statics
                .replacements
                .iter()
                .filter(|(_, _, _, _, d)| regen(d))
                .map(|(s, _, _, a, _)| ReplKey::Static(*s, a.uid)),
        );
        keys
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
                // CR 613.7c: every counter of this kind gets the new counter's timestamp.
                let ts = self.new_timestamp();
                let ob = &mut self.objects[o.0 as usize];
                *ob.counters.entry(kind.clone()).or_insert(0) += n;
                ob.counter_timestamps.insert(kind.clone(), ts);
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
        self.remove_counters_by(target, kind, n, None)
    }

    /// Removes up to `n` counters of a kind; `by` is the player removing them (the
    /// controller of the effect, or the player paying a cost). Returns the number removed.
    pub fn remove_counters_by(
        &mut self,
        target: Entity,
        kind: &str,
        n: u32,
        by: Option<PlayerId>,
    ) -> u32 {
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
            by,
        });
        k
    }

    // ------------------------------------------------------------------
    // Life (CR 119)
    // ------------------------------------------------------------------

    pub fn gain_life(&mut self, p: PlayerId, n: u32) -> u32 {
        if n == 0 || !self.player(p).in_game() {
            return 0;
        }
        if self.cant_gain_life(p) {
            // CR 614.17c: an event that can't happen can be replaced only by a
            // self-replacement effect.
            for e in self.replace_self_only(ReplEvent::GainLife {
                player: p,
                amount: n,
            }) {
                if !matches!(e, ReplEvent::GainLife { .. }) {
                    self.execute_repl_event(e);
                }
            }
            self.run_post_replacement_effects();
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
        crate::life_totals::share_team_life(self, p);
        *self.history.life_gained.entry(p).or_insert(0) += n;
        // Life totals feed conditional statics and P/T-defining values (CR 611.3a).
        self.dirty = true;
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
            from_damage: false,
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
        crate::life_totals::share_team_life(self, p);
        *self.history.life_lost.entry(p).or_insert(0) += n;
        self.dirty = true;
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
        // CR 120.8 / 614.7a: 0 damage isn't dealt.
        let mut events: Vec<(ObjectId, Entity, u32)> = events
            .into_iter()
            .filter(|(_, t, a)| *a > 0 && self.valid_damage_recipient(*t))
            .collect();
        // CR 616.1: players choose among replacement effects for simultaneous events in
        // APNAP order.
        let apnap = self.apnap();
        events.sort_by_key(|(_, t, _)| {
            let p = match t {
                Entity::Player(p) => *p,
                Entity::Object(o) => self.obj(*o).controller,
            };
            apnap.iter().position(|x| *x == p).unwrap_or(usize::MAX)
        });
        // CR 615.7: which simultaneous damage a prevention shield prevents.
        crate::prevention::order_for_shields(self, &mut events, combat);
        let first_event = self.events.len();
        for (s, t, a) in events {
            finals.extend(self.replace(ReplEvent::Damage {
                source: s,
                target: t,
                amount: a,
                combat,
            }));
        }
        crate::prevention::merge_prevention_events(self, first_event);
        // CR 120.10: what would be excess damage, as the damage is about to be dealt.
        let excess_before = crate::excess_damage::thresholds_before(
            self,
            &crate::excess_damage::damage_events(&finals),
        );
        let mut dealt: Vec<(ObjectId, Entity, u32)> = Vec::new();
        // CR 702.15e: each source with lifelink causes one life gain event, even if it
        // dealt damage to several recipients at once.
        let mut lifelink_gains: Vec<(ObjectId, PlayerId, u32)> = Vec::new();
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
                        dealt.push((source, target, amount));
                        // CR 120.3f, 702.15b: lifelink — damage causes the source's
                        // controller (its owner if it has none) to gain that much life.
                        // The source's last known information is used if it has left its
                        // zone (702.15c), whatever zone it deals damage from (702.15d).
                        if self.obj(source).has_keyword(KeywordKind::Lifelink) {
                            let who = self.obj(source).controller;
                            match lifelink_gains.iter_mut().find(|(s, _, _)| *s == source) {
                                Some(g) => g.2 += amount,
                                None => lifelink_gains.push((source, who, amount)),
                            }
                        }
                    }
                }
                other => self.execute_repl_event(other),
            }
        }
        crate::excess_damage::record_excess(self, &excess_before, &dealt, combat);
        for (_, p, n) in lifelink_gains {
            self.gain_life(p, n);
        }
        self.run_post_replacement_effects();
        self.recompute();
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
        let wither = crate::kw::wither::deals_damage_with_wither(self, &src);
        let deathtouch = src.has_keyword(KeywordKind::Deathtouch);
        match target {
            Entity::Player(p) => {
                if infect {
                    // CR 120.3b; the counters can be modified by replacement effects
                    // (CR 120.4c).
                    self.put_damage_counters(Entity::Player(p), counters::POISON, amount, source);
                } else {
                    // CR 120.3a (life loss can be replaced as "lose life")
                    for e in self.replace(ReplEvent::LoseLife {
                        player: p,
                        amount,
                        from_damage: true,
                    }) {
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
                        // CR 120.3d, 120.4c
                        self.put_damage_counters(
                            Entity::Object(o),
                            counters::MINUS1,
                            amount,
                            source,
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
                self.history.damage_by_source.insert((source, o));
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

    /// Counters that are the result of damage (infect and wither, CR 120.3b, 120.3d),
    /// as modified by replacement effects that interact with them (CR 120.4c).
    fn put_damage_counters(&mut self, target: Entity, kind: &str, n: u32, source: ObjectId) {
        for e in self.replace(ReplEvent::AddCounters {
            target,
            kind: kind.into(),
            n,
            source: Some(source),
        }) {
            self.execute_repl_event(e);
        }
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

    /// Creates `count` tokens that enter the battlefield attached to `to` (`None`: an
    /// undefined object or player). An Aura token that can't legally be attached to it
    /// isn't created; any other token enters unattached (CR 303.4g–i, 301.5e).
    pub fn create_tokens_attached(
        &mut self,
        controller: PlayerId,
        spec: TokenCreate,
        count: u32,
        source: Option<ObjectId>,
        to: Option<Entity>,
    ) -> Vec<ObjectId> {
        let prev = self.token_attach.replace(to);
        let out = self.create_tokens(controller, spec, count, source);
        self.token_attach = prev;
        out
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
            attach_to: self.token_attach.flatten(),
            attach_specified: self.token_attach.is_some(),
            counters: self.token_counters.clone(),
            ..Default::default()
        };
        if let Some(src) = spec.copy_of {
            match crate::copy_rules::double_faced_copy_face(self, src) {
                // CR 707.8a: a double-faced token, with the same face up; each face's
                // characteristics come from the same face of the card.
                Some(face) if spec.card.is_some() => {
                    etb.face = Some(face);
                    etb.copiable_mods = spec.copy_exceptions.clone();
                }
                _ => {
                    etb.copy_of = Some(src);
                    etb.copy_exceptions = spec.copy_exceptions.clone();
                }
            }
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
        self.attach_checked(obj, to, false)
    }

    /// Attaches `obj` to `to` as though `to` were a creature ("equip planeswalker",
    /// CR 702.6e). Returns false if it couldn't be attached.
    pub fn attach_as_creature(&mut self, obj: ObjectId, to: Entity) -> bool {
        self.attach_checked(obj, to, true)
    }

    fn attach_checked(&mut self, obj: ObjectId, to: Entity, as_creature: bool) -> bool {
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
        if !crate::attach::can_attach_as(self, obj, to, as_creature) {
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
        for m in mana {
            self.players[p.idx()].mana_pool.add(m);
        }
        self.emit(Event::ManaAdded { player: p, source });
        // CR 106.12a: "tapped for mana" is reported once the whole mana ability has
        // resolved, with all the mana it produced (see `Game::activate_ability`).
    }
}
