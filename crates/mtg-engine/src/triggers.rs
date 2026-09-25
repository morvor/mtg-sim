//! Triggered abilities (CR 603): detecting triggers from events, putting them on the
//! stack, state triggers, and delayed triggers.

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::events::LookbackSnapshot;
use crate::game::*;
use crate::object::*;
use crate::turn::Step;
use crate::types::*;
use std::collections::BTreeSet;
use std::sync::Arc;

/// Whether an ability is a keyword whose rules include a triggered ability.
pub fn keyword_has_trigger(a: &AbilityDef) -> bool {
    matches!(&a.kind, AbilityKind::Keyword(_))
}

/// Extra keys used in [`GameObject::triggers_this_turn`] besides plain ability uids
/// (ability uids are small counters, so the high bits are free).
pub mod turn_keys {
    /// Times an activated or triggered ability has resolved this turn (CR 603.7h).
    pub const RESOLVED: u64 = 1 << 63;
    /// A "Do this only once each turn" action was taken this turn (CR 603.2h).
    pub const DONE_ONCE: u64 = 1 << 62;
}

/// Whether a triggered ability with this trigger condition "looks back in time" for the
/// event: whether it triggers, and what the objects involved look like, is determined
/// from the game immediately before the event (CR 603.10).
pub fn looks_back(cond: &TriggerCond, ev: &Event) -> bool {
    match (cond, ev) {
        (TriggerCond::AnyOf(v), ev) => v.iter().any(|c| looks_back(c, ev)),
        // CR 603.10a: leaves-the-battlefield abilities.
        (
            TriggerCond::LeavesBattlefield(_) | TriggerCond::Dies(_),
            Event::ZoneChange {
                from: Zone::Battlefield,
                ..
            },
        ) => true,
        // CR 603.10a: leaving the battlefield or a graveyard, or an object all players
        // can see being put into a hand or library. "From anywhere" triggers are never
        // leaves-the-battlefield abilities (CR 603.6c).
        (TriggerCond::ZoneChange { from, to, .. }, Event::ZoneChange { from: zf, .. }) => {
            match from {
                Some(ZoneKind::Battlefield) | Some(ZoneKind::Graveyard) => true,
                _ => {
                    matches!(to, Some(ZoneKind::Hand) | Some(ZoneKind::Library))
                        && zf.is_public()
                        && *zf != Zone::Nowhere
                }
            }
        }
        // CR 603.10a: sacrificing a permanent.
        (TriggerCond::Sacrificed(_) | TriggerCond::YouSacrifice(_), Event::Sacrificed { .. }) => {
            true
        }
        // CR 603.10c: becoming unattached.
        (TriggerCond::BecomesUnattached(_), Event::Unattached { .. }) => true,
        // CR 603.10e: a spell being countered.
        (TriggerCond::SpellCountered(_), Event::Countered { .. }) => true,
        _ => false,
    }
}

impl Game {
    /// Processes pending events: records turn history and detects triggered abilities.
    pub fn flush_events(&mut self) {
        if self.events.is_empty() {
            return;
        }
        if self.dirty {
            self.recompute();
        }
        let events = std::mem::take(&mut self.events);
        // Snapshots taken just before zone changes, for events that follow them and look
        // back in time (sacrifices, countering, becoming unattached).
        let mut recent: Vec<(ObjectId, Arc<LookbackSnapshot>)> = Vec::new();
        let mut once_delayed: Vec<(u32, EventInfo)> = Vec::new();
        for ev in &events {
            self.record_history(ev);
            if let Event::ZoneChange {
                old,
                lookback: Some(lb),
                ..
            } = ev
            {
                recent.push((*old, lb.clone()));
            }
            once_delayed.extend(self.detect_triggers(ev, &recent));
        }
        self.fire_once_delayed(once_delayed);
        self.turn_events.extend(events);
        // Events emitted while detecting triggers (rare) are handled on the next flush.
    }

    fn record_history(&mut self, ev: &Event) {
        match ev {
            Event::SpellCast { spell, player, .. } => {
                self.history.spells_cast.push((*player, *spell))
            }
            Event::AttackersDeclared { attackers, .. } => {
                for (a, t) in attackers {
                    self.history.attackers.push(*a);
                    if let Entity::Player(p) = t {
                        self.history.players_attacked.insert(*p);
                    }
                }
            }
            Event::LandPlayed { player, .. } => {
                *self.history.lands_played.entry(*player).or_insert(0) += 1
            }
            Event::CrimeCommitted { player } => {
                *self.history.crimes.entry(*player).or_insert(0) += 1
            }
            _ => {}
        }
    }

    /// All (source, controller, ability) triples whose triggered abilities currently
    /// function, excluding look-back handling. Objects that are at no time visible to all
    /// players (cards in hands and libraries) don't trigger (CR 603.2f).
    pub(crate) fn current_trigger_sources(&self) -> Vec<(ObjectId, PlayerId, Ability)> {
        let mut out = Vec::new();
        for id in self.live_objects() {
            let o = self.obj(id);
            if matches!(o.zone, Zone::Hand(_) | Zone::Library(_)) {
                continue;
            }
            for a in &o.chars.abilities {
                let zone = match &a.kind {
                    AbilityKind::Triggered(t) => t.zone,
                    _ => continue,
                };
                if self.ability_functions(o, zone, false) {
                    out.push((id, o.controller, a.clone()));
                }
            }
        }
        out
    }

    /// Detects triggered abilities for one event (CR 603.2).
    pub fn check_triggers(&mut self, ev: &Event) {
        let once = self.detect_triggers(ev, &[]);
        self.fire_once_delayed(once);
    }

    /// Detects triggered abilities for one event, putting them in the pending list.
    /// Returns the matches of delayed triggers that trigger only once, which are resolved
    /// for a whole batch of simultaneous events by [`Game::fire_once_delayed`].
    fn detect_triggers(
        &mut self,
        ev: &Event,
        recent: &[(ObjectId, Arc<LookbackSnapshot>)],
    ) -> Vec<(u32, EventInfo)> {
        let lookback: Option<Arc<LookbackSnapshot>> = match ev {
            Event::ZoneChange {
                lookback: Some(lb), ..
            } => Some(lb.clone()),
            Event::Sacrificed { obj, .. }
            | Event::Countered { what: obj }
            | Event::Unattached { obj, .. } => recent
                .iter()
                .rev()
                .find(|(o, _)| o == obj)
                .map(|(_, lb)| lb.clone()),
            _ => None,
        };
        // CR 603.10: normally, abilities existing immediately after the event are
        // checked; abilities that look back use the existence and appearance of objects
        // immediately before it.
        let mut sources: Vec<(ObjectId, PlayerId, Ability)> = Vec::new();
        for (id, ctl, a) in self.current_trigger_sources() {
            let AbilityKind::Triggered(t) = &a.kind else {
                continue;
            };
            if lookback.is_some() && looks_back(&t.trigger, ev) {
                continue;
            }
            sources.push((id, ctl, a));
        }
        if let Some(lb) = &lookback {
            for (id, ctl, a) in &lb.sources {
                if let AbilityKind::Triggered(t) = &a.kind {
                    if looks_back(&t.trigger, ev) {
                        sources.push((*id, *ctl, a.clone()));
                    }
                }
            }
        }
        // CR 603.10b: abilities that trigger when a permanent phases out look back in
        // time, so the permanents that just phased out still have them.
        if let Event::PhasedOut { obj } = ev {
            let mut objs = vec![*obj];
            objs.extend(
                self.battlefield
                    .iter()
                    .copied()
                    .filter(|a| self.obj(*a).attached_to == Some(Entity::Object(*obj))),
            );
            for o in objs {
                let ob = self.obj(o);
                for a in &ob.chars.abilities {
                    if let AbilityKind::Triggered(t) = &a.kind {
                        if matches!(t.trigger, TriggerCond::PhasesOut(_))
                            && t.zone == FunctionZone::Battlefield
                            && !sources.iter().any(|(s, _, x)| *s == o && x.uid == a.uid)
                        {
                            sources.push((o, ob.controller, a.clone()));
                        }
                    }
                }
            }
        }
        // CR 603.10d: "when you lose control of" abilities look back: they're controlled by
        // the player who controlled the object before the change.
        if let Event::ControlChanged { obj, from, .. } = ev {
            for s in sources.iter_mut() {
                if s.0 == *obj {
                    if let AbilityKind::Triggered(t) = &s.2.kind {
                        if matches!(t.trigger, TriggerCond::LoseControl(_)) {
                            s.1 = *from;
                        }
                    }
                }
            }
        }
        let mut found: Vec<PendingTrigger> = Vec::new();
        for (src, ctl, a) in sources {
            let AbilityKind::Triggered(t) = &a.kind else {
                continue;
            };
            // Filters like "the chosen color" refer to the ability's linked choices.
            let mut base = Ctx::new(Some(src), ctl);
            base.link = a.link;
            for info in self.trigger_matches_ctx(&t.trigger, &base, ev) {
                let mut ctx = Ctx::new(Some(src), ctl);
                ctx.link = a.link;
                ctx.event = Some(info.clone());
                // CR 603.4: intervening "if" must be true when the event occurs.
                if let Some(c) = &t.intervening_if {
                    if !self.eval_cond(c, &ctx) {
                        continue;
                    }
                }
                let counted = |g: &Game, key: u64| {
                    g.obj(src)
                        .triggers_this_turn
                        .get(&key)
                        .copied()
                        .unwrap_or(0)
                        > 0
                };
                if t.once_per_turn && counted(self, a.uid) {
                    continue;
                }
                // CR 603.2h: "Do this only once each turn" — triggers only if the action
                // hasn't been taken this turn.
                if t.do_once_per_turn && counted(self, a.uid | turn_keys::DONE_ONCE) {
                    continue;
                }
                if t.once_per_turn {
                    *self.objects[src.0 as usize]
                        .triggers_this_turn
                        .entry(a.uid)
                        .or_insert(0) += 1;
                }
                // CR 603.2d: effects may make an ability trigger additional times.
                let times = 1 + self.additional_triggers(src, ev);
                for _ in 0..times {
                    self.trigger_order += 1;
                    found.push(PendingTrigger {
                        source: src,
                        controller: ctl,
                        ability: a.clone(),
                        event: info.clone(),
                        source_lki: Some(Box::new(self.obj(src).chars.clone())),
                        saved: None,
                        body: None,
                        order: self.trigger_order,
                    });
                }
            }
        }
        // Delayed triggers (CR 603.7).
        let turn = self.turn.number;
        // CR 603.7b: a delayed trigger that can trigger more than once has a stated
        // duration ("this turn"); it ends with the turn.
        self.delayed_triggers
            .retain(|d| d.once || d.created_turn == turn);
        let mut once_matches: Vec<(u32, EventInfo)> = Vec::new();
        for d in self.delayed_triggers.clone() {
            if let TriggerCond::BeginningOf { .. } = d.trigger {
                // "at the beginning of the next end step" doesn't fire in the step it was
                // created in (CR 513.2).
                // A cleanup step can be followed by another cleanup step in the same turn,
                // which is "the next cleanup step" (CR 514.3a).
                if let Event::StepBegan { step, .. } = ev {
                    if d.created_step == Some(*step)
                        && d.created_turn == self.turn.number
                        && *step != Step::Cleanup
                    {
                        continue;
                    }
                }
            }
            // CR 603.7a/c: the delayed trigger refers to the objects and choices of the
            // effect that created it.
            let mut base = d.ctx.clone();
            base.source = d.source;
            base.controller = d.controller;
            for info in self.trigger_matches_ctx(&d.trigger, &base, ev) {
                if d.once {
                    once_matches.push((d.id, info));
                } else {
                    self.trigger_order += 1;
                    found.push(self.delayed_pending(&d, info));
                }
            }
        }
        for t in found {
            // CR 605.1b / 605.4a: triggered mana abilities resolve immediately.
            if t.ability.is_mana_ability() {
                self.resolve_trigger_immediately(t);
            } else {
                self.pending_triggers.push(t);
            }
        }
        once_matches
    }

    fn delayed_pending(&self, d: &DelayedTrigger, info: EventInfo) -> PendingTrigger {
        let ability = AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(d.trigger.clone(), d.body.clone())),
            "delayed trigger",
        );
        PendingTrigger {
            source: d.source.unwrap_or(ObjectId(0)),
            controller: d.controller,
            ability,
            event: info,
            source_lki: None,
            saved: Some(d.ctx.clone()),
            body: Some(d.body.clone()),
            order: self.trigger_order,
        }
    }

    /// Delayed triggered abilities without a stated duration trigger only once, the next
    /// time their trigger event occurs. If it occurs more than once simultaneously, the
    /// delayed trigger's controller chooses which event causes it to trigger (CR 603.7b).
    fn fire_once_delayed(&mut self, matches: Vec<(u32, EventInfo)>) {
        let mut ids: Vec<u32> = Vec::new();
        for (id, _) in &matches {
            if !ids.contains(id) {
                ids.push(*id);
            }
        }
        for id in ids {
            let Some(d) = self.delayed_triggers.iter().find(|d| d.id == id).cloned() else {
                continue;
            };
            let infos: Vec<EventInfo> = matches
                .iter()
                .filter(|(i, _)| *i == id)
                .map(|(_, e)| e.clone())
                .collect();
            let pick = if infos.len() > 1 {
                let labels = infos
                    .iter()
                    .map(|e| match (e.object, e.player) {
                        (Some(o), _) => self.describe(o),
                        (None, Some(p)) => format!("{p}"),
                        _ => "event".to_string(),
                    })
                    .collect();
                self.ask_option(
                    d.controller,
                    d.source,
                    "Choose the event that causes the delayed trigger",
                    labels,
                )
            } else {
                0
            };
            self.delayed_triggers.retain(|x| x.id != id);
            self.trigger_order += 1;
            let t = self.delayed_pending(&d, infos[pick.min(infos.len() - 1)].clone());
            self.pending_triggers.push(t);
        }
    }

    /// How many additional times an ability of `src` triggers because of effects such as
    /// "that ability triggers an additional time" (CR 603.2d). Each such effect adds one;
    /// they don't apply to delayed or reflexive triggered abilities.
    fn additional_triggers(&self, src: ObjectId, ev: &Event) -> usize {
        self.statics
            .other
            .iter()
            .filter(|(s, c, e)| match e {
                StaticEffect::AdditionalTrigger { sources, cause } => {
                    let ctx = Ctx::new(Some(*s), *c);
                    self.matches(src, sources, &ctx)
                        && cause
                            .as_ref()
                            .is_none_or(|cond| !self.trigger_matches(cond, *s, *c, ev).is_empty())
                }
                _ => false,
            })
            .count()
    }

    /// Checks one trigger condition against an event. Returns one [`EventInfo`] per time
    /// the ability triggers (CR 603.2c).
    pub fn trigger_matches(
        &self,
        cond: &TriggerCond,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Vec<EventInfo> {
        self.trigger_matches_ctx(cond, &Ctx::new(Some(src), ctl), ev)
    }

    /// As [`Game::trigger_matches`], with a full context (delayed triggers refer to the
    /// targets and variables of the effect that created them).
    pub fn trigger_matches_ctx(
        &self,
        cond: &TriggerCond,
        base: &Ctx,
        ev: &Event,
    ) -> Vec<EventInfo> {
        let src = base.source.unwrap_or(ObjectId(0));
        let ctl = base.controller;
        let ctx = base.clone();
        let one = |info: EventInfo| vec![info];
        let none = Vec::new;
        match (cond, ev) {
            (
                TriggerCond::EntersBattlefield(f),
                Event::ZoneChange {
                    new,
                    to: Zone::Battlefield,
                    ..
                },
            ) => {
                if self.matches(*new, f, &ctx) {
                    one(EventInfo {
                        object: Some(*new),
                        player: Some(self.obj(*new).controller),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::LeavesBattlefield(f),
                Event::ZoneChange {
                    old,
                    new,
                    from: Zone::Battlefield,
                    to,
                    ..
                },
            ) if *to != Zone::Battlefield => {
                if self.matches(*old, f, &ctx) {
                    one(EventInfo {
                        object: Some(*new),
                        lki: Some(*old),
                        player: Some(self.obj(*old).controller),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::Dies(f),
                Event::ZoneChange {
                    old,
                    new,
                    from: Zone::Battlefield,
                    to: Zone::Graveyard(_),
                    ..
                },
            ) => {
                // CR 700.4: "dies" = put into a graveyard from the battlefield.
                if self.matches(*old, f, &ctx) {
                    one(EventInfo {
                        object: Some(*new),
                        lki: Some(*old),
                        player: Some(self.obj(*old).controller),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::ZoneChange { filter, from, to },
                Event::ZoneChange {
                    old,
                    new,
                    from: zf,
                    to: zt,
                    ..
                },
            ) => {
                let fm = from.is_none_or(|z| zf.kind() == Some(z));
                let tm = to.is_none_or(|z| zt.kind() == Some(z));
                let check = if *zf == Zone::Battlefield { *old } else { *new };
                if fm && tm && self.matches(check, filter, &ctx) {
                    one(EventInfo {
                        object: Some(*new),
                        lki: Some(*old),
                        player: Some(self.obj(*old).owner),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::CastSpell { who, filter }, Event::SpellCast { spell, player, .. }) => {
                if self.player_rel_matches(*who, *player, &ctx)
                    && self.matches(*spell, filter, &ctx)
                {
                    one(EventInfo {
                        object: Some(*spell),
                        spell: Some(*spell),
                        player: Some(*player),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::NthSpellCast { who, n }, Event::SpellCast { spell, player, .. }) => {
                let count = self
                    .history
                    .spells_cast
                    .iter()
                    .filter(|(p, _)| p == player)
                    .count() as u32;
                if self.player_rel_matches(*who, *player, &ctx) && count == *n {
                    one(EventInfo {
                        object: Some(*spell),
                        spell: Some(*spell),
                        player: Some(*player),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::AbilityActivated {
                    who,
                    source,
                    include_mana,
                },
                Event::AbilityActivated {
                    ability,
                    source: s,
                    player,
                    is_mana,
                },
            ) => {
                if (!is_mana || *include_mana)
                    && self.player_rel_matches(*who, *player, &ctx)
                    && self.matches(*s, source, &ctx)
                {
                    one(EventInfo {
                        object: Some(*s),
                        spell: *ability,
                        player: Some(*player),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Attacks(f), Event::AttackersDeclared { attackers, player }) => attackers
                .iter()
                .filter(|(a, _)| self.matches(*a, f, &ctx))
                .map(|(a, t)| EventInfo {
                    object: Some(*a),
                    player: match t {
                        Entity::Player(p) => Some(*p),
                        Entity::Object(o) => Some(self.obj(*o).controller),
                    },
                    other: t.object(),
                    objects: attackers.iter().map(|x| x.0).collect(),
                    amount: attackers.len() as i32,
                    ..Default::default()
                })
                .map(|mut e| {
                    if e.player.is_none() {
                        e.player = Some(*player);
                    }
                    e
                })
                .collect(),
            (TriggerCond::PlayerAttacks(rel), Event::AttackersDeclared { attackers, player }) => {
                if !attackers.is_empty() && self.player_rel_matches(*rel, *player, &ctx) {
                    one(EventInfo {
                        player: Some(*player),
                        objects: attackers.iter().map(|x| x.0).collect(),
                        amount: attackers.len() as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::AttacksUnblocked(f), Event::AttackerUnblocked { attacker }) => {
                if self.matches(*attacker, f, &ctx) {
                    one(EventInfo {
                        object: Some(*attacker),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Blocks(f), Event::BlockersDeclared { blocks }) => {
                let mut seen = BTreeSet::new();
                blocks
                    .iter()
                    .filter(|(b, _)| self.matches(*b, f, &ctx) && seen.insert(*b))
                    .map(|(b, a)| EventInfo {
                        object: Some(*b),
                        other: Some(*a),
                        ..Default::default()
                    })
                    .collect()
            }
            (TriggerCond::BecomesBlocked(f), Event::BecameBlocked { attacker, blockers }) => {
                if self.matches(*attacker, f, &ctx) {
                    one(EventInfo {
                        object: Some(*attacker),
                        other: blockers.first().copied(),
                        objects: blockers.clone(),
                        amount: blockers.len() as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::BlocksOrBecomesBlocked(f), Event::BlockersDeclared { blocks }) => {
                let mut out = Vec::new();
                let mut seen = BTreeSet::new();
                for (b, a) in blocks {
                    if self.matches(*b, f, &ctx) && seen.insert(*b) {
                        out.push(EventInfo {
                            object: Some(*b),
                            other: Some(*a),
                            ..Default::default()
                        });
                    }
                }
                let mut seen_a = BTreeSet::new();
                for (b, a) in blocks {
                    if self.matches(*a, f, &ctx) && seen_a.insert(*a) && !seen.contains(a) {
                        out.push(EventInfo {
                            object: Some(*a),
                            other: Some(*b),
                            ..Default::default()
                        });
                    }
                }
                out
            }
            (
                TriggerCond::DealsDamage {
                    source,
                    to,
                    combat_only,
                },
                Event::Damage {
                    source: s,
                    target,
                    amount,
                    combat,
                },
            ) => {
                if (*combat_only && !combat) || !self.matches(*s, source, &ctx) {
                    return none();
                }
                let ok = match (to, target) {
                    (DamageRecipient::Any, _) => true,
                    (DamageRecipient::Player(rel), Entity::Player(p)) => {
                        self.player_rel_matches(*rel, *p, &ctx)
                    }
                    (DamageRecipient::Object(f), Entity::Object(o)) => self.matches(*o, f, &ctx),
                    (DamageRecipient::PlayerOrPlaneswalker(rel), Entity::Player(p)) => {
                        self.player_rel_matches(*rel, *p, &ctx)
                    }
                    (DamageRecipient::PlayerOrPlaneswalker(rel), Entity::Object(o)) => {
                        self.obj(*o).is(CardType::Planeswalker)
                            && self.player_rel_matches(*rel, self.obj(*o).controller, &ctx)
                    }
                    _ => false,
                };
                if ok {
                    one(EventInfo {
                        object: target.object(),
                        other: Some(*s),
                        player: target
                            .player()
                            .or_else(|| target.object().map(|o| self.obj(o).controller)),
                        amount: *amount as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::IsDealtDamage {
                    filter,
                    combat_only,
                },
                Event::Damage {
                    source,
                    target: Entity::Object(o),
                    amount,
                    combat,
                },
            ) => {
                if (!*combat_only || *combat) && self.matches(*o, filter, &ctx) {
                    one(EventInfo {
                        object: Some(*o),
                        other: Some(*source),
                        amount: *amount as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::PlayerDealtDamage { who, combat_only },
                Event::Damage {
                    source,
                    target: Entity::Player(p),
                    amount,
                    combat,
                },
            ) => {
                if (!*combat_only || *combat) && self.player_rel_matches(*who, *p, &ctx) {
                    one(EventInfo {
                        player: Some(*p),
                        other: Some(*source),
                        amount: *amount as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::BeginningOf { step, whose }, Event::StepBegan { step: s, active }) => {
                let matches_step = s.trigger_step() == *step
                    // "At the beginning of combat damage step" triggers for each damage step.
                    && !(*s == Step::FirstStrikeDamage && *step != TriggerStep::CombatDamage);
                // "At the beginning of your precombat main phase" only for the first main phase.
                let main_ok = !(*step == TriggerStep::PrecombatMain && self.turn.main_phases > 1);
                if matches_step && main_ok && self.player_rel_matches(*whose, *active, &ctx) {
                    one(EventInfo {
                        player: Some(*active),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::BeginningOf {
                    step: TriggerStep::Turn,
                    whose,
                },
                Event::TurnBegan { active, .. },
            ) => {
                if self.player_rel_matches(*whose, *active, &ctx) {
                    one(EventInfo {
                        player: Some(*active),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Draws { who }, Event::Drew { player, card, nth }) => {
                if self.player_rel_matches(*who, *player, &ctx) {
                    one(EventInfo {
                        player: Some(*player),
                        object: Some(*card),
                        amount: *nth as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Discards { who, filter }, Event::Discarded { player, card }) => {
                if self.player_rel_matches(*who, *player, &ctx) && self.matches(*card, filter, &ctx)
                {
                    one(EventInfo {
                        player: Some(*player),
                        object: Some(*card),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::GainsLife { who }, Event::LifeGained { player, amount }) => {
                if self.player_rel_matches(*who, *player, &ctx) {
                    one(EventInfo {
                        player: Some(*player),
                        amount: *amount as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::LosesLife { who }, Event::LifeLost { player, amount }) => {
                if self.player_rel_matches(*who, *player, &ctx) {
                    one(EventInfo {
                        player: Some(*player),
                        amount: *amount as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::CountersPut { filter, kind },
                Event::CountersAdded {
                    target: Entity::Object(o),
                    kind: k,
                    n,
                },
            ) => {
                if kind.as_ref().is_none_or(|x| x == k) && self.matches(*o, filter, &ctx) {
                    one(EventInfo {
                        object: Some(*o),
                        amount: *n as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::CountersRemoved { filter, kind },
                Event::CountersRemoved {
                    target: Entity::Object(o),
                    kind: k,
                    n,
                },
            ) => {
                if kind.as_ref().is_none_or(|x| x == k) && self.matches(*o, filter, &ctx) {
                    one(EventInfo {
                        object: Some(*o),
                        amount: *n as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::BecomesTapped(f), Event::Tapped { obj, .. }) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::BecomesUntapped(f), Event::Untapped { obj }) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::BecomesTarget { filter, by },
                Event::BecameTarget {
                    target: Entity::Object(o),
                    by: s,
                    controller,
                },
            ) => {
                if self.matches(*o, filter, &ctx) && self.player_rel_matches(*by, *controller, &ctx)
                {
                    one(EventInfo {
                        object: Some(*o),
                        spell: Some(*s),
                        player: Some(*controller),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Sacrificed(f), Event::Sacrificed { obj, player }) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        lki: Some(*obj),
                        player: Some(*player),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::YouSacrifice(f), Event::Sacrificed { obj, player }) => {
                if *player == ctl && self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        lki: Some(*obj),
                        player: Some(*player),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::TokenCreated(f), Event::TokenCreated { obj, controller }) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        player: Some(*controller),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::LandPlayed { who, filter }, Event::LandPlayed { player, land }) => {
                if self.player_rel_matches(*who, *player, &ctx) && self.matches(*land, filter, &ctx)
                {
                    one(EventInfo {
                        object: Some(*land),
                        player: Some(*player),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Cycled { who, filter }, Event::Cycled { player, card }) => {
                if self.player_rel_matches(*who, *player, &ctx) && self.matches(*card, filter, &ctx)
                {
                    one(EventInfo {
                        object: Some(*card),
                        player: Some(*player),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Searched(rel), Event::Searched { player }) => {
                if self.player_rel_matches(*rel, *player, &ctx) {
                    one(EventInfo {
                        player: Some(*player),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::TurnedFaceUp(f), Event::TurnedFaceUp { obj }) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Transforms(f), Event::Transformed { obj }) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::ControlChanged(f), Event::ControlChanged { obj, to, .. }) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        player: Some(*to),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::PlayerLoses, Event::PlayerLost { player }) => one(EventInfo {
                player: Some(*player),
                ..Default::default()
            }),
            (TriggerCond::RollDie(rel), Event::DieRolled { player, result, .. }) => {
                if self.player_rel_matches(*rel, *player, &ctx) {
                    one(EventInfo {
                        player: Some(*player),
                        amount: *result as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::FlipCoin(rel), Event::CoinFlipped { player, won }) => {
                if self.player_rel_matches(*rel, *player, &ctx) {
                    one(EventInfo {
                        player: Some(*player),
                        amount: *won as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Mills(rel), Event::Milled { player, cards }) => {
                if self.player_rel_matches(*rel, *player, &ctx) {
                    one(EventInfo {
                        player: Some(*player),
                        objects: cards.clone(),
                        amount: cards.len() as i32,
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::CommitCrime(rel), Event::CrimeCommitted { player }) => {
                if self.player_rel_matches(*rel, *player, &ctx) {
                    one(EventInfo {
                        player: Some(*player),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::DayNightChanges, Event::DayNightChanged { .. }) => {
                one(EventInfo::default())
            }
            (TriggerCond::AnyOf(conds), ev) => conds
                .iter()
                .flat_map(|c| self.trigger_matches_ctx(c, base, ev))
                .collect(),
            (TriggerCond::PhasesOut(f), Event::PhasedOut { obj }) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        player: Some(self.obj(*obj).controller),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::BecomesUnattached(f), Event::Unattached { obj, from }) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: from.object(),
                        other: Some(*obj),
                        player: from.player(),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::LoseControl(f), Event::ControlChanged { obj, from, to }) => {
                if *from == ctl && self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        player: Some(*to),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::AbilityResolved {
                    source: f,
                    final_chapter,
                },
                Event::AbilityResolved {
                    ability, source, ..
                },
            ) => {
                let chapter_ok = !*final_chapter || {
                    let final_n = crate::saga::final_chapter(self.obj(*source));
                    match self.obj(*ability).stack.as_deref().map(|s| &s.kind) {
                        Some(StackKind::Triggered { ability: a, .. }) => match &a.kind {
                            AbilityKind::Triggered(t) => match &t.trigger {
                                TriggerCond::Custom(n) => {
                                    n.strip_prefix("chapter:").is_some_and(|ns| {
                                        ns.split(',')
                                            .any(|x| x.trim().parse::<u32>().ok() == final_n)
                                    })
                                }
                                _ => false,
                            },
                            _ => false,
                        },
                        _ => false,
                    }
                };
                if chapter_ok && self.matches(*source, f, &ctx) {
                    one(EventInfo {
                        object: Some(self.current(*source)),
                        lki: Some(*source),
                        other: Some(*ability),
                        player: Some(self.obj(*ability).controller),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::TappedForMana(f),
                Event::TappedForMana {
                    obj,
                    player,
                    produced,
                },
            ) => {
                if self.matches(*obj, f, &ctx) {
                    one(EventInfo {
                        object: Some(*obj),
                        player: Some(*player),
                        amount: crate::mana::mask_of_types(produced),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::SpellCountered(f), Event::Countered { what }) => {
                if self.obj(*what).kind != ObjKind::StackAbility && self.matches(*what, f, &ctx) {
                    one(EventInfo {
                        object: Some(self.current(*what)),
                        lki: Some(*what),
                        spell: Some(*what),
                        player: Some(self.obj(*what).controller),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (
                TriggerCond::AbilityTriggered { cause, source },
                Event::AbilityTriggeredOnStack { ability, source: s },
            ) => {
                // CR 603.3b: an ability that triggers on another ability triggering. The
                // other ability's trigger event must be of the stated kind.
                let Some(si) = self.obj(*ability).stack.as_deref() else {
                    return none();
                };
                let StackKind::Triggered { ability: a, .. } = &si.kind else {
                    return none();
                };
                let AbilityKind::Triggered(t) = &a.kind else {
                    return none();
                };
                let info = si.event.clone().unwrap_or_default();
                let about = info.lki.or(info.object);
                let cause_ok = std::mem::discriminant(&**cause)
                    == std::mem::discriminant(&t.trigger)
                    && match &**cause {
                        TriggerCond::EntersBattlefield(f)
                        | TriggerCond::LeavesBattlefield(f)
                        | TriggerCond::Dies(f)
                        | TriggerCond::Attacks(f) => {
                            about.is_some_and(|o| self.matches(o, f, &ctx))
                        }
                        _ => true,
                    };
                if cause_ok && self.matches(*s, source, &ctx) {
                    one(EventInfo {
                        object: info.object,
                        spell: Some(*ability),
                        player: Some(self.obj(*ability).controller),
                        ..Default::default()
                    })
                } else {
                    none()
                }
            }
            (TriggerCond::Custom(name), ev) => {
                crate::custom::custom_trigger(self, name, src, ctl, ev)
            }
            // Combat trigger conditions (CR 506.5–6, 508.3, 509.3) and blocks added by
            // effects (CR 509.3, 509.4) live in combat.rs.
            (cond, ev) => {
                crate::combat::combat_trigger_matches(self, cond, &ctx, ev).unwrap_or_default()
            }
        }
    }

    /// Checks state triggers (CR 603.8).
    pub fn check_state_triggers(&mut self) {
        if self.dirty {
            self.recompute();
        }
        let mut found = Vec::new();
        for (src, ctl, a) in self.current_trigger_sources() {
            let AbilityKind::Triggered(t) = &a.kind else {
                continue;
            };
            let TriggerCond::State(c) = &t.trigger else {
                continue;
            };
            // Doesn't trigger again until it has resolved / left the stack.
            let key = (src, a.uid);
            if self.state_triggers_active.contains(&key) {
                continue;
            }
            let ctx = Ctx::new(Some(src), ctl);
            if self.eval_cond(c, &ctx) {
                self.state_triggers_active.insert(key);
                self.trigger_order += 1;
                found.push(PendingTrigger {
                    source: src,
                    controller: ctl,
                    ability: a.clone(),
                    event: EventInfo::default(),
                    source_lki: Some(Box::new(self.obj(src).chars.clone())),
                    saved: None,
                    body: None,
                    order: self.trigger_order,
                });
            }
        }
        self.pending_triggers.extend(found);
    }

    /// Puts all waiting triggered abilities on the stack in APNAP order, each player
    /// ordering their own (CR 603.3b).
    pub fn put_triggers_on_stack(&mut self) {
        let pending = std::mem::take(&mut self.pending_triggers);
        if pending.is_empty() {
            return;
        }
        for p in self.apnap() {
            let mut mine: Vec<PendingTrigger> = pending
                .iter()
                .filter(|t| t.controller == p)
                .cloned()
                .collect();
            if mine.is_empty() {
                continue;
            }
            mine.sort_by_key(|t| t.order);
            let order = if mine.len() > 1 {
                let names: Vec<String> = mine
                    .iter()
                    .map(|t| format!("{}: {}", self.obj(t.source).chars.name, t.ability.text))
                    .collect();
                self.ask_order(
                    p,
                    "Order triggered abilities (first = bottom of stack)",
                    names,
                )
            } else {
                vec![0]
            };
            for i in order {
                self.put_trigger_on_stack(mine[i].clone());
            }
        }
        // Triggers controlled by players who left the game are removed (CR 800.4a).
    }

    fn put_trigger_on_stack(&mut self, t: PendingTrigger) {
        let body = t.body.clone().unwrap_or_else(|| match &t.ability.kind {
            AbilityKind::Triggered(tr) => tr.body.clone(),
            _ => Body::default(),
        });
        let mut ctx = t
            .saved
            .clone()
            .unwrap_or_else(|| Ctx::new(Some(t.source), t.controller));
        ctx.controller = t.controller;
        ctx.event = Some(t.event.clone());
        ctx.ability_uid = t.ability.uid;
        ctx.link = t.ability.link;
        let id = crate::stack::create_stack_ability(
            self,
            t.source,
            t.controller,
            t.ability.clone(),
            StackKind::Triggered {
                source: t.source,
                ability: t.ability.clone(),
            },
            Some(t.event.clone()),
            t.source_lki.clone(),
        );
        // CR 603.3c–d: choose modes and targets; if targets can't be chosen, remove it.
        if !self.choose_modes_and_targets(id, &body, &mut ctx) {
            self.stack.retain(|x| *x != id);
            self.objects[id.0 as usize].zone = Zone::Nowhere;
            self.state_triggers_active
                .remove(&(t.source, t.ability.uid));
            return;
        }
        if let Some(saved) = t.saved {
            if let Some(si) = self.objects[id.0 as usize].stack.as_mut() {
                si.x = Some(saved.x);
            }
            self.saved_ctx.insert(id, saved);
        }
        self.emit(Event::AbilityTriggeredOnStack {
            ability: id,
            source: t.source,
        });
        self.flush_events();
    }

    fn resolve_trigger_immediately(&mut self, t: PendingTrigger) {
        let body = match &t.ability.kind {
            AbilityKind::Triggered(tr) => tr.body.clone(),
            _ => return,
        };
        let mut ctx = Ctx::new(Some(t.source), t.controller);
        ctx.event = Some(t.event.clone());
        self.exec(&body.effect, &mut ctx);
    }
}
