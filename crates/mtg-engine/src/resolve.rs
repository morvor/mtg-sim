//! The effect interpreter: performs [`Effect`]s as spells and abilities resolve
//! (CR 608.2c–h, 609, 610, 611).

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::mana::ManaType;
use crate::object::*;
use crate::replacement::*;
use crate::types::*;

/// Resolution context saved with delayed triggers / stack objects.
pub type SavedCtx = Ctx;

impl Game {
    /// Executes an effect.
    pub fn exec(&mut self, e: &Effect, ctx: &mut Ctx) {
        if self.result.is_some() || self.end.restart.is_some() {
            return;
        }
        if self.dirty {
            self.recompute();
        }
        // Each instruction is a separate action: events it causes form their own batch for
        // "one or more" triggers (CR 603.2c, 608.2c).
        self.end_event_batch();
        if ctx.entering.is_some() && self.effect_on_entering_object(e, ctx) {
            return;
        }
        match e {
            Effect::Noop => {}
            Effect::Seq(v) => {
                for (i, x) in v.iter().enumerate() {
                    self.exec(x, ctx);
                    // CR 727.4: the rest of an effect that restarted the game happens as the
                    // new game begins.
                    if let Some(r) = self.end.restart.as_mut() {
                        r.then.extend(v[i + 1..].iter().cloned());
                        break;
                    }
                    // CR 603.8: state triggers trigger as soon as the game state matches,
                    // even momentarily during a resolution.
                    self.check_state_triggers();
                }
            }
            Effect::If {
                cond,
                then,
                otherwise,
            } => {
                if self.eval_cond(cond, ctx) {
                    self.exec(then, ctx);
                } else {
                    self.exec(otherwise, ctx);
                }
            }
            Effect::May { who, effect } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let text = format!("{effect:?}");
                // CR 121.2b, 121.3: a player can't choose to draw more cards than they may.
                let yes = crate::draw_rules::can_choose(self, effect, ctx)
                    && self.ask_yes_no(
                        p,
                        ctx.source,
                        &format!("You may: {}", truncate(&text, 120)),
                        true,
                    );
                ctx.prev_happened = yes;
                if yes {
                    // Effects that can fail to do what they say (sacrificing, paying,
                    // countering) record whether they did it ("if you do", "when you do").
                    self.exec(effect, ctx);
                }
            }
            Effect::PayOptional {
                who,
                cost,
                then,
                otherwise,
            } => {
                let cost = &crate::mana_abilities::bind_x_for_payment(self, cost, ctx);
                // "[cost] for each ...": the total is determined now (CR 702.24a).
                let cost = &crate::kw::cumulative_upkeep::expand_repeated(self, cost, ctx);
                let players = self.eval_players(who, ctx);
                let mut paid = false;
                for p in players {
                    if self.can_pay_cost(p, cost, ctx.source, ctx)
                        && self.ask_yes_no(
                            p,
                            ctx.source,
                            &format!("Pay {}?", describe_cost(cost)),
                            false,
                        )
                        && self.pay_cost(p, cost, ctx.source, ctx)
                    {
                        paid = true;
                        break;
                    }
                }
                ctx.prev_happened = paid;
                if paid {
                    self.exec(then, ctx);
                } else {
                    self.exec(otherwise, ctx);
                }
                ctx.prev_happened = paid;
            }
            Effect::ForEach { sel, var, effect } => {
                let items = self.resolve_sel(sel, ctx);
                for it in items {
                    let saved = ctx.vars.insert(*var, vec![it]);
                    self.exec(effect, ctx);
                    match saved {
                        Some(s) => {
                            ctx.vars.insert(*var, s);
                        }
                        None => {
                            ctx.vars.remove(var);
                        }
                    }
                }
            }
            Effect::ForEachPlayer { who, effect } => {
                let players = self.eval_players(who, ctx);
                let saved = ctx.iter_player;
                for p in players {
                    ctx.iter_player = Some(p);
                    self.exec(effect, ctx);
                }
                ctx.iter_player = saved;
            }
            Effect::Repeat { times, effect } => {
                let n = self.eval_value(times, ctx).max(0);
                for _ in 0..n {
                    self.exec(effect, ctx);
                }
            }
            Effect::ChooseOne { who, options } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let labels = options.iter().map(|(l, _)| l.clone()).collect();
                let i = self.ask_option(p, ctx.source, "Choose one", labels);
                if let Some((_, eff)) = options.get(i) {
                    self.exec(eff, ctx);
                }
            }
            Effect::Store { var, sel } => {
                let v = self.resolve_sel(sel, ctx);
                ctx.vars.insert(*var, v);
            }
            Effect::StoreValue { var, value } => {
                let n = self.eval_value(value, ctx);
                ctx.nums.insert(*var, n);
            }
            Effect::Note { value } => {
                let n = self.eval_value(value, ctx) as i32;
                if let Some(src) = ctx.source {
                    self.objects[src.0 as usize]
                        .linked_choices
                        .entry(ctx.link)
                        .or_default()
                        .number = Some(n);
                }
            }
            Effect::SetX { value } => {
                ctx.x = self.eval_value(value, ctx) as i32;
                ctx.x_defined = true;
            }

            // --- Objects -------------------------------------------------------
            Effect::Destroy { what, no_regen } => {
                let objs = self.resolve_objects(what, ctx);
                let res = self.destroy_all(objs.clone(), ctx.source, *no_regen);
                ctx.prev_affected = res.iter().map(|o| Entity::Object(*o)).collect();
                ctx.set_var(vars::IT, res.into_iter().map(Entity::Object).collect());
            }
            Effect::Exile {
                what,
                face_down,
                link,
            } => {
                let objs = self.resolve_objects(what, ctx);
                let prev_link = self.current_link;
                self.current_link = ctx.link;
                let moves: Vec<MoveEv> = objs
                    .iter()
                    .filter(|o| self.is_live(**o))
                    .map(|o| MoveEv {
                        obj: *o,
                        to: Zone::Exile,
                        pos: LibraryPosition::Top,
                        cause: MoveCause::Exile,
                        by: Some(ctx.controller),
                        etb: EtbInfo {
                            face_down: if *face_down {
                                Some(KeywordKind::Morph)
                            } else {
                                None
                            },
                            ..Default::default()
                        },
                        // CR 607.2a: cards an ability exiles are exiled with its source,
                        // for the abilities linked to it.
                        source: ctx.source,
                    })
                    .collect();
                let _ = link;
                let res: Vec<ObjectId> = self.move_objects(moves).into_iter().flatten().collect();
                // CR 712.21c, 730.3c: a melded or merged permanent became several cards.
                let res = crate::merge::found_all(self, res);
                self.current_link = prev_link;
                ctx.prev_affected = res.iter().map(|o| Entity::Object(*o)).collect();
                ctx.set_var(vars::IT, res.into_iter().map(Entity::Object).collect());
            }
            Effect::Sacrifice { who, filter, count } => {
                let players = self.eval_players(who, ctx);
                let n = self.eval_value(count, ctx).max(0) as u32;
                let mut all = Vec::new();
                // CR 101.4 / 608.2e: choices in APNAP order, then performed simultaneously.
                let mut chosen: Vec<(PlayerId, ObjectId)> = Vec::new();
                let round = self.apnap_choices.len();
                let requests = players.into_iter().map(|p| (p, ())).collect();
                let rctx: &Ctx = ctx;
                self.apnap_round(requests, |g, p, ()| {
                    let mut pctx = rctx.clone();
                    pctx.iter_player = Some(p);
                    let cands: Vec<ObjectId> = g
                        .objects_matching(filter, &pctx)
                        .into_iter()
                        .filter(|o| g.obj(*o).controller == p && !g.cant_be_sacrificed(*o))
                        .collect();
                    let k = n.min(cands.len() as u32);
                    let pick = g.ask_objects(
                        p,
                        rctx.source,
                        "Choose permanents to sacrifice",
                        cands,
                        k,
                        k,
                    );
                    g.record_apnap_choice(p, pick.clone());
                    for o in pick {
                        chosen.push((p, o));
                    }
                    vec![]
                });
                let sac: Vec<(ObjectId, PlayerId)> = chosen.iter().map(|(p, o)| (*o, *p)).collect();
                let mut sacrificed = Vec::new();
                for (old, new) in self.sacrifice_simultaneously(&sac) {
                    all.push(Entity::Object(new));
                    sacrificed.push(Entity::Object(old));
                }
                self.end_apnap_choices(round);
                for (_, o) in chosen {
                    ctx.prev_affected.push(Entity::Object(o));
                }
                ctx.prev_value = all.len() as i64;
                ctx.prev_happened = !all.is_empty();
                ctx.set_var(vars::IT, all);
                // "The sacrificed creature" (its last known information).
                if !sacrificed.is_empty() {
                    ctx.set_var(vars::SACRIFICED, sacrificed);
                }
            }
            Effect::SacrificeObjects { what } => {
                let objs = self.resolve_objects(what, ctx);
                // Sacrificed at the same time (CR 101.4).
                let what: Vec<(ObjectId, PlayerId)> = objs
                    .into_iter()
                    .filter(|o| self.is_live(*o))
                    .map(|o| (o, self.obj(o).controller))
                    .collect();
                let mut res = Vec::new();
                let mut sacrificed = Vec::new();
                for (old, new) in self.sacrifice_simultaneously(&what) {
                    res.push(Entity::Object(new));
                    sacrificed.push(Entity::Object(old));
                }
                ctx.prev_happened = !res.is_empty();
                ctx.set_var(vars::IT, res);
                if !sacrificed.is_empty() {
                    ctx.set_var(vars::SACRIFICED, sacrificed);
                }
            }
            Effect::Move { what, to } => {
                let objs = self.resolve_objects(what, ctx);
                let res = self.move_to_destination(objs, to, ctx);
                // CR 712.21c, 730.3c: a melded or merged permanent became several cards.
                let res = crate::merge::found_all(self, res);
                if to.zone == ZoneKind::Battlefield {
                    self.link_to_creator(ctx, &res);
                }
                ctx.prev_affected = res.iter().map(|o| Entity::Object(*o)).collect();
                ctx.set_var(vars::IT, res.into_iter().map(Entity::Object).collect());
            }
            Effect::Tap { what } => {
                for o in self.resolve_objects(what, ctx) {
                    self.tap(o);
                }
            }
            Effect::Untap { what } => {
                for o in self.resolve_objects(what, ctx) {
                    self.untap(o);
                }
            }
            Effect::DealDamage { source, amount, to } => {
                let src = self.damage_source(source, ctx);
                let n = self.eval_value(amount, ctx).max(0) as u32;
                let recipients = self.resolve_sel(to, ctx);
                if let Some(src) = src {
                    let evs = recipients.into_iter().map(|r| (src, r, n)).collect();
                    let before = self.events.len();
                    self.deal_damage_batch(evs, false);
                    self.record_damaged(src, before, ctx);
                    ctx.prev_value = n as i64;
                }
            }
            Effect::DealDamageExcess {
                source,
                amount,
                to,
                excess_to,
            } => {
                // CR 120.4a: the excess is split off before replacement and prevention
                // effects apply to the damage.
                let src = self.damage_source(source, ctx);
                let n = self.eval_value(amount, ctx).max(0) as u32;
                let recipients = self.resolve_sel(to, ctx);
                let other = self.resolve_sel(excess_to, ctx).into_iter().next();
                if let Some(src) = src {
                    let mut evs = Vec::new();
                    let mut excess_total = 0;
                    for r in recipients {
                        let (main, excess) = crate::excess_damage::split_excess(self, src, r, n);
                        evs.push((src, r, main));
                        if let (Some(o), true) = (other, excess > 0) {
                            evs.push((src, o, excess));
                        }
                        excess_total += excess;
                    }
                    self.deal_damage_batch(evs, false);
                    ctx.prev_value = excess_total as i64;
                }
            }
            Effect::DealDividedDamage { source, slot } => {
                if let Some(src) = self.damage_source(source, ctx) {
                    let targets = ctx.targets.get(*slot as usize).cloned().unwrap_or_default();
                    let div = ctx.divided.get(*slot as usize).cloned().unwrap_or_default();
                    // Divisions are kept aligned with the targets that are still legal
                    // (CR 608.2b; see `recheck_targets`).
                    let evs: Vec<(ObjectId, Entity, u32)> = targets
                        .iter()
                        .enumerate()
                        .map(|(i, t)| (src, *t, div.get(i).copied().unwrap_or(0)))
                        .collect();
                    let before = self.events.len();
                    self.deal_damage_batch(evs, false);
                    self.record_damaged(src, before, ctx);
                }
            }
            Effect::Fight { a, b } => {
                // CR 701.14
                let a = self
                    .resolve_objects(a, ctx)
                    .into_iter()
                    .find(|o| self.is_live(*o) && self.obj(*o).is_creature());
                let b = self
                    .resolve_objects(b, ctx)
                    .into_iter()
                    .find(|o| self.is_live(*o) && self.obj(*o).is_creature());
                if let (Some(a), Some(b)) = (a, b) {
                    let pa = self.obj(a).power().max(0) as u32;
                    let pb = self.obj(b).power().max(0) as u32;
                    if a == b {
                        // CR 701.14c: a creature that fights itself deals damage to itself
                        // equal to twice its power.
                        self.deal_damage_batch(vec![(a, Entity::Object(a), 2 * pa)], false);
                    } else {
                        self.deal_damage_batch(
                            vec![(a, Entity::Object(b), pa), (b, Entity::Object(a), pb)],
                            false,
                        );
                    }
                }
            }
            Effect::AddCounters { what, kind, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                for t in self.resolve_sel(what, ctx) {
                    self.add_counters(t, kind, k, ctx.source);
                }
            }
            Effect::RemoveCounters { what, kind, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                let mut total = 0;
                for t in self.resolve_sel(what, ctx) {
                    match kind {
                        Some(kind) => {
                            total += self.remove_counters_by(t, kind, k, Some(ctx.controller))
                        }
                        None => {
                            let kinds: Vec<CounterKind> = match t {
                                Entity::Object(o) => self.obj(o).counters.keys().cloned().collect(),
                                Entity::Player(p) => {
                                    self.player(p).counters.keys().cloned().collect()
                                }
                            };
                            for kk in kinds {
                                total += self.remove_counters_by(t, &kk, k, Some(ctx.controller));
                            }
                        }
                    }
                }
                ctx.prev_value = total as i64;
            }
            Effect::MoveCounters { from, to, kind, n } => {
                let from = self.resolve_objects(from, ctx).into_iter().next();
                let to = self.resolve_sel(to, ctx).into_iter().next();
                let mut total = 0;
                if let (Some(from), Some(to)) = (from, to) {
                    let present: Vec<CounterKind> =
                        self.obj(from).counters.keys().cloned().collect();
                    let kinds: Vec<CounterKind> = match (kind, n) {
                        (Some(k), _) => vec![k.clone()],
                        // "Move all counters": each kind.
                        (None, None) => present,
                        // "Move a counter": of a kind the player chooses.
                        (None, Some(_)) => {
                            let labels = present.iter().map(|k| k.to_string()).collect();
                            let i = self.ask_option(
                                ctx.controller,
                                ctx.source,
                                "Choose a kind of counter",
                                labels,
                            );
                            present.get(i).cloned().into_iter().collect()
                        }
                    };
                    for k in kinds {
                        let count = match n {
                            Some(v) => self.eval_value(v, ctx).max(0) as u32,
                            None => self.obj(from).counter(&k),
                        };
                        total += crate::counter_rules::move_counters(self, from, to, &k, count);
                    }
                }
                ctx.prev_value = total as i64;
            }
            Effect::PutCountersOf { from, to, kind } => {
                // "Its counters": the counters the object had as it last existed (CR 122.8),
                // not the new object it became.
                let from = self
                    .eval_sel(from, ctx)
                    .into_iter()
                    .find_map(|e| e.object());
                let mut total = 0;
                if let Some(from) = from {
                    for t in self.resolve_sel(to, ctx) {
                        total +=
                            crate::counter_rules::put_counters_of(self, from, t, kind.as_deref());
                    }
                }
                ctx.prev_value = total as i64;
            }
            Effect::Modify {
                what,
                mods,
                duration,
            } => {
                // CR 611.2b: a "for as long as" duration that already ended means the
                // effect does nothing.
                if self.effect_expired(duration, ctx.source, ctx.controller) {
                    return;
                }
                let objs: Vec<ObjectId> = self
                    .resolve_objects(what, ctx)
                    .into_iter()
                    .filter(|o| self.is_live(*o))
                    .collect();
                if objs.is_empty() {
                    return;
                }
                let fixed = self.fix_mods(mods, ctx);
                let ts = self.new_timestamp();
                // CR 612.5: an exchange of text boxes gives each object the other's text.
                let parts: Vec<(Option<ObjectId>, Vec<Modification>)> =
                    match crate::text_change::exchange_mods(self, &objs, &fixed) {
                        Some(v) => v.into_iter().map(|(o, m)| (Some(o), m)).collect(),
                        None => vec![(None, fixed)],
                    };
                for (o, part) in parts {
                    let id = self.new_effect_id();
                    self.effects.push(ContinuousEffect {
                        id,
                        source: ctx.source,
                        controller: ctx.controller,
                        timestamp: ts,
                        duration: duration.clone(),
                        affected: Affected::Objects(match o {
                            Some(o) => vec![o],
                            None => objs.clone(),
                        }),
                        mods: part,
                        layer1: None,
                        created_turn: self.turn.number,
                    });
                }
                self.dirty = true;
            }
            Effect::AddRestriction {
                restriction,
                duration,
            } => {
                // CR 611.2b: a "for as long as" duration that already ended.
                if self.effect_expired(duration, ctx.source, ctx.controller) {
                    return;
                }
                let id = self.new_effect_id();
                let ts = self.new_timestamp();
                let objects = self.lock_restriction_objects(restriction, ctx);
                self.rule_effects.push(RuleEffect {
                    id,
                    source: ctx.source,
                    controller: ctx.controller,
                    timestamp: ts,
                    duration: duration.clone(),
                    restriction: self.fix_restriction(restriction, ctx),
                    objects,
                });
                self.dirty = true;
            }
            Effect::AddPlayerEffect {
                who,
                effect,
                duration,
            } => {
                let players = self.eval_players(who, ctx);
                let id = self.new_effect_id();
                let ts = self.new_timestamp();
                self.player_effects.push(PlayerEffect {
                    id,
                    players,
                    controller: ctx.controller,
                    timestamp: ts,
                    duration: duration.clone(),
                    source: ctx.source,
                    effect: effect.clone(),
                });
                self.dirty = true;
            }
            Effect::AddReplacement {
                def,
                duration,
                uses,
            } => {
                let id = self.new_effect_id();
                let ts = self.new_timestamp();
                let objects = self.lock_replacement_objects(def, ctx);
                // Once locked onto specific objects, references to the resolving
                // ability's targets/variables can't be evaluated later (the instance is
                // matched without them), so they're dropped from the stored filter.
                let def = &match objects {
                    Some(_) => unlock_replacement_def(def),
                    None => def.clone(),
                };
                let remaining = match &def.action {
                    ReplacementAction::PreventAmount(v)
                    | ReplacementAction::PreventAndThen(Some(v), _) => {
                        Some(self.eval_value(v, ctx).max(0) as u32)
                    }
                    _ => None,
                };
                // Chosen objects the effect refers to are locked in (CR 609.7b, 611.2c).
                let def = &crate::prevention::lock_def(self, def, ctx);
                self.replacements.push(ReplacementInstance {
                    id,
                    source: ctx.source,
                    controller: ctx.controller,
                    timestamp: ts,
                    duration: duration.clone(),
                    def: def.clone(),
                    uses: *uses,
                    objects,
                    remaining,
                });
            }
            Effect::GainControl {
                what,
                who,
                duration,
            } => {
                // CR 611.2b (Master Thief).
                if self.effect_expired(duration, ctx.source, ctx.controller) {
                    return;
                }
                let objs: Vec<ObjectId> = self
                    .resolve_objects(what, ctx)
                    .into_iter()
                    .filter(|o| self.is_live(*o))
                    .collect();
                let Some(p) = self.eval_player(who, ctx) else {
                    return;
                };
                if objs.is_empty() {
                    return;
                }
                let id = self.new_effect_id();
                let ts = self.new_timestamp();
                self.effects.push(ContinuousEffect {
                    id,
                    source: ctx.source,
                    controller: ctx.controller,
                    timestamp: ts,
                    duration: duration.clone(),
                    affected: Affected::Objects(objs),
                    mods: vec![Modification::SetController(player_const(p))],
                    layer1: None,
                    created_turn: self.turn.number,
                });
                self.dirty = true;
                self.recompute();
            }
            Effect::ExchangeControl { a, b } => {
                // CR 701.12a: exactly two permanents, or no part of the exchange occurs
                // ("two target creatures" select both from one target slot).
                let mut both: Vec<ObjectId> = Vec::new();
                for o in self
                    .resolve_objects(a, ctx)
                    .into_iter()
                    .chain(self.resolve_objects(b, ctx))
                {
                    if !both.contains(&o) {
                        both.push(o);
                    }
                }
                let on_battlefield =
                    |g: &Self, o: &ObjectId| g.is_live(*o) && g.obj(*o).zone == Zone::Battlefield;
                if both.len() == 2 && both.iter().all(|o| on_battlefield(self, o)) {
                    let (a, b) = (both[0], both[1]);
                    let (ca, cb) = (self.obj(a).controller, self.obj(b).controller);
                    if ca == cb {
                        return;
                    }
                    for (obj, p) in [(a, cb), (b, ca)] {
                        let id = self.new_effect_id();
                        let ts = self.new_timestamp();
                        self.effects.push(ContinuousEffect {
                            id,
                            source: ctx.source,
                            controller: ctx.controller,
                            timestamp: ts,
                            duration: Duration::Permanent,
                            affected: Affected::Objects(vec![obj]),
                            mods: vec![Modification::SetController(player_const(p))],
                            layer1: None,
                            created_turn: self.turn.number,
                        });
                    }
                    self.dirty = true;
                }
            }
            Effect::CreateToken {
                spec,
                count,
                controller,
                tapped,
                attacking,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let players = self.eval_players(controller, ctx);
                let mut created = Vec::new();
                for p in players {
                    let chars = crate::tokens::token_characteristics(spec);
                    let attack = if *attacking {
                        self.attack_target_for_new_attacker(ctx)
                    } else {
                        None
                    };
                    let tc = TokenCreate {
                        chars,
                        card: crate::tokens::predefined_card(spec),
                        tapped: *tapped,
                        attacking: attack,
                        copy_of: None,
                        copy_exceptions: vec![],
                    };
                    created.extend(self.create_tokens(p, tc, n, ctx.source));
                }
                self.link_to_creator(ctx, &created);
                ctx.prev_value = created.len() as i64;
                ctx.set_var(
                    vars::CREATED,
                    created.into_iter().map(Entity::Object).collect(),
                );
            }
            Effect::CreateTokenAttached {
                spec,
                count,
                controller,
                to,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                // What they enter attached to; `None` if it's undefined (CR 303.4i).
                let attach = self.resolve_sel(to, ctx).first().copied();
                let players = self.eval_players(controller, ctx);
                let mut created = Vec::new();
                for p in players {
                    let tc = TokenCreate {
                        chars: crate::tokens::token_characteristics(spec),
                        card: crate::tokens::predefined_card(spec),
                        tapped: false,
                        attacking: None,
                        copy_of: None,
                        copy_exceptions: vec![],
                    };
                    created.extend(self.create_tokens_attached(p, tc, n, ctx.source, attach));
                }
                self.link_to_creator(ctx, &created);
                ctx.prev_value = created.len() as i64;
                ctx.set_var(
                    vars::CREATED,
                    created.into_iter().map(Entity::Object).collect(),
                );
            }
            Effect::CreateTokenCopy {
                of,
                count,
                controller,
                tapped,
                attacking,
                mods,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let sources = self.resolve_objects(of, ctx);
                let players = self.eval_players(controller, ctx);
                let fixed = self.fix_mods(mods, ctx);
                let mut created = Vec::new();
                for p in players {
                    for s in &sources {
                        let attack = if *attacking {
                            self.attack_target_for_new_attacker(ctx)
                        } else {
                            None
                        };
                        let tc = TokenCreate {
                            chars: self.obj(*s).copiable.clone(),
                            card: self.obj(*s).card.clone(),
                            tapped: *tapped,
                            attacking: attack,
                            copy_of: Some(*s),
                            copy_exceptions: fixed.clone(),
                        };
                        created.extend(self.create_tokens(p, tc, n, ctx.source));
                    }
                }
                ctx.set_var(
                    vars::CREATED,
                    created.into_iter().map(Entity::Object).collect(),
                );
            }
            Effect::CounterSpell { what } => {
                let mut any = false;
                for o in self.resolve_objects(what, ctx) {
                    any |= self.counter(o, ctx.source);
                }
                ctx.prev_happened = any;
            }
            Effect::CopySpell {
                what,
                count,
                new_targets,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                for o in self.resolve_objects(what, ctx) {
                    for _ in 0..n {
                        crate::copy::copy_spell(self, o, ctx.controller, *new_targets);
                    }
                }
            }
            Effect::OfferSpecialAction {
                def,
                duration,
                repeatable,
            } => {
                crate::special_actions::offer(
                    self,
                    (**def).clone(),
                    ctx,
                    duration.clone(),
                    *repeatable,
                );
            }
            Effect::PutSticker {
                who,
                what,
                kind,
                max_ticket,
                free,
            } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let max = max_ticket
                    .as_ref()
                    .map(|v| self.eval_value(v, ctx).max(0) as u32);
                let mut placed = false;
                for o in self.resolve_objects(what, ctx) {
                    placed |= crate::stickers::put_from_sheets(self, p, o, *kind, max, *free);
                }
                ctx.prev_happened = placed;
            }
            Effect::SpendAnyTypeMana {
                who,
                what,
                duration,
            } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let turn = self.turn.number;
                for o in self.resolve_objects(what, ctx) {
                    self.special
                        .any_type_mana
                        .push((p, o, duration.clone(), ctx.source, turn));
                }
            }
            Effect::ChangeTargets { what, who, how, to } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let forced = match to {
                    Some(sel) => match self.resolve_sel(sel, ctx).into_iter().next() {
                        Some(e) => Some(e),
                        None => return,
                    },
                    None => None,
                };
                let mut changed = false;
                for o in self.resolve_objects(what, ctx) {
                    changed |= crate::target_rules::change_targets(self, p, o, *how, forced);
                }
                ctx.prev_happened = changed;
            }
            Effect::BecomeCopy { what, of, duration } => {
                let targets = self.resolve_objects(what, ctx);
                let Some(src) = self.resolve_objects(of, ctx).into_iter().next() else {
                    return;
                };
                let values = Box::new(self.obj(src).copiable.clone());
                let id = self.new_effect_id();
                let ts = self.new_timestamp();
                self.effects.push(ContinuousEffect {
                    id,
                    source: ctx.source,
                    controller: ctx.controller,
                    timestamp: ts,
                    duration: duration.clone(),
                    affected: Affected::Objects(targets),
                    mods: vec![],
                    layer1: Some(Layer1::Copy {
                        values,
                        exceptions: vec![],
                    }),
                    created_turn: self.turn.number,
                });
                self.dirty = true;
            }
            Effect::Transform { what } => {
                for o in self.resolve_objects(what, ctx) {
                    // CR 701.27f: not if it transformed since its ability was put onto the
                    // stack (or, for a delayed triggered ability, created).
                    if crate::transform_rules::ability_may_transform(self, o, ctx) {
                        crate::dfc::transform(self, o);
                    }
                }
            }
            Effect::Regenerate { what } => {
                for o in self.resolve_objects(what, ctx) {
                    // CR 701.19: a regeneration shield for this turn.
                    let id = self.new_effect_id();
                    let ts = self.new_timestamp();
                    self.replacements.push(ReplacementInstance {
                        id,
                        source: ctx.source,
                        controller: ctx.controller,
                        timestamp: ts,
                        duration: Duration::EndOfTurn,
                        def: ReplacementDef {
                            event: ReplacementEvent::Destroy(Filter::Any),
                            action: ReplacementAction::Regenerate,
                            self_replacement: false,
                            optional: false,
                        },
                        uses: Some(1),
                        objects: Some(vec![o]),
                        remaining: None,
                    });
                }
            }
            Effect::Attach { what, to } => {
                let objs = self.resolve_objects(what, ctx);
                if let Some(t) = self.resolve_sel(to, ctx).into_iter().next() {
                    for o in objs {
                        self.attach(o, t);
                    }
                }
            }
            Effect::AttachAsCreature { what, to } => {
                let objs = self.resolve_objects(what, ctx);
                if let Some(t) = self.resolve_sel(to, ctx).into_iter().next() {
                    for o in objs {
                        self.attach_as_creature(o, t);
                    }
                }
            }
            Effect::Unattach { what } => {
                for o in self.resolve_objects(what, ctx) {
                    self.unattach(o);
                }
            }
            Effect::PhaseOut { what } => {
                let objs = self.resolve_objects(what, ctx);
                crate::keyword_impls::phase_out(self, objs);
            }
            Effect::TurnFaceUp { what } => {
                for o in self.resolve_objects(what, ctx) {
                    crate::facedown::turn_face_up(self, o, false);
                }
            }
            Effect::TurnFaceDown { what } => {
                let mut any = false;
                for o in self.resolve_objects(what, ctx) {
                    any |= crate::facedown::turn_face_down(self, o);
                }
                ctx.prev_happened = any;
            }
            Effect::RemoveFromCombat { what } => {
                for o in self.resolve_objects(what, ctx) {
                    crate::combat::remove_from_combat(self, o);
                }
            }
            Effect::Choose { who, kind } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                crate::choices::make_choice(self, p, kind, ctx);
            }
            // CR 614.1c: modify how the permanent enters (only while applying an "as this
            // enters" replacement effect).
            Effect::EnterTapped => {
                if let Some(e) = ctx.entering.as_mut() {
                    e.tapped = true;
                }
            }
            Effect::EnterPrepared => {
                if let Some(e) = ctx.entering.as_mut() {
                    e.prepared = true;
                }
            }
            Effect::EnterCopyExceptions(mods) => {
                if let Some(e) = ctx.entering.as_mut() {
                    e.copy_exceptions.extend(mods.iter().cloned());
                }
            }
            Effect::EnterCopyExtra {
                only_if,
                mods,
                effect,
            } => {
                let saved = ctx.clone();
                if let Some(e) = ctx.entering.as_mut() {
                    e.copy_extras.push(crate::copy_rules::CopyExtra {
                        ctx: Ctx {
                            entering: None,
                            ..saved
                        },
                        only_if: only_if.clone(),
                        mods: mods.clone(),
                        effect: (**effect).clone(),
                    });
                }
            }
            Effect::CopySpellRetargeted { what, target } => {
                let targets = target.as_ref().map(|t| self.resolve_sel(t, ctx));
                for o in self.resolve_objects(what, ctx) {
                    match &targets {
                        Some(ts) => {
                            crate::copy_rules::copy_with_target(
                                self,
                                o,
                                ctx.controller,
                                ts.clone(),
                            );
                        }
                        None => {
                            crate::copy_rules::copy_for_each_target(self, o, ctx.controller);
                        }
                    }
                }
            }
            Effect::CopyCard { what, named } => {
                crate::copy_rules::copy_cards(self, what, named, ctx);
            }
            Effect::OnEntry(effect) => {
                if let Some(e) = ctx.entering.as_mut() {
                    e.on_entry.push((**effect).clone());
                }
            }
            Effect::EnterAs(mods) => {
                if let Some(e) = ctx.entering.as_mut() {
                    e.copiable.extend(mods.iter().cloned());
                }
            }
            Effect::SetDayNight { day } => self.set_day(*day),
            Effect::SetPrepared { what, prepared } => {
                for o in self.resolve_objects(what, ctx) {
                    if *prepared {
                        crate::designations::become_prepared(self, o);
                    } else {
                        crate::designations::become_unprepared(self, o);
                    }
                }
            }
            Effect::EnterWithCounters { kind, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                if let Some(e) = ctx.entering.as_mut() {
                    if k > 0 {
                        e.counters.push((kind.clone(), k));
                    }
                }
            }

            // --- Players -----------------------------------------------------------
            Effect::Draw { who, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                let mut drawn = Vec::new();
                // CR 121.2c: the active player draws first, then the others in turn order.
                let players = crate::draw_rules::draw_order(self, self.eval_players(who, ctx));
                for p in players {
                    drawn.extend(self.draw_cards(p, k));
                }
                ctx.prev_value = drawn.len() as i64;
                ctx.set_var(
                    vars::REVEALED,
                    drawn.into_iter().map(Entity::Object).collect(),
                );
            }
            Effect::Discard {
                who,
                n,
                random,
                filter,
            } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                let mut discarded = Vec::new();
                // CR 101.4: each player chooses in APNAP order — cards chosen in a hand stay
                // face down (CR 101.4a) — then the cards are discarded.
                let round = self.apnap_choices.len();
                let mut picks: Vec<(PlayerId, Vec<ObjectId>)> = Vec::new();
                let requests = self
                    .eval_players(who, ctx)
                    .into_iter()
                    .map(|p| (p, ()))
                    .collect();
                let rctx: &Ctx = ctx;
                self.apnap_round(requests, |g, p, ()| {
                    let hand: Vec<ObjectId> = g
                        .player(p)
                        .hand
                        .clone()
                        .into_iter()
                        .filter(|c| g.matches(*c, filter, rctx))
                        .collect();
                    let k = k.min(hand.len() as u32);
                    let pick = if *random {
                        use rand::seq::SliceRandom;
                        let mut h = hand.clone();
                        h.shuffle(&mut g.rng);
                        h.into_iter().take(k as usize).collect()
                    } else {
                        g.ask_objects(p, rctx.source, "Choose cards to discard", hand, k, k)
                    };
                    g.record_apnap_choice(p, pick.clone());
                    picks.push((p, pick));
                    vec![]
                });
                for (p, pick) in picks {
                    for c in pick {
                        if let Some(n) = self.discard(p, c, ctx.source) {
                            discarded.push(Entity::Object(n));
                        }
                    }
                }
                self.end_apnap_choices(round);
                ctx.prev_value = discarded.len() as i64;
                ctx.prev_happened = !discarded.is_empty();
                ctx.prev_affected = discarded.clone();
                ctx.set_var(vars::IT, discarded);
            }
            Effect::DiscardHand { who } => {
                let mut n = 0;
                for p in self.eval_players(who, ctx) {
                    for c in self.player(p).hand.clone() {
                        if self.discard(p, c, ctx.source).is_some() {
                            n += 1;
                        }
                    }
                }
                ctx.prev_value = n;
            }
            Effect::Mill { who, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                let mut all = Vec::new();
                for p in self.eval_players(who, ctx) {
                    all.extend(self.mill(p, k));
                }
                ctx.prev_value = all.len() as i64;
                ctx.set_var(vars::IT, all.into_iter().map(Entity::Object).collect());
            }
            Effect::GainLife { who, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                for p in self.eval_players(who, ctx) {
                    self.gain_life(p, k);
                }
            }
            Effect::LoseLife { who, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                let mut total = 0;
                for p in self.eval_players(who, ctx) {
                    total += self.lose_life(p, k);
                }
                ctx.prev_value = total as i64;
            }
            Effect::SetLife { who, n } => {
                // CR 119.5: gaining or losing the difference.
                let k = self.eval_value(n, ctx) as i32;
                for p in self.eval_players(who, ctx) {
                    let cur = self.player(p).life;
                    if k > cur {
                        self.gain_life(p, (k - cur) as u32);
                    } else if k < cur {
                        self.lose_life(p, (cur - k) as u32);
                    }
                }
            }
            Effect::ExchangeLifeTotals { a, b } => {
                let a = self.eval_player(a, ctx);
                let b = self.eval_player(b, ctx);
                ctx.prev_happened = match (a, b) {
                    (Some(a), Some(b)) => crate::life_totals::exchange_life_totals(self, a, b),
                    _ => false,
                };
            }
            Effect::AddMana {
                who,
                mana,
                restriction,
            } => {
                crate::mana_abilities::resolve_add_mana(self, who, mana, restriction, ctx);
            }
            Effect::AddManaWithSpentTrigger {
                add,
                spell_filter,
                body,
            } => {
                crate::mana_abilities::resolve_add_mana_with_rider(
                    self,
                    add,
                    spell_filter,
                    body,
                    ctx,
                );
            }
            Effect::PersistentMana(inner) => {
                crate::mana_abilities::resolve_persistent_mana(self, inner, ctx);
            }
            Effect::SetClassLevel { level } => {
                if let Some(s) = ctx.source.filter(|s| self.is_live(*s)) {
                    self.obj_mut(s).class_level = *level;
                }
            }
            Effect::ActivateManaAbilities { who, filter } => {
                crate::mana_abilities::activate_mana_abilities_of_each(self, who, filter, ctx);
            }
            Effect::LoseUnspentMana { who, to } => {
                crate::mana_abilities::lose_unspent_mana(self, who, to.as_ref(), ctx);
            }
            Effect::AddPlayerCounters { who, kind, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                for p in self.eval_players(who, ctx) {
                    self.add_counters(Entity::Player(p), kind, k, ctx.source);
                }
            }
            Effect::Scry { who, n } => {
                // CR 701.22c: players scrying at once do so at the same time.
                let k = self.eval_value(n, ctx).max(0) as u32;
                let players = self.eval_players(who, ctx);
                let look = crate::scry_rules::Look::Scry;
                crate::scry_rules::perform(self, &players, k, look, ctx.source);
            }
            Effect::Surveil { who, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                let players = self.eval_players(who, ctx);
                let look = crate::scry_rules::Look::Surveil;
                crate::scry_rules::perform(self, &players, k, look, ctx.source);
            }
            Effect::Search {
                who,
                whose,
                filter,
                count,
                to,
                reveal,
                shuffle,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let mut searchers = self.eval_players(who, ctx);
                if searchers.is_empty() {
                    searchers.push(ctx.controller);
                }
                // CR 701.23i: several players searching at once look at the cards at the
                // same time and choose in APNAP order; then the found cards move.
                let mut founds: Vec<(PlayerId, PlayerId, Vec<ObjectId>)> = Vec::new();
                for p in searchers {
                    let mut c = ctx.clone();
                    c.iter_player = Some(p);
                    let owner = self.eval_player(whose, &c).unwrap_or(p);
                    let found = crate::library::search(self, p, owner, filter, n, &c);
                    founds.push((p, owner, found));
                }
                let mut all = Vec::new();
                for (p, owner, found) in founds {
                    let mut c = ctx.clone();
                    c.iter_player = Some(p);
                    let res = if *shuffle
                        && to.zone == ZoneKind::Library
                        && matches!(to.position, LibraryPosition::Top)
                    {
                        // "Then shuffle and put that card on top" (CR 701.24b): the found
                        // cards stay in the library but aren't shuffled, then go on top
                        // (no zone change).
                        self.shuffle_library(owner);
                        crate::library::put_on_top(self, owner, &found);
                        // CR 701.23e: revealed only if the effect says so.
                        if *reveal {
                            crate::reveal::reveal_in(self, p, &found, Some(&c));
                        }
                        found
                    } else {
                        // CR 701.23e: revealed only if the effect says so.
                        if *reveal {
                            crate::reveal::reveal_in(self, p, &found, Some(&c));
                        }
                        let res = self.move_to_destination(found, to, &mut c);
                        if *shuffle {
                            self.shuffle_library(owner);
                        }
                        res
                    };
                    all.extend(res);
                }
                ctx.prev_affected = all.iter().map(|o| Entity::Object(*o)).collect();
                ctx.set_var(vars::IT, all.into_iter().map(Entity::Object).collect());
            }
            Effect::Shuffle { who } => {
                for p in self.eval_players(who, ctx) {
                    self.shuffle_library(p);
                }
            }
            Effect::ShuffleInto { what } => {
                let objs = self.resolve_objects(what, ctx);
                let mut owners = Vec::new();
                for o in objs {
                    let owner = self.obj(o).owner;
                    self.move_object_ev(MoveEv {
                        obj: o,
                        to: Zone::Library(owner),
                        pos: LibraryPosition::Top,
                        cause: MoveCause::Effect,
                        by: Some(ctx.controller),
                        etb: EtbInfo::default(),
                        source: ctx.source,
                    });
                    if !owners.contains(&owner) {
                        owners.push(owner);
                    }
                }
                for o in owners {
                    self.shuffle_library(o);
                }
            }
            Effect::ShuffleIntoLibrary { what, library } => {
                let objs = self.resolve_objects(what, ctx);
                // CR 701.24c, 701.24d: the libraries are shuffled even if the objects
                // aren't where they're expected to be, or there are none.
                let mut libraries = self.eval_players(library, ctx);
                for o in &objs {
                    let owner = self.obj(*o).owner;
                    if !libraries.contains(&owner) {
                        libraries.push(owner);
                    }
                }
                let moves: Vec<MoveEv> = objs
                    .iter()
                    .filter(|o| self.is_live(**o))
                    .map(|o| MoveEv {
                        obj: *o,
                        to: Zone::Library(self.obj(*o).owner),
                        pos: LibraryPosition::Top,
                        cause: MoveCause::Effect,
                        by: Some(ctx.controller),
                        etb: EtbInfo::default(),
                        source: ctx.source,
                    })
                    .collect();
                let moved: Vec<ObjectId> = self.move_objects(moves).into_iter().flatten().collect();
                for p in libraries {
                    self.shuffle_library(p);
                }
                let moved: Vec<Entity> = moved
                    .into_iter()
                    .filter(|o| matches!(self.obj(*o).zone, Zone::Library(_)))
                    .map(Entity::Object)
                    .collect();
                ctx.prev_value = moved.len() as i64;
                ctx.prev_happened = !moved.is_empty();
                ctx.prev_affected = moved.clone();
                ctx.set_var(vars::IT, moved);
            }
            Effect::Dig {
                who,
                n,
                reveal,
                filter,
                take,
                take_up_to,
                take_to,
                rest_to,
            } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let k = self.eval_value(n, ctx).max(0) as u32;
                let t = self.eval_value(take, ctx).max(0) as u32;
                crate::library::dig(
                    self,
                    p,
                    k,
                    *reveal,
                    filter,
                    t,
                    *take_up_to,
                    take_to,
                    rest_to,
                    ctx,
                );
            }
            // CR 701.20a: the cards are revealed while the rest of the effect needs them.
            Effect::RevealHand { who } => {
                for p in self.eval_players(who, ctx) {
                    let hand = self.player(p).hand.clone();
                    crate::reveal::reveal_in(self, p, &hand, Some(ctx));
                }
            }
            Effect::RevealUntil {
                who,
                filter,
                found_to,
                rest_to,
            } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                crate::library::reveal_until(self, p, filter, found_to, rest_to, ctx);
            }
            Effect::ExtraTurn { who } => {
                // CR 500.7: most recently created extra turn is taken first.
                for p in self.eval_players(who, ctx) {
                    self.extra_turns.push(p);
                }
            }
            Effect::ExtraCombat { after_this } => {
                let _ = after_this;
                self.add_extra_combat(true);
            }
            Effect::AddTurnParts {
                parts,
                after_phase,
                n,
                who,
            } => {
                // CR 500.10a: "you get" adds nothing to another player's turn.
                let gets = who.as_ref().map(|w| self.eval_players(w, ctx));
                if gets.is_none_or(|ps| ps.iter().any(|p| self.is_active_player(*p))) {
                    let n = self.eval_value(n, ctx).max(0) as usize;
                    for _ in 0..n {
                        self.add_turn_parts(parts, *after_phase);
                    }
                }
            }
            Effect::Skip { who, step } => {
                for p in self.eval_players(who, ctx) {
                    self.players[p.idx()].skips.push(*step);
                }
            }
            Effect::DelayedTrigger {
                trigger,
                body,
                once,
            } => {
                // CR 603.7a: it won't trigger on events that happened before it was created.
                self.flush_events();
                let id = self.new_effect_id();
                self.delayed_triggers.push(DelayedTrigger {
                    id,
                    source: ctx.source,
                    controller: ctx.controller,
                    trigger: trigger.clone(),
                    body: (**body).clone(),
                    once: *once,
                    ctx: crate::transform_rules::delayed_ctx(self, ctx),
                    created_turn: self.turn.number,
                    created_step: Some(self.turn.step),
                    for_rest_of_game: false,
                });
            }
            Effect::Reflexive { body } => {
                // CR 603.12: a reflexive triggered ability is checked immediately after it's
                // created; it triggers now and waits to be put on the stack (with its own
                // targets) until a player would receive priority. It's controlled by the
                // controller of the resolving spell or ability (CR 603.7d–e).
                self.trigger_order += 1;
                let ability = AbilityDef::new(
                    AbilityKind::Triggered(TriggeredAbility::new(
                        TriggerCond::Custom("reflexive".into()),
                        (**body).clone(),
                    )),
                    "reflexive trigger",
                );
                let src = ctx
                    .stack_obj
                    .filter(|s| self.obj(*s).is_spell())
                    .or(ctx.source);
                self.pending_triggers.push(PendingTrigger {
                    source: src.unwrap_or(ObjectId(0)),
                    controller: ctx.controller,
                    ability,
                    event: ctx.event.clone().unwrap_or_default(),
                    source_lki: src.map(|s| Box::new(self.obj(s).chars.clone())),
                    saved: Some(ctx.clone()),
                    body: Some((**body).clone()),
                    order: self.trigger_order,
                });
            }
            Effect::AtNext { step, effect } => {
                self.flush_events();
                let id = self.new_effect_id();
                self.delayed_triggers.push(DelayedTrigger {
                    id,
                    source: ctx.source,
                    controller: ctx.controller,
                    trigger: TriggerCond::BeginningOf {
                        step: *step,
                        whose: PlayerRel::Any,
                    },
                    body: Body::effect((**effect).clone()),
                    once: true,
                    ctx: ctx.clone(),
                    created_turn: self.turn.number,
                    created_step: Some(self.turn.step),
                    for_rest_of_game: false,
                });
            }
            Effect::CreateEmblem { who, abilities } => {
                for p in self.eval_players(who, ctx) {
                    crate::tokens::create_emblem(self, p, abilities.clone(), ctx.source);
                }
            }
            Effect::RestartGame { keep } => {
                crate::restart::request_restart(self, keep.as_ref(), ctx);
            }
            Effect::KeepAndSacrificeRest { who, among, keep } => {
                crate::apnap::keep_and_sacrifice_rest(self, who, among, keep, ctx);
            }
            Effect::WinGame { who } => {
                // CR 104.2b, 104.3f: players named together win simultaneously.
                let ps = self.eval_players(who, ctx);
                self.players_win(&ps);
            }
            Effect::LoseGame { who } => {
                // CR 104.3e; "can't lose" effects win (CR 101.2).
                let ps = self.eval_players(who, ctx);
                self.lose_game_simultaneously(&ps);
            }
            Effect::CastCard {
                who,
                what,
                free,
                optional,
            } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let mut cast: Vec<Entity> = Vec::new();
                for o in self.resolve_objects(what, ctx) {
                    if *optional && !self.ask_yes_no(p, Some(o), "Cast this card?", true) {
                        continue;
                    }
                    // CR 118.8c: casting "if able" isn't required when the spell has a
                    // mandatory additional cost involving hidden cards with a quality.
                    if !*optional && crate::cost_rules::may_decline_cast_if_able(self, p, o) {
                        continue;
                    }
                    let method = if *free {
                        CastMethod::Free
                    } else {
                        CastMethod::Normal
                    };
                    if let Ok(spell) = crate::casting::cast_during_resolution(self, p, o, method) {
                        cast.push(Entity::Object(spell));
                    }
                }
                // CR 400.7h: other parts of the effect can find the spells cast this way.
                ctx.set_var(vars::IT, cast);
            }
            Effect::PlayCard {
                who,
                what,
                free,
                optional,
            } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                for o in self.resolve_objects(what, ctx) {
                    if *optional && !self.ask_yes_no(p, Some(o), "Play this card?", true) {
                        continue;
                    }
                    // A face-down card (e.g. exiled with hideaway) is played face up.
                    if self.face_characteristics(o, FaceState::Front).is_land() {
                        // CR 305.2b, 305.3: ignored if the player can't play a land now.
                        let _ = self.play_land_during_resolution(p, o);
                        continue;
                    }
                    let method = if *free {
                        CastMethod::Free
                    } else {
                        CastMethod::Normal
                    };
                    let _ = crate::casting::cast_during_resolution(self, p, o, method);
                }
            }
            Effect::GrantPlayPermission {
                who,
                what,
                duration,
                free,
            } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let objs = self.resolve_objects(what, ctx);
                crate::casting::grant_play_permission(
                    self,
                    p,
                    objs,
                    duration.clone(),
                    *free,
                    ctx.source,
                );
            }
            Effect::PreventDamage {
                to,
                amount,
                duration,
                combat_only,
            } => {
                let targets = self.resolve_sel(to, ctx);
                for t in targets {
                    let id = self.new_effect_id();
                    let ts = self.new_timestamp();
                    let (to_players, to_objects, objects) = match t {
                        Entity::Player(p) => (Some(player_filter_const(p)), None, None),
                        Entity::Object(o) => (None, Some(Filter::Any), Some(vec![o])),
                    };
                    let action = match amount {
                        Some(v) => ReplacementAction::PreventAmount(Value::Const(
                            self.eval_value(v, ctx) as i32,
                        )),
                        None => ReplacementAction::Prevent,
                    };
                    let remaining = match amount {
                        Some(v) => Some(self.eval_value(v, ctx).max(0) as u32),
                        None => None,
                    };
                    self.replacements.push(ReplacementInstance {
                        id,
                        source: ctx.source,
                        controller: ctx.controller,
                        timestamp: ts,
                        duration: duration.clone(),
                        def: ReplacementDef {
                            event: ReplacementEvent::Damage {
                                source: Filter::Any,
                                to_players,
                                to_objects,
                                combat_only: *combat_only,
                            },
                            action,
                            self_replacement: false,
                            optional: false,
                        },
                        uses: None,
                        objects,
                        remaining,
                    });
                }
            }
            Effect::BecomeMonarch { who } => {
                if let Some(p) = self.eval_player(who, ctx) {
                    crate::designations::become_monarch(self, p);
                }
            }
            Effect::TakeInitiative { who } => {
                if let Some(p) = self.eval_player(who, ctx) {
                    crate::designations::take_initiative(self, p);
                }
            }
            Effect::KeywordAction {
                action,
                who,
                what,
                n,
            } => {
                crate::keyword_actions::perform(self, *action, who, what, n, ctx);
            }
            Effect::KeywordActionEx(spec) => crate::kwa::perform_spec(self, spec, ctx),
            Effect::ChangeText {
                what,
                words,
                exclude,
                duration,
            } => {
                let objs = self.resolve_objects(what, ctx);
                crate::text_change::exec_change_text(self, objs, *words, exclude, duration, ctx);
            }
            Effect::ExileUntil { what, until } => {
                let objs = self.resolve_objects(what, ctx);
                crate::until::exec_exile_until(self, objs, until, ctx);
            }
            Effect::PhaseOutUntil { what, until } => {
                let objs = self.resolve_objects(what, ctx);
                crate::until::exec_phase_out_until(self, objs, until, ctx);
            }
            Effect::SelfReplace {
                replacement,
                effect,
            } => {
                // CR 614.15: a self-replacement effect applies to this effect's own events,
                // before other replacement effects (CR 616.1a).
                let mut def = crate::prevention::lock_def(self, replacement, ctx);
                def.self_replacement = true;
                let id = self.new_effect_id();
                let ts = self.new_timestamp();
                self.replacements.push(ReplacementInstance {
                    id,
                    source: ctx.source,
                    controller: ctx.controller,
                    timestamp: ts,
                    duration: Duration::Permanent,
                    def,
                    uses: None,
                    objects: None,
                    remaining: None,
                });
                self.exec(effect, ctx);
                self.replacements.retain(|r| r.id != id);
            }
            Effect::ChooseSource { who, filter, var } => {
                crate::prevention::exec_choose_source(self, who, filter, *var, ctx);
            }
            Effect::NextSpell {
                filter,
                mods,
                expires,
            } => {
                crate::next_spell::exec_next_spell(self, filter, mods, expires, ctx);
            }
            Effect::RollDice(spec) => crate::dice::roll(self, spec, ctx),
            Effect::Piles(action) => crate::piles::perform(self, action, ctx),
            Effect::Exchange(spec) => crate::exchange::perform(self, spec, ctx),
            Effect::FlipCoins(spec) => crate::dice::flip(self, spec, ctx),
            Effect::Custom(name) => crate::custom::custom_effect(self, name, ctx),
        }
    }

    /// Resolves a selection, making any choices it requires (Sel::Choose).
    pub fn resolve_sel(&mut self, sel: &Sel, ctx: &mut Ctx) -> Vec<Entity> {
        match sel {
            Sel::Choose {
                chooser,
                filter,
                count,
                up_to,
                store,
            } => {
                let p = self.eval_player(chooser, ctx).unwrap_or(ctx.controller);
                let n = self.eval_value(count, ctx).max(0) as u32;
                // CR 614.13a: objects entering the battlefield right now can't be chosen.
                let cands: Vec<ObjectId> = self
                    .objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| !self.entering.contains(o))
                    .collect();
                let min = if *up_to { 0 } else { n.min(cands.len() as u32) };
                // CR 406.4: face-down exiled cards the player can't look at are chosen by
                // pile.
                let picked: Vec<Entity> =
                    crate::zones::choose_objects(self, p, ctx.source, "Choose", cands, min, n)
                        .into_iter()
                        .map(Entity::Object)
                        .collect();
                if let Some(v) = store {
                    ctx.vars.insert(*v, picked.clone());
                }
                picked
            }
            Sel::Union(v) => {
                let mut out = Vec::new();
                for s in v {
                    for e in self.resolve_sel(s, ctx) {
                        if !out.contains(&e) {
                            out.push(e);
                        }
                    }
                }
                out
            }
            Sel::This | Sel::TriggerLki => {
                let v = self.eval_sel(sel, ctx);
                v.into_iter()
                    .map(|e| self.follow_zone_change_trigger_object(e, ctx))
                    .collect()
            }
            other => self.eval_sel(other, ctx),
        }
    }

    /// While an "as this enters" replacement effect is being applied (CR 614.12a), the
    /// entering object isn't on the battlefield yet. Tapping it or putting counters on it
    /// modifies how it enters (CR 614.1c, 122.6); other effects on it happen as it's put
    /// onto the battlefield. Returns true if `e` was handled that way.
    fn effect_on_entering_object(&mut self, e: &Effect, ctx: &mut Ctx) -> bool {
        match e {
            Effect::Tap { what: Sel::This } => {
                if let Some(em) = ctx.entering.as_mut() {
                    em.tapped = true;
                }
                true
            }
            Effect::AddCounters {
                what: Sel::This,
                kind,
                n,
            } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                if let Some(em) = ctx.entering.as_mut() {
                    if k > 0 {
                        em.counters.push((kind.clone(), k));
                    }
                }
                true
            }
            Effect::Untap { what: Sel::This }
            | Effect::Modify {
                what: Sel::This, ..
            } => {
                if let Some(em) = ctx.entering.as_mut() {
                    em.on_entry.push(e.clone());
                }
                true
            }
            _ => false,
        }
    }

    /// CR 400.7e: an ability that triggers when an object moves from one zone to another
    /// can find the new object it became in the zone it moved to, if that zone is public
    /// ("When ~ dies, return it to its owner's hand"). Information about the object (its
    /// power, etc.) still uses last known information, via `eval_sel`.
    fn follow_zone_change_trigger_object(&self, e: Entity, ctx: &Ctx) -> Entity {
        let Entity::Object(id) = e else {
            return e;
        };
        if self.is_live(id) {
            return e;
        }
        let Some(ev) = ctx.event.as_ref() else {
            return e;
        };
        match (ev.lki, ev.object) {
            (Some(old), Some(new)) if old == id && new != id => {
                let public = !matches!(
                    self.obj(new).zone,
                    Zone::Hand(_) | Zone::Library(_) | Zone::Outside(_) | Zone::Nowhere
                );
                if public {
                    Entity::Object(new)
                } else {
                    e
                }
            }
            _ => e,
        }
    }

    pub fn resolve_objects(&mut self, sel: &Sel, ctx: &mut Ctx) -> Vec<ObjectId> {
        self.resolve_sel(sel, ctx)
            .into_iter()
            .filter_map(|e| e.object())
            .collect()
    }

    /// Records the objects that were actually dealt damage by `src` since event index
    /// `before` (after replacement and prevention) as "dealt damage this way"
    /// ([`vars::DAMAGED`]) and as the previous effect's affected objects.
    fn record_damaged(&mut self, src: ObjectId, before: usize, ctx: &mut Ctx) {
        let mut damaged: Vec<Entity> = Vec::new();
        for ev in &self.events[before.min(self.events.len())..] {
            if let Event::Damage {
                source,
                target: Entity::Object(o),
                amount,
                ..
            } = ev
            {
                if *source == src && *amount > 0 && !damaged.contains(&Entity::Object(*o)) {
                    damaged.push(Entity::Object(*o));
                }
            }
        }
        ctx.prev_affected = damaged.clone();
        ctx.set_var(vars::DAMAGED, damaged);
    }

    /// The source of damage for an effect: the named object, or the resolving object's
    /// source (CR 120.2, 609.7). Uses last known information if it has left.
    fn damage_source(&mut self, sel: &Sel, ctx: &mut Ctx) -> Option<ObjectId> {
        match sel {
            Sel::None | Sel::This => ctx
                .stack_obj
                .filter(|s| self.obj(*s).is_spell())
                .or(ctx.source),
            other => self.resolve_objects(other, ctx).into_iter().next(),
        }
    }

    /// Evaluates dynamic values in modifications once, at resolution (CR 608.2h, 611.2c).
    pub fn fix_mods(&self, mods: &[Modification], ctx: &Ctx) -> Vec<Modification> {
        // "gain [keywords] until end of turn if [objects] have them" (CR 702.1c): which
        // keywords is determined as the effect is created.
        let mods: Vec<Modification> = mods
            .iter()
            .flat_map(|m| match m {
                Modification::AddKeywordsOf { kinds, from } => {
                    crate::layers::keywords_of(self, kinds, from, ctx)
                        .into_iter()
                        .map(Modification::AddKeyword)
                        .collect()
                }
                other => vec![other.clone()],
            })
            .collect();
        mods.iter()
            .map(|m| match m {
                Modification::ModifyPT(p, t) => Modification::ModifyPT(
                    Value::Const(self.eval_value(p, ctx) as i32),
                    Value::Const(self.eval_value(t, ctx) as i32),
                ),
                Modification::SetPT(p, t) => Modification::SetPT(
                    p.as_ref()
                        .map(|v| Value::Const(self.eval_value(v, ctx) as i32)),
                    t.as_ref()
                        .map(|v| Value::Const(self.eval_value(v, ctx) as i32)),
                ),
                Modification::SetController(r) => match self.eval_player(r, ctx) {
                    Some(p) => Modification::SetController(player_const(p)),
                    None => m.clone(),
                },
                // Values chosen for the source are locked in as the effect is created
                // (CR 608.2h, 607.2d).
                Modification::AddKeyword(k)
                    if k.filter
                        .as_ref()
                        .is_some_and(crate::choices::filter_mentions_choice) =>
                {
                    let mut k = k.clone();
                    if let (Some(f), Some(src)) = (k.filter.as_ref(), ctx.source) {
                        k.filter = Some(crate::choices::bind_choices(f, &self.obj(src).choices));
                    }
                    Modification::AddKeyword(k)
                }
                Modification::SetChosenColor => {
                    match ctx.source.and_then(|s| self.obj(s).choices.color) {
                        Some(c) => Modification::SetColors(ColorSet::single(c)),
                        None => m.clone(),
                    }
                }
                Modification::AddChosenType => {
                    match ctx.source.and_then(|s| {
                        let ch = &self.obj(s).choices;
                        ch.creature_type.clone().or(ch.basic_land_type.clone())
                    }) {
                        Some(t) => Modification::AddSubtypes(vec![t]),
                        None => m.clone(),
                    }
                }
                Modification::SetChosenBasicLandType => {
                    match ctx
                        .source
                        .and_then(|s| self.obj(s).choices.basic_land_type.clone())
                    {
                        Some(t) => Modification::SetBasicLandType(vec![t]),
                        None => m.clone(),
                    }
                }
                other => other.clone(),
            })
            .collect()
    }

    /// A restriction locked onto specific objects (see [`Self::lock_restriction_objects`])
    /// applies to exactly those objects: its filter named them through this resolution's
    /// targets or event ("target creature", "that creature"), which aren't available when
    /// the restriction is checked later, so it becomes "any object" (of the locked ones).
    fn fix_restriction(&self, r: &Restriction, ctx: &Ctx) -> Restriction {
        let mut r = r.clone();
        if let Some(f) = restriction_object_filter(&mut r) {
            if filter_references_specific(f) {
                *f = Filter::Any;
            }
        }
        // "Target creature blocks this creature this combat if able": both creatures are
        // the objects named as the effect began.
        // Likewise "target creature can't block this creature this turn".
        if let Restriction::MustBlockAttacker { blocker, attacker }
        | Restriction::CantBeBlockedBy { attacker, blocker } = &mut r
        {
            for f in [blocker, attacker] {
                if filter_references_specific(f) {
                    *f = Filter::Objects(self.named_objects(f, ctx));
                }
            }
        }
        // A restriction on a referenced player ("target player can't play lands this
        // turn") locks onto that player.
        if let Some(pf) = restriction_player_filter(&mut r) {
            if let PlayerFilter::Ref(who) = pf {
                let ps = self.eval_players(who, ctx);
                *pf = PlayerFilter::Or(ps.into_iter().map(PlayerFilter::Is).collect());
            }
        }
        r
    }

    /// Restrictions naming specific objects ("target creature can't block this turn")
    /// lock onto those objects.
    fn lock_restriction_objects(&self, r: &Restriction, ctx: &Ctx) -> Option<Vec<ObjectId>> {
        let mut r = r.clone();
        let f = restriction_object_filter(&mut r)?;
        if filter_references_specific(f) {
            Some(self.named_objects(f, ctx))
        } else {
            None
        }
    }

    /// The objects a filter naming specific objects matches as an effect begins.
    fn named_objects(&self, f: &Filter, ctx: &Ctx) -> Vec<ObjectId> {
        let mut v = self.objects_matching(f, ctx);
        // Targets outside the battlefield ("target spell can't be countered"), and the
        // resolving spell itself ("the damage [this spell deals] can't be prevented").
        let extra = ctx
            .targets
            .iter()
            .flatten()
            .filter_map(|e| e.object())
            .chain(ctx.source);
        for o in extra {
            if !v.contains(&o) && self.matches(o, f, ctx) {
                v.push(o);
            }
        }
        v
    }

    fn lock_replacement_objects(&self, d: &ReplacementDef, ctx: &Ctx) -> Option<Vec<ObjectId>> {
        let f = match &d.event {
            ReplacementEvent::Destroy(f)
            | ReplacementEvent::Dies(f)
            | ReplacementEvent::EntersBattlefield(f) => f,
            ReplacementEvent::ZoneChange { filter, .. } => filter,
            _ => return None,
        };
        if filter_references_specific(f) {
            Some(self.eval_filter_specific(f, ctx))
        } else {
            None
        }
    }

    fn eval_filter_specific(&self, f: &Filter, ctx: &Ctx) -> Vec<ObjectId> {
        match f {
            Filter::In(sel) => self.eval_sel_objects(sel, ctx),
            Filter::Source => ctx.source.into_iter().collect(),
            Filter::And(v) => v
                .iter()
                .flat_map(|x| self.eval_filter_specific(x, ctx))
                .collect(),
            _ => vec![],
        }
    }

    /// Moves objects to a destination (hand, library, battlefield, exile, graveyard).
    pub fn move_to_destination(
        &mut self,
        objs: Vec<ObjectId>,
        to: &Destination,
        ctx: &mut Ctx,
    ) -> Vec<ObjectId> {
        let controller = to
            .controller
            .as_ref()
            .and_then(|r| self.eval_player(r, ctx));
        // "under its owner's control" / "under their owners' control": each object
        // enters under its own owner's control.
        let owners_control = matches!(to.controller, Some(PlayerRef::OwnerOf(_)));
        let mut counters: Vec<(CounterKind, u32)> = Vec::new();
        for (k, v) in &to.with_counters {
            counters.push((k.clone(), self.eval_value(v, ctx).max(0) as u32));
        }
        let attack = if to.attacking {
            self.attack_target_for_new_attacker(ctx)
        } else {
            None
        };
        let with_mods = if to.zone == ZoneKind::Battlefield && !to.with_mods.is_empty() {
            Some((
                ctx.source,
                ctx.controller,
                self.fix_mods(&to.with_mods, ctx),
            ))
        } else {
            None
        };
        // "Put onto the battlefield attached to [x]": `None` if x is undefined
        // (CR 301.5e, 303.4i).
        let attach_to = match (&to.attached_to, to.zone) {
            (Some(sel), ZoneKind::Battlefield) => Some(self.resolve_sel(sel, ctx).first().copied()),
            _ => None,
        };
        let moves: Vec<MoveEv> = objs
            .iter()
            .filter(|o| self.is_live(**o))
            .map(|o| {
                let owner = self.obj(*o).owner;
                MoveEv {
                    obj: *o,
                    to: Zone::of_kind(to.zone, owner),
                    pos: to.position,
                    cause: MoveCause::Effect,
                    by: Some(ctx.controller),
                    etb: EtbInfo {
                        tapped: to.tapped,
                        counters: counters.clone(),
                        controller: if to.zone == ZoneKind::Battlefield && owners_control {
                            Some(owner)
                        } else if to.zone == ZoneKind::Battlefield {
                            Some(controller.unwrap_or(ctx.controller))
                        } else {
                            None
                        },
                        face_down: if to.face_down {
                            Some(KeywordKind::Morph)
                        } else {
                            None
                        },
                        transformed: to.transformed,
                        attacking: attack,
                        with_mods: with_mods.clone(),
                        attach_to: attach_to.flatten(),
                        attach_specified: attach_to.is_some(),
                        ..Default::default()
                    },
                    source: ctx.source,
                }
            })
            .collect();
        self.move_objects(moves).into_iter().flatten().collect()
    }

    /// Default attack target for "put onto the battlefield attacking": the defending
    /// player of the source if it's attacking, else a defending player (CR 508.4).
    fn attack_target_for_new_attacker(&mut self, ctx: &Ctx) -> Option<Entity> {
        let combat = self.combat.as_ref()?;
        if let Some(src) = ctx.source {
            if let Some(t) = combat.attack_target(src) {
                return Some(t);
            }
        }
        // CR 508.4: otherwise its controller chooses what it's attacking.
        crate::combat::choose_attack_target_for_new_attacker(self, ctx.controller)
    }

    /// Determines the mana types produced by an AddMana effect (CR 106).
    pub fn produce_mana(&mut self, p: PlayerId, mana: &ManaProduction, ctx: &Ctx) -> Vec<ManaType> {
        match mana {
            ManaProduction::Fixed(v) => v.clone(),
            ManaProduction::Amount(t, n) => vec![*t; self.eval_value(n, ctx).max(0) as usize],
            ManaProduction::AnyOneColor(n) => {
                let k = self.eval_value(n, ctx).max(0) as usize;
                let c = self.choose_mana_color(
                    p,
                    ctx,
                    &[
                        ManaType::W,
                        ManaType::U,
                        ManaType::B,
                        ManaType::R,
                        ManaType::G,
                    ],
                );
                vec![c; k]
            }
            ManaProduction::AnyCombination(n) => {
                let k = self.eval_value(n, ctx).max(0) as usize;
                self.choose_mana_combination(
                    p,
                    ctx,
                    &[
                        ManaType::W,
                        ManaType::U,
                        ManaType::B,
                        ManaType::R,
                        ManaType::G,
                    ],
                    k,
                )
            }
            ManaProduction::CombinationOf(opts, n) => {
                let k = self.eval_value(n, ctx).max(0) as usize;
                self.choose_mana_combination(p, ctx, opts, k)
            }
            ManaProduction::OneOf(opts) => vec![self.choose_mana_color(p, ctx, opts)],
            ManaProduction::OneOfOrChosenColor(opts) => {
                let mut u = opts.clone();
                if let Some(c) = self
                    .source_choices(ctx)
                    .and_then(|c| c.color)
                    .map(ManaType::from_color)
                {
                    if !u.contains(&c) {
                        u.push(c);
                    }
                }
                vec![self.choose_mana_color(p, ctx, &u)]
            }
            ManaProduction::ChosenColor(n) => {
                // CR 607.5a: an undefined choice produces nothing.
                let k = self.eval_value(n, ctx).max(0) as usize;
                match self.source_choices(ctx).and_then(|c| c.color) {
                    Some(c) => vec![ManaType::from_color(c); k],
                    None => vec![],
                }
            }
            ManaProduction::CouldProduce(f) => {
                let types = crate::mana_abilities::types_could_produce(self, f, ctx);
                if types.is_empty() {
                    vec![]
                } else {
                    vec![self.choose_mana_color(p, ctx, &types)]
                }
            }
            ManaProduction::CouldProduceColor(f) => {
                // CR 106.7: colorless isn't a color.
                let types: Vec<ManaType> = crate::mana_abilities::types_could_produce(self, f, ctx)
                    .into_iter()
                    .filter(|t| *t != ManaType::C)
                    .collect();
                if types.is_empty() {
                    vec![]
                } else {
                    vec![self.choose_mana_color(p, ctx, &types)]
                }
            }
            ManaProduction::ManaCostOf(sel) => {
                // CR 106.8–106.11: hybrid symbols let the player choose a half; Phyrexian
                // symbols add one mana of their color; generic and snow symbols add
                // colorless mana.
                let symbols: Vec<crate::mana::ManaSymbol> = self
                    .eval_sel_objects(sel, ctx)
                    .first()
                    .and_then(|o| self.obj(*o).chars.mana_cost.clone())
                    .map(|m| m.symbols.to_vec())
                    .unwrap_or_default();
                let mut out = Vec::new();
                for s in symbols {
                    match s {
                        crate::mana::ManaSymbol::TwoHybrid(c) => {
                            let t = self.choose_mana_color(
                                p,
                                ctx,
                                &[ManaType::from_color(c), ManaType::C],
                            );
                            // The generic half {2} adds two colorless mana.
                            let n = if t == ManaType::C { 2 } else { 1 };
                            out.extend(std::iter::repeat_n(t, n));
                        }
                        other => {
                            for unit in crate::mana_abilities::symbol_units(other) {
                                out.push(self.choose_mana_color(p, ctx, &unit));
                            }
                        }
                    }
                }
                out
            }
            ManaProduction::DoubleUnspent => {
                // CR 701.10f: add as much of each type as the player already has.
                let pool = &self.player(p).mana_pool;
                ManaType::ALL
                    .iter()
                    .flat_map(|t| std::iter::repeat_n(*t, pool.count(*t)))
                    .collect()
            }
            ManaProduction::AnyColorAmong(f) => {
                let mut cs = ColorSet::NONE;
                for o in self.objects_matching(f, ctx) {
                    cs = cs.union(self.obj(o).chars.colors);
                }
                let types: Vec<ManaType> = cs.iter().map(ManaType::from_color).collect();
                if types.is_empty() {
                    vec![]
                } else {
                    vec![self.choose_mana_color(p, ctx, &types)]
                }
            }
            ManaProduction::CommanderIdentity => {
                // CR 903.4f: undefined without a commander; no mana.
                let types = crate::mana_abilities::commander_identity_types(self, p);
                if types.is_empty() {
                    vec![]
                } else {
                    vec![self.choose_mana_color(p, ctx, &types)]
                }
            }
            // CR 106.12a: one mana of any type the triggering mana ability produced.
            ManaProduction::AnyTypeProduced | ManaProduction::TypeProduced => {
                let types = produced_types(ctx);
                if types.is_empty() {
                    vec![]
                } else {
                    vec![self.choose_mana_color(p, ctx, &types)]
                }
            }
        }
    }

    /// CR 607.2c, 607.1d: objects an ability created or put onto the battlefield are
    /// linked to that ability of its source ("created with ~", "put onto the battlefield
    /// with ~").
    pub(crate) fn link_to_creator(&mut self, ctx: &Ctx, objs: &[ObjectId]) {
        let Some(src) = ctx.source else { return };
        if !self.is_live(src) {
            return;
        }
        for o in objs {
            if *o == src || !self.is_live(*o) {
                continue;
            }
            self.objects[src.0 as usize]
                .linked
                .entry(ctx.link)
                .or_default()
                .push(*o);
            self.objects[o.0 as usize].created_by = Some((src, ctx.link));
        }
    }

    /// Chooses the type of each of `k` mana "in any combination of" `opts`. A pending
    /// payment's hint names one type per unit, so each hinted type is used once.
    fn choose_mana_combination(
        &mut self,
        p: PlayerId,
        ctx: &Ctx,
        opts: &[ManaType],
        k: usize,
    ) -> Vec<ManaType> {
        let mut hint = self.mana_hint.clone().unwrap_or_default();
        let mut out = Vec::with_capacity(k);
        for _ in 0..k {
            if let Some(i) = hint.iter().position(|t| opts.contains(t)) {
                out.push(hint.remove(i));
                continue;
            }
            let saved = self.mana_hint.take();
            out.push(self.choose_mana_color(p, ctx, opts));
            self.mana_hint = saved;
        }
        out
    }

    fn choose_mana_color(&mut self, p: PlayerId, ctx: &Ctx, opts: &[ManaType]) -> ManaType {
        if opts.len() == 1 {
            return opts[0];
        }
        // A pending payment may hint which type is needed.
        if let Some(hint) = self
            .mana_hint
            .as_ref()
            .and_then(|h| h.iter().find(|t| opts.contains(t)).copied())
        {
            return hint;
        }
        let labels = opts.iter().map(|t| format!("{t:?}")).collect();
        let i = match self.ask(
            p,
            Decision::ChooseOption {
                source: ctx.source,
                prompt: "Choose mana type".into(),
                options: labels,
            },
        ) {
            Answer::Index(i) if i < opts.len() => i,
            _ => 0,
        };
        opts[i]
    }
}

/// The distinct types of mana the triggering mana ability produced (CR 106.12a).
pub fn produced_types(ctx: &Ctx) -> Vec<ManaType> {
    let mut types: Vec<ManaType> = Vec::new();
    for t in ctx.event.iter().flat_map(|e| e.mana.iter()) {
        if !types.contains(t) {
            types.push(*t);
        }
    }
    types
}

/// A [`PlayerRef`] that always refers to a specific player (locked in at resolution).
pub fn player_const(p: PlayerId) -> PlayerRef {
    PlayerRef::Player(p)
}

/// A player filter matching exactly one player.
pub fn player_filter_const(p: PlayerId) -> PlayerFilter {
    PlayerFilter::Is(p)
}

/// Replaces `Filter::In(..)` parts of a locked replacement's event filter with `Any`
/// (the lock already restricts the effect to those objects).
fn unlock_replacement_def(d: &ReplacementDef) -> ReplacementDef {
    fn unlock(f: &Filter) -> Filter {
        match f {
            Filter::In(_) => Filter::Any,
            Filter::And(v) => Filter::and(v.iter().map(unlock).collect()),
            other => other.clone(),
        }
    }
    let mut d = d.clone();
    match &mut d.event {
        ReplacementEvent::Destroy(f)
        | ReplacementEvent::Dies(f)
        | ReplacementEvent::EntersBattlefield(f) => *f = unlock(f),
        ReplacementEvent::ZoneChange { filter, .. } => *filter = unlock(filter),
        _ => {}
    }
    d
}

/// The filter selecting the objects a restriction applies to.
fn restriction_object_filter(r: &mut Restriction) -> Option<&mut Filter> {
    match r {
        Restriction::CantAttack(f)
        | Restriction::CantBlock(f)
        | Restriction::CantAttackOrBlock(f)
        | Restriction::MustAttack(f)
        | Restriction::MustBlock(f)
        | Restriction::MustBeBlocked(f)
        | Restriction::MustBeBlockedByAll(f)
        | Restriction::CantBeBlocked(f)
        | Restriction::DoesntUntap(f)
        | Restriction::CantBeCountered(f)
        | Restriction::CantBeSacrificed(f)
        | Restriction::CantBeRegenerated(f)
        | Restriction::SourceDamageCantBePrevented(f)
        | Restriction::AttackDespiteDefender(f)
        | Restriction::Goaded(f)
        | Restriction::DamageByToughness(f) => Some(f),
        Restriction::CantBeTargeted { what, .. } => Some(what),
        _ => None,
    }
}

/// The player filter of a restriction on players.
fn restriction_player_filter(r: &mut Restriction) -> Option<&mut PlayerFilter> {
    match r {
        Restriction::CantGainLife(f)
        | Restriction::CantLoseLife(f)
        | Restriction::CantLoseGame(f)
        | Restriction::CantWinGame(f)
        | Restriction::CantSearch(f)
        | Restriction::SorcerySpeedOnly(f)
        | Restriction::CantPlayLands(f)
        | Restriction::MaxDrawsPerTurn(f, _)
        | Restriction::MaxSpellsPerTurn(f, _)
        | Restriction::CantPlayLandCards { who: f, .. } => Some(f),
        Restriction::CantCast { who, .. } => Some(who),
        _ => None,
    }
}

fn filter_references_specific(f: &Filter) -> bool {
    match f {
        Filter::In(_) | Filter::Source | Filter::AttachedToSource => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(filter_references_specific),
        _ => false,
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        let mut end = n;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

pub fn describe_cost(c: &Cost) -> String {
    let mut parts = Vec::new();
    if let Some(m) = &c.mana {
        parts.push(m.to_string());
    }
    for p in &c.parts {
        parts.push(format!("{p:?}"));
    }
    parts.join(", ")
}

#[allow(dead_code)]
fn unused(_: Event) {}
