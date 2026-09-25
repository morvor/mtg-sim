//! Triggered abilities (CR 603): detecting triggers from events, putting them on the
//! stack, state triggers, and delayed triggers.

use crate::ability::*;
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::*;
use crate::object::*;
use crate::turn::Step;
use crate::types::*;
use std::collections::BTreeSet;

/// Whether an ability is a keyword whose rules include a triggered ability.
pub fn keyword_has_trigger(a: &AbilityDef) -> bool {
    matches!(&a.kind, AbilityKind::Keyword(_))
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
        for ev in &events {
            self.record_history(ev);
            self.check_triggers(ev);
        }
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
    /// function, excluding look-back handling.
    fn current_trigger_sources(&self) -> Vec<(ObjectId, PlayerId, Ability)> {
        let mut out = Vec::new();
        for id in self.live_objects() {
            let o = self.obj(id);
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
        let lookback = match ev {
            Event::ZoneChange {
                from: Zone::Battlefield,
                lookback: Some(lb),
                ..
            } => Some(lb.clone()),
            _ => None,
        };
        let mut sources = self.current_trigger_sources();
        if let Some(lb) = &lookback {
            // CR 603.10a: leaves-the-battlefield abilities look back in time. Use the
            // snapshot for permanents; keep current sources from other zones.
            sources.retain(|(id, _, _)| self.obj(*id).zone != Zone::Battlefield);
            for (id, ctl, a) in &lb.sources {
                if matches!(a.kind, AbilityKind::Triggered(_)) {
                    sources.push((*id, *ctl, a.clone()));
                }
            }
        }
        let mut found: Vec<PendingTrigger> = Vec::new();
        for (src, ctl, a) in sources {
            let AbilityKind::Triggered(t) = &a.kind else {
                continue;
            };
            for info in self.trigger_matches(&t.trigger, src, ctl, ev) {
                let mut ctx = Ctx::new(Some(src), ctl);
                ctx.event = Some(info.clone());
                // CR 603.4: intervening "if" must be true when the event occurs.
                if let Some(c) = &t.intervening_if {
                    if !self.eval_cond(c, &ctx) {
                        continue;
                    }
                }
                if t.once_per_turn
                    && self
                        .obj(src)
                        .triggers_this_turn
                        .get(&a.uid)
                        .copied()
                        .unwrap_or(0)
                        > 0
                {
                    continue;
                }
                if t.once_per_turn {
                    *self.objects[src.0 as usize]
                        .triggers_this_turn
                        .entry(a.uid)
                        .or_insert(0) += 1;
                }
                self.trigger_order += 1;
                found.push(PendingTrigger {
                    source: src,
                    controller: ctl,
                    ability: a.clone(),
                    event: info,
                    source_lki: Some(Box::new(self.obj(src).chars.clone())),
                    saved: None,
                    body: None,
                    order: self.trigger_order,
                });
            }
        }
        // Delayed triggers (CR 603.7).
        let mut fired: Vec<u32> = Vec::new();
        for d in self.delayed_triggers.clone() {
            if let TriggerCond::BeginningOf { .. } = d.trigger {
                // "at the beginning of the next end step" doesn't fire in the step it was
                // created in (CR 513.2).
                if let Event::StepBegan { step, .. } = ev {
                    if d.created_step == Some(*step) && d.created_turn == self.turn.number {
                        continue;
                    }
                }
            }
            let src = d.source.unwrap_or(ObjectId(0));
            for info in self.trigger_matches(&d.trigger, src, d.controller, ev) {
                self.trigger_order += 1;
                let ability = AbilityDef::new(
                    AbilityKind::Triggered(TriggeredAbility::new(
                        d.trigger.clone(),
                        d.body.clone(),
                    )),
                    "delayed trigger",
                );
                found.push(PendingTrigger {
                    source: src,
                    controller: d.controller,
                    ability,
                    event: info,
                    source_lki: None,
                    saved: Some(d.ctx.clone()),
                    body: Some(d.body.clone()),
                    order: self.trigger_order,
                });
                if d.once {
                    fired.push(d.id);
                    break;
                }
            }
        }
        if !fired.is_empty() {
            self.delayed_triggers.retain(|d| !fired.contains(&d.id));
        }
        for t in found {
            // CR 605.1b / 605.3: triggered mana abilities resolve immediately.
            if t.ability.is_mana_ability() {
                self.resolve_trigger_immediately(t);
            } else {
                self.pending_triggers.push(t);
            }
        }
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
        let ctx = Ctx::new(Some(src), ctl);
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
            (
                TriggerCond::TappedForManaOfType { filter, mana },
                Event::TappedForMana {
                    obj,
                    player,
                    produced,
                },
            ) => {
                // CR 106.12a: "tapped for mana of a specified type" triggers only if that
                // type of mana was produced.
                if produced.contains(mana) && self.matches(*obj, filter, &ctx) {
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
            (TriggerCond::Custom(name), ev) => {
                crate::custom::custom_trigger(self, name, src, ctl, ev)
            }
            _ => none(),
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
