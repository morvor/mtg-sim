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
        // Each instruction is a separate action: events it causes form their own batch for
        // "one or more" triggers (CR 603.2c, 608.2c), and those of the instructions before
        // it are checked for triggers before it happens (CR 603.2, 603.10).
        self.action_boundary();
        if self.dirty {
            self.recompute();
        }
        if ctx.entering.is_some() && self.effect_on_entering_object(e, ctx) {
            return;
        }
        if crate::trigger_timing::is_sequencing(e) {
            self.exec_effect(e, ctx);
        } else {
            self.atomically(|g| g.exec_effect(e, ctx));
        }
    }

    /// Performs one effect (see [`Game::exec`]).
    fn exec_effect(&mut self, e: &Effect, ctx: &mut Ctx) {
        match e {
            Effect::Noop => {}
            Effect::Seq(v) => {
                // "Create a [token] and a [token]" is one instruction that the compiler
                // splits into one creation per kind: the tokens enter at the same time, as
                // one batch of events (CR 603.2c, 608.2c). The compiler gives separate
                // creation sentences ("Create A. Then create B.") the same shape, but no
                // card prints creation sentences with nothing else between or around them,
                // and a sequence with any other instruction keeps one batch per element.
                // Being one event, it's also checked for triggers as a whole (`exec` runs
                // it atomically, see `trigger_timing`).
                let together = crate::trigger_timing::creates_tokens_together(v);
                if together {
                    self.end_event_batch();
                    self.batch_hold += 1;
                }
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
                if together {
                    self.batch_hold -= 1;
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
                    // "If it's a permanent card, you may put it onto the battlefield. If
                    // you do, ...", "Then if there are three or more collection counters
                    // on it, sacrifice it. If you do, ...": an instruction whose condition
                    // didn't hold wasn't done (also "... sacrifice ~. When you do, ...").
                    if matches!(**otherwise, Effect::Noop) {
                        ctx.prev_happened = false;
                    }
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
                    // CR 800.4f: a player who has left the game doesn't pay.
                    if crate::multiplayer::cant_pay(self, p) {
                        continue;
                    }
                    if !crate::entry_costs::can_pay(self, p, cost, ctx) {
                        continue;
                    }
                    let pays = self.ask_yes_no(
                        p,
                        ctx.source,
                        &format!("Pay {}?", describe_cost(cost)),
                        false,
                    );
                    if pays && crate::entry_costs::pay(self, p, cost, ctx) {
                        paid = true;
                        break;
                    }
                    if !pays && matches!(**then, Effect::Noop) {
                        // CR 732.6: declining the [B] of "[A] unless [B]".
                        crate::shortcuts::declined_unless(self);
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
                // CR 608.2f: an action on several objects happens to all of them at once.
                crate::simultaneous::for_each_object(self, items, *var, effect, ctx);
            }
            Effect::ForEachPlayer { who, effect } => {
                let players = self.eval_players(who, ctx);
                // CR 101.4, 608.2e–f: what several players do at the same time.
                crate::simultaneous::for_each_player(self, players, effect, ctx);
            }
            Effect::AsPlayer { who, effect } => {
                if let Some(p) = self.eval_player(who, ctx) {
                    let saved = ctx.controller;
                    let saved_resolving = ctx.resolving_controller;
                    ctx.resolving_controller.get_or_insert(saved);
                    ctx.controller = p;
                    self.exec(effect, ctx);
                    ctx.controller = saved;
                    ctx.resolving_controller = saved_resolving;
                }
            }
            Effect::Repeat { times, effect } => {
                let n = self.eval_value(times, ctx).max(0);
                for _ in 0..n {
                    self.exec(effect, ctx);
                }
            }
            Effect::RepeatProcess { body } => crate::repeat_process::run(self, body, ctx),
            Effect::RepeatThisProcess => crate::repeat_process::request(ctx),
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
                let res = self.destroy_all_by(
                    objs.clone(),
                    crate::event_causes::Cause::of(ctx),
                    *no_regen,
                );
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
                // "... can't cause you to sacrifice or exile [permanents]" (CR 701.21).
                let cause = crate::rule_statics::sacrifice_causes::cause_of(ctx);
                let moves: Vec<MoveEv> = objs
                    .iter()
                    .filter(|o| self.is_live(**o))
                    .filter(|o| {
                        cause.as_ref().is_none_or(|c| {
                            !crate::rule_statics::sacrifice_causes::forbidden(self, **o, c, true)
                        })
                    })
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
                // "Exile a creature card from your graveyard. If you do, ...": whether
                // anything was exiled (as for moving it, CR 608.2c).
                ctx.prev_happened = !res.is_empty();
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
                // What makes them sacrifice (CR 701.21; see `rule_statics::sacrifice_causes`).
                let cause = crate::rule_statics::sacrifice_causes::cause_of(ctx);
                self.apnap_round(requests, |g, p, ()| {
                    let mut pctx = rctx.clone();
                    pctx.iter_player = Some(p);
                    let cands: Vec<ObjectId> = g
                        .objects_matching(filter, &pctx)
                        .into_iter()
                        .filter(|o| {
                            g.obj(*o).controller == p && !g.sacrifice_forbidden(*o, cause.as_ref())
                        })
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
                // "Sacrifice ~": the ability's controller sacrifices its source, which they
                // can't do if another player controls it now (CR 701.21a).
                let own_source = matches!(what, Sel::This);
                let objs = self.resolve_objects(what, ctx);
                let cause = crate::rule_statics::sacrifice_causes::cause_of(ctx);
                // Sacrificed at the same time (CR 101.4).
                let what: Vec<(ObjectId, PlayerId)> = objs
                    .into_iter()
                    .filter(|o| self.is_live(*o))
                    .filter(|o| !self.sacrifice_forbidden(*o, cause.as_ref()))
                    .filter(|o| !own_source || self.obj(*o).controller == ctx.controller)
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
                let objs: Vec<ObjectId> = self
                    .resolve_objects(what, ctx)
                    .into_iter()
                    .filter_map(|o| self.found_after_move(Entity::Object(o), ctx).object())
                    .collect();
                let res = self.move_to_destination(objs, to, ctx);
                // CR 712.21c, 730.3c: a melded or merged permanent became several cards.
                let res = crate::merge::found_all(self, res);
                if to.zone == ZoneKind::Battlefield {
                    self.link_to_creator(ctx, &res);
                }
                ctx.prev_affected = res.iter().map(|o| Entity::Object(*o)).collect();
                // "... return it to the battlefield. If you do, ...": whether it moved
                // (it may not, CR 400.7, 712.14a).
                ctx.prev_happened = !res.is_empty();
                ctx.set_var(vars::IT, res.into_iter().map(Entity::Object).collect());
            }
            Effect::Tap { what } => {
                let mut tapped = Vec::new();
                for o in self.resolve_objects(what, ctx) {
                    if self.tap(o) {
                        tapped.push(Entity::Object(o));
                    }
                }
                ctx.set_var(vars::TAPPED, tapped);
            }
            Effect::Untap { what } => {
                for o in self.resolve_objects(what, ctx) {
                    self.untap(o);
                }
            }
            Effect::DealDamage { source, amount, to } => {
                // "Each creature you control deals damage equal to its power to ...": every
                // one of those objects deals its own damage, all at the same time (CR
                // 120.2); the amount is evaluated for each of them (`vars::AFFECTED`).
                let multi = matches!(source, Sel::All(_) | Sel::Union(_));
                let srcs: Vec<ObjectId> = if multi {
                    self.resolve_objects(source, ctx)
                } else {
                    self.damage_source(source, ctx).into_iter().collect()
                };
                let recipients = self.resolve_sel(to, ctx);
                if !srcs.is_empty() {
                    let mut evs = Vec::new();
                    for &src in &srcs {
                        // Only the several-sources form binds "its" (the compiler reads it
                        // as `vars::AFFECTED` there); a single source leaves the variables
                        // as they are.
                        let saved = multi
                            .then(|| ctx.vars.insert(vars::AFFECTED, vec![Entity::Object(src)]));
                        let n = self.eval_value(amount, ctx).max(0) as u32;
                        match saved {
                            Some(Some(v)) => {
                                ctx.vars.insert(vars::AFFECTED, v);
                            }
                            Some(None) => {
                                ctx.vars.remove(&vars::AFFECTED);
                            }
                            None => {}
                        }
                        evs.extend(recipients.iter().map(|r| (src, *r, n)));
                    }
                    let before = self.events.len();
                    self.deal_damage_batch(evs, false);
                    self.record_damaged(&srcs, before, ctx);
                    // "The damage dealt this way": the total actually dealt to all the
                    // recipients, as modified by replacement and prevention (CR 120.4b).
                    ctx.prev_value = self.events[before.min(self.events.len())..]
                        .iter()
                        .map(|e| match e {
                            Event::Damage { source, amount, .. } if srcs.contains(source) => {
                                *amount as i64
                            }
                            _ => 0,
                        })
                        .sum();
                    // "The excess damage dealt this way" (CR 120.10).
                    let from = before.min(self.events.len());
                    let excess = crate::excess_damage::excess_in(&self.events[from..]);
                    ctx.nums.insert(vars::EXCESS, excess);
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
                    self.record_damaged(&[src], before, ctx);
                }
            }
            Effect::Fight { a, b } => {
                // CR 701.14
                let fighters = |g: &mut Game, s: &Sel, ctx: &mut Ctx| -> Vec<ObjectId> {
                    let v = g.resolve_objects(s, ctx);
                    v.into_iter()
                        .filter(|o| g.is_live(*o) && g.obj(*o).is_creature())
                        .collect()
                };
                let (a, b) = match (a, b) {
                    // "Choose two target creatures ... Those creatures fight each other."
                    // (one instance of the word "target"): the two fight each other; if
                    // either is an illegal target, no damage is dealt (CR 701.14b).
                    (Sel::Target(x), Sel::Target(y)) if x == y => {
                        match fighters(self, a, ctx).as_slice() {
                            [a, b] => (Some(*a), Some(*b)),
                            _ => (None, None),
                        }
                    }
                    _ => (
                        fighters(self, a, ctx).first().copied(),
                        fighters(self, b, ctx).first().copied(),
                    ),
                };
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
                let mut placed = 0;
                let mut got = Vec::new();
                for t in self.resolve_sel(what, ctx) {
                    let t = self.found_after_move(t, ctx);
                    let n = self.put_counters(t, kind, k, crate::event_causes::CounterPut::of(ctx));
                    if n > 0 {
                        got.push(t);
                    }
                    placed += n;
                }
                // "Put a coin counter on this artifact. When you do, ..." (CR 603.12):
                // whether any counter was put.
                ctx.prev_happened = placed > 0;
                // "Put a quest counter on this enchantment. When you do, if it has four or
                // more quest counters on it, ..." (Earthbender Ascension): "it" is what got
                // the counters.
                if !got.is_empty() {
                    ctx.set_var(vars::IT, got);
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
                        // N counters of the kinds the controller chooses.
                        None => {
                            total += crate::counter_rules::remove_chosen_counters(
                                self,
                                t,
                                None,
                                k,
                                ctx.controller,
                                ctx.source,
                            )
                        }
                    }
                }
                ctx.prev_value = total as i64;
                // "Remove a counter from it. If you do, …" (CR 608.2c).
                ctx.prev_happened = total > 0;
            }
            Effect::ChooseCounterKind { from, then } => {
                if let Some(e) =
                    crate::counter_rules::with_chosen_counter_kind(self, from, then, ctx)
                {
                    self.exec(&e, ctx);
                }
            }
            Effect::RemoveCountersUpTo { what, kind, max } => {
                let max = max.as_ref().map(|v| self.eval_value(v, ctx).max(0) as u32);
                let mut total = 0;
                for t in self.resolve_sel(what, ctx) {
                    total += crate::counter_rules::remove_up_to_counters(
                        self,
                        t,
                        kind.as_ref(),
                        max,
                        ctx.controller,
                        ctx.source,
                    );
                }
                ctx.prev_value = total as i64;
                ctx.prev_happened = total > 0;
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
                        total += crate::counter_rules::move_counters_by(
                            self, from, to, &k, count, ctx.source,
                        );
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
                        total += crate::counter_rules::put_counters_of(
                            self,
                            from,
                            t,
                            kind.as_deref(),
                            ctx.source,
                        );
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
                    .filter_map(|o| self.found_after_move(Entity::Object(o), ctx).object())
                    .filter(|o| self.is_live(*o))
                    .collect();
                if objs.is_empty() {
                    return;
                }
                // CR 608.2h: P/T values are determined once, as the effect is created. One
                // that depends on each affected object ("becomes an artifact creature with
                // power and toughness each equal to its mana value") is determined for each
                // of them (`vars::AFFECTED`, as for static abilities).
                let per_object = mods.iter().any(|m| {
                    let computed = |v: &Value| !matches!(v, Value::Const(_));
                    match m {
                        Modification::ModifyPT(p, t) => computed(p) || computed(t),
                        Modification::SetPT(p, t) => {
                            p.as_ref().is_some_and(computed) || t.as_ref().is_some_and(computed)
                        }
                        _ => false,
                    }
                });
                let ts = self.new_timestamp();
                let parts: Vec<(Option<ObjectId>, Vec<Modification>)> = if per_object {
                    objs.iter()
                        .map(|o| {
                            let mut c = ctx.clone();
                            c.set_var(vars::AFFECTED, vec![Entity::Object(*o)]);
                            (Some(*o), self.fix_mods(mods, &c))
                        })
                        .collect()
                } else {
                    let fixed = self.fix_mods(mods, ctx);
                    // CR 612.5: an exchange of text boxes gives each object the other's
                    // text.
                    match crate::text_change::exchange_mods(self, &objs, &fixed) {
                        Some(v) => v.into_iter().map(|(o, m)| (Some(o), m)).collect(),
                        None => vec![(None, fixed)],
                    }
                };
                let first = self.effects.len();
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
                // CR 113.11: not for an object that can't have an ability it adds.
                crate::kw::cant_have::drop_ungrantable_keywords(self, first);
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
                    | ReplacementAction::PreventAndThen(Some(v), _)
                    | ReplacementAction::RedirectNext(_, v) => {
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
                // ("two target creatures" select both from one target slot). Whether it
                // happened is what "If you do" / "If you don't or can't make an exchange"
                // ask about (CR 701.12b: between one player's permanents it does nothing).
                ctx.prev_happened = false;
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
                    ctx.prev_happened = true;
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
                    let tc = TokenCreate {
                        chars: crate::tokens::token_characteristics_in(self, spec, ctx),
                        card: crate::tokens::predefined_card(spec),
                        tapped: *tapped,
                        attacking: None,
                        copy_of: None,
                        copy_exceptions: vec![],
                    };
                    created.extend(self.create_tokens_maybe_attacking(p, tc, n, *attacking, ctx));
                }
                self.link_to_creator(ctx, &created);
                ctx.prev_value = created.len() as i64;
                ctx.set_var(
                    vars::CREATED,
                    created.into_iter().map(Entity::Object).collect(),
                );
            }
            Effect::CreateTokenWithPT {
                spec,
                power,
                toughness,
                count,
                controller,
                tapped,
                attacking,
            } => {
                // CR 107.3c: the numbers are determined as the tokens are created.
                let mut spec = spec.clone();
                spec.power = Some(self.eval_value(power, ctx) as i32);
                spec.toughness = Some(self.eval_value(toughness, ctx) as i32);
                let create = Effect::CreateToken {
                    spec,
                    count: count.clone(),
                    controller: controller.clone(),
                    tapped: *tapped,
                    attacking: *attacking,
                };
                self.exec(&create, ctx);
            }
            Effect::CreateTokenAttached {
                spec,
                count,
                controller,
                to,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                // What they enter attached to: each of the objects "it" is (a melded
                // permanent returned as two cards, Not Dead After All's ruling); `None` if
                // it's undefined (CR 303.4i).
                let mut attach: Vec<Option<Entity>> =
                    self.resolve_sel(to, ctx).into_iter().map(Some).collect();
                if attach.is_empty() {
                    attach.push(None);
                }
                let players = self.eval_players(controller, ctx);
                let mut created = Vec::new();
                for p in players {
                    for a in &attach {
                        let tc = TokenCreate {
                            chars: crate::tokens::token_characteristics_in(self, spec, ctx),
                            card: crate::tokens::predefined_card(spec),
                            tapped: false,
                            attacking: None,
                            copy_of: None,
                            copy_exceptions: vec![],
                        };
                        created.extend(self.create_tokens_attached(p, tc, n, ctx.source, *a));
                    }
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
                let sources = crate::copy_rules::token_copy_sources(self, of, ctx);
                let players = self.eval_players(controller, ctx);
                let fixed = self.fix_mods(mods, ctx);
                let mut created = Vec::new();
                for p in players {
                    for s in &sources {
                        let tc = TokenCreate {
                            chars: self.obj(*s).copiable.clone(),
                            card: self.obj(*s).card.clone(),
                            tapped: *tapped,
                            attacking: None,
                            copy_of: Some(*s),
                            copy_exceptions: fixed.clone(),
                        };
                        created
                            .extend(self.create_tokens_maybe_attacking(p, tc, n, *attacking, ctx));
                    }
                }
                ctx.set_var(
                    vars::CREATED,
                    created.into_iter().map(Entity::Object).collect(),
                );
            }
            Effect::CounterSpell { what } => {
                let mut any = false;
                let mut moved = Vec::new();
                for o in self.resolve_objects(what, ctx) {
                    if self.counter_by(o, crate::event_causes::Cause::of(ctx)) {
                        any = true;
                        // CR 400.7j: other parts of the effect can find the countered card
                        // in the public zone it moved to ("exile it instead ... You may
                        // cast that card ...").
                        let now = self.current(o);
                        if now != o
                            && matches!(self.obj(now).zone, Zone::Graveyard(_) | Zone::Exile)
                        {
                            moved.push(Entity::Object(now));
                        }
                    }
                }
                ctx.prev_happened = any;
                if !moved.is_empty() {
                    ctx.set_var(vars::IT, moved);
                }
            }
            Effect::CopySpell {
                what,
                count,
                new_targets,
            } => {
                let n = self.eval_value(count, ctx).max(0) as u32;
                let mut copies = vec![];
                for o in self.resolve_objects(what, ctx) {
                    for _ in 0..n {
                        copies.extend(crate::copy::copy_spell(
                            self,
                            o,
                            ctx.controller,
                            *new_targets,
                        ));
                    }
                }
                // CR 405.3: the copies are put on the stack at once, in the order
                // their controller chooses.
                crate::copy::order_copies(self, ctx.controller, &copies);
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
                crate::copy::become_copy(self, what, of, duration, &[], ctx);
            }
            Effect::BecomeCopyExcept {
                what,
                of,
                duration,
                exceptions,
            } => {
                crate::copy::become_copy(self, what, of, duration, exceptions, ctx);
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
                // "Attach ~ to a creature you control. If you do, ...": whether anything
                // became attached.
                let mut any = false;
                if let Some(t) = self.resolve_sel(to, ctx).into_iter().next() {
                    for o in objs {
                        any |= self.attach(o, t);
                    }
                }
                ctx.prev_happened = any;
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
                let mut turned = Vec::new();
                for o in self.resolve_objects(what, ctx) {
                    if crate::facedown::turn_face_down(self, o) {
                        turned.push(Entity::Object(o));
                    }
                }
                ctx.prev_happened = !turned.is_empty();
                ctx.set_var(vars::TURNED_FACE_DOWN, turned);
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
                    // Only cards can be discarded (CR 701.9a, 108.2): a token returned to a
                    // hand isn't a card (CR 111.6) and can't leave it (CR 111.8).
                    let hand: Vec<ObjectId> = g
                        .player(p)
                        .hand
                        .clone()
                        .into_iter()
                        .filter(|c| g.obj(*c).is_card() && g.matches(*c, filter, rctx))
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
                ctx.set_var(crate::discard_rules::DISCARDED, discarded.clone());
                ctx.set_var(vars::IT, discarded);
            }
            Effect::DiscardHand { who } => {
                let mut discarded = Vec::new();
                for p in self.eval_players(who, ctx) {
                    for c in self.player(p).hand.clone() {
                        if let Some(n) = self.discard(p, c, ctx.source) {
                            discarded.push(Entity::Object(n));
                        }
                    }
                }
                // "for each card discarded this way". A hand of no cards is discarded too
                // ("you may discard your hand. If you do, ..."): it still happened.
                ctx.prev_value = discarded.len() as i64;
                ctx.set_var(crate::discard_rules::DISCARDED, discarded.clone());
                ctx.set_var(vars::IT, discarded);
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
                // CR 810.9d: on a team sharing a life total, only one member is affected.
                let players = self.eval_players(who, ctx);
                for p in crate::multiplayer::two_headed::life_setters(self, players) {
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
                    crate::classes::set_level(self, s, *level);
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
                    self.put_counters(
                        Entity::Player(p),
                        kind,
                        k,
                        crate::event_causes::CounterPut::of(ctx),
                    );
                }
            }
            Effect::Scry { who, n } => {
                // CR 701.22c: players scrying at once do so at the same time.
                let k = self.eval_value(n, ctx).max(0) as u32;
                let players = self.eval_players(who, ctx);
                let look = crate::scry_rules::Look::Scry;
                crate::scry_rules::perform(self, &players, k, look, ctx.source);
                // CR 701.22b, 701.22d: scrying N > 0 always happens ("When you do, ...").
                ctx.prev_happened = k > 0 && !players.is_empty();
            }
            Effect::Surveil { who, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                let players = self.eval_players(who, ctx);
                let look = crate::scry_rules::Look::Surveil;
                crate::scry_rules::perform(self, &players, k, look, ctx.source);
                // CR 701.25c-d: surveilling N > 0 always happens, even with fewer cards in
                // the library ("When you do, ...").
                ctx.prev_happened = k > 0 && !players.is_empty();
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
                // The players who chose to search ("each player may search", a variable)
                // can be no one.
                if searchers.is_empty() && !matches!(who, PlayerRef::Var(_)) {
                    searchers.push(ctx.controller);
                }
                // CR 118.12b: "If you do" after a search checks whether the player
                // searched (one who can't search libraries doesn't), not whether they
                // found anything or took the additional actions.
                let searched = searchers.iter().any(|p| {
                    !self.player_restricted(*p, |r| matches!(r, Restriction::CantSearch(_)))
                });
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
                    // A rule that deals with the found cards instead ("they exile each
                    // card they find"): the rest of the effect still applies.
                    if !found.is_empty() && crate::kw::search_found(self, p, owner, &found) {
                        if *shuffle {
                            self.shuffle_library(owner);
                        }
                        continue;
                    }
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
                ctx.prev_happened = searched;
                ctx.set_var(vars::IT, all.into_iter().map(Entity::Object).collect());
            }
            Effect::SearchCards(spec) => crate::search_rules::perform(self, spec, ctx),
            Effect::Shuffle { who } => {
                for p in self.eval_players(who, ctx) {
                    self.shuffle_library(p);
                }
            }
            Effect::ShuffleInto { what } => {
                // Into their owners' libraries (CR 701.24c).
                let objs = self.resolve_objects(what, ctx);
                crate::shuffle_rules::shuffle_into(self, &objs, Vec::new(), ctx);
            }
            Effect::ShuffleIntoLibrary { what, library } => {
                let objs = self.resolve_objects(what, ctx);
                let libraries = self.eval_players(library, ctx);
                let moved: Vec<Entity> =
                    crate::shuffle_rules::shuffle_into(self, &objs, libraries, ctx)
                        .into_iter()
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
                let mut revealed = Vec::new();
                for p in self.eval_players(who, ctx) {
                    let hand = self.player(p).hand.clone();
                    crate::reveal::reveal_in(self, p, &hand, Some(ctx));
                    revealed.extend(hand.into_iter().map(Entity::Object));
                }
                // "If a card with the chosen name is revealed this way" (CR 701.20a).
                ctx.set_var(vars::REVEALED, revealed);
            }
            Effect::LookAtHand { who } => {
                // Looking gives the controller information only; the cards aren't
                // revealed and nothing else happens.
                let viewer = ctx.controller;
                for p in self.eval_players(who, ctx) {
                    let hand = self.player(p).hand.clone();
                    self.log(|g| {
                        let cards: Vec<String> = hand.iter().map(|c| g.describe(*c)).collect();
                        format!("{viewer} looks at {p}'s hand: {}", cards.join(", "))
                    });
                }
            }
            Effect::RevealUntil {
                who,
                filter,
                found_to,
                rest_to,
            } => {
                let players = self.eval_players(who, ctx);
                if let [p] = players[..] {
                    crate::library::reveal_until(self, p, filter, found_to, rest_to, ctx);
                } else {
                    // "Each opponent exiles cards from the top of their library until they
                    // exile a nonland card.": the cards found, and the others, of all of
                    // them. With no player (an illegal target player isn't affected, CR
                    // 608.2b; no opponent left), nothing happens.
                    let (mut found, mut rest) = (Vec::new(), Vec::new());
                    for p in players {
                        ctx.set_var(vars::IT, vec![]);
                        ctx.set_var(vars::REVEALED, vec![]);
                        crate::library::reveal_until(self, p, filter, found_to, rest_to, ctx);
                        found.extend(ctx.vars.get(&vars::IT).cloned().unwrap_or_default());
                        rest.extend(ctx.vars.get(&vars::REVEALED).cloned().unwrap_or_default());
                    }
                    ctx.set_var(vars::IT, found);
                    ctx.set_var(vars::REVEALED, rest);
                }
            }
            Effect::ExtraTurn { who } => {
                // CR 500.7: most recently created extra turn is taken first. With shared
                // team turns, the team takes it, once per team (CR 805.8).
                let who = self.eval_players(who, ctx);
                for p in crate::skip::once_per_team(self, who) {
                    self.extra_turns.push(p);
                }
            }
            Effect::ExtraTurnWith { who, at_start } => {
                let who = self.eval_players(who, ctx);
                for p in crate::skip::once_per_team(self, who) {
                    crate::skip::queue_extra_turn(self, p, ctx, at_start);
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
                let who = self.eval_players(who, ctx);
                for p in crate::skip::once_per_team(self, who) {
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
                let (controller, performer) = delayed_controller(ctx);
                let mut saved = crate::transform_rules::delayed_ctx(self, ctx);
                saved.resolving_controller = None;
                self.delayed_triggers.push(DelayedTrigger {
                    id,
                    source: ctx.source,
                    controller,
                    trigger: trigger.clone(),
                    body: (**body).clone(),
                    once: *once,
                    ctx: saved,
                    created_turn: self.turn.number,
                    created_step: Some(self.turn.step),
                    created_steps: self.turn.step_log.len(),
                    for_rest_of_game: false,
                    performer,
                });
            }
            Effect::NoteLinked { what, replace } => {
                crate::linked_notes::exec(self, what, *replace, ctx);
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
                let mut saved = ctx.clone();
                saved.resolving_controller = None;
                if saved.reflexive_parent.is_none() {
                    saved.reflexive_parent = self.resolving_ability(ctx).map(Box::new);
                }
                let (controller, performer) = delayed_controller(ctx);
                let mut body = (**body).clone();
                if let Some(p) = performer {
                    body = performed_by(body, p);
                }
                self.pending_triggers.push(PendingTrigger {
                    source: src.unwrap_or(ObjectId(0)),
                    controller,
                    ability,
                    event: ctx.event.clone().unwrap_or_default(),
                    source_lki: src.map(|s| Box::new(self.obj(s).chars.clone())),
                    saved: Some(saved),
                    body: Some(body),
                    order: self.trigger_order,
                });
            }
            Effect::AtNext { step, effect } => {
                self.flush_events();
                let id = self.new_effect_id();
                let (controller, performer) = delayed_controller(ctx);
                let mut saved = ctx.clone();
                saved.resolving_controller = None;
                self.delayed_triggers.push(DelayedTrigger {
                    id,
                    source: ctx.source,
                    controller,
                    trigger: TriggerCond::BeginningOf {
                        step: *step,
                        whose: PlayerRel::Any,
                    },
                    body: Body::effect((**effect).clone()),
                    once: true,
                    ctx: saved,
                    created_turn: self.turn.number,
                    created_step: Some(self.turn.step),
                    created_steps: self.turn.step_log.len(),
                    for_rest_of_game: false,
                    performer,
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
            Effect::KeepAndSacrificeRest {
                who,
                among,
                keep,
                up_to,
            } => {
                crate::apnap::keep_and_sacrifice_rest(self, who, among, keep, *up_to, ctx);
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
                // CR 601.3e: a choice of "a spell with [characteristics]" considers the
                // spell each card could become, and it's cast as such.
                let chosen = crate::spell_choice::choose_cards_to_cast(self, what, ctx);
                let objs = match &chosen {
                    Some(v) => v.iter().map(|(o, _)| *o).collect(),
                    None => self.resolve_objects(what, ctx),
                };
                for o in objs {
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
                    let only = chosen
                        .as_ref()
                        .and_then(|v| v.iter().find(|(x, _)| *x == o))
                        .map(|(_, f)| f.clone());
                    if let Ok(spell) = crate::casting::cast_during_resolution_as(
                        self,
                        p,
                        o,
                        method,
                        only.as_deref(),
                    ) {
                        cast.push(Entity::Object(spell));
                    }
                }
                // "You may cast ... If you do, ...": whether a spell was cast.
                ctx.prev_happened = !cast.is_empty();
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
            Effect::WithPlayTerms { terms, effect } => {
                // The permissions the effect gives come with the terms.
                let before = self.play_grants.len();
                self.exec(effect, ctx);
                for g in self.play_grants.iter_mut().skip(before) {
                    g.terms.merge(terms);
                }
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
            Effect::PreventDividedDamage { slot, duration } => {
                // Divisions are kept aligned with the targets that are still legal
                // (CR 608.2b; see `recheck_targets`).
                let targets = ctx.targets.get(*slot as usize).cloned().unwrap_or_default();
                let div = ctx.divided.get(*slot as usize).cloned().unwrap_or_default();
                for (i, t) in targets.into_iter().enumerate() {
                    let n = div.get(i).copied().unwrap_or(0);
                    if n == 0 {
                        continue;
                    }
                    let to = match t {
                        Entity::Player(p) => Sel::Players(PlayerRef::Player(p)),
                        Entity::Object(o) => Sel::All(Filter::Objects(vec![o])),
                    };
                    let shield = Effect::PreventDamage {
                        to,
                        amount: Some(Value::c(n as i32)),
                        duration: duration.clone(),
                        combat_only: false,
                    };
                    self.exec(&shield, ctx);
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
            Effect::TokensEnterWithCounters { counters, effect } => {
                crate::tokens::enter_with_counters(self, counters, effect, ctx)
            }
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
                let mut cands: Vec<ObjectId> = self
                    .objects_matching(filter, ctx)
                    .into_iter()
                    .filter(|o| !self.entering.contains(o))
                    .collect();
                // CR 723.4: a controlled player can't be made to choose cards from
                // outside the game.
                crate::player_control::visible_choices(self, p, &mut cands);
                let min = if *up_to { 0 } else { n.min(cands.len() as u32) };
                // CR 406.4: face-down exiled cards the player can't look at are chosen by
                // pile.
                let chosen = crate::zones::choose_objects(
                    self,
                    p,
                    ctx.source,
                    "Choose",
                    cands.clone(),
                    min,
                    n,
                );
                // "Choose any number of ... tokens you control with different names": the
                // objects chosen must have the relationship.
                let picked: Vec<Entity> = crate::target_groups::fit_together(
                    self,
                    filter,
                    chosen,
                    &cands,
                    min as usize,
                    ctx,
                )
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
                let mut out = Vec::new();
                for e in v {
                    let followed = self.follow_zone_change_trigger_object(e, ctx);
                    // CR 712.21c, 730.3c: the new object a melded or merged permanent
                    // became as it left the battlefield is each of its cards.
                    let found = match followed {
                        Entity::Object(o) if followed != e => crate::merge::found_objects(self, o)
                            .into_iter()
                            .map(Entity::Object)
                            .collect(),
                        _ => vec![followed],
                    };
                    for f in found {
                        if !out.contains(&f) {
                            out.push(f);
                        }
                    }
                }
                out
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

    /// CR 400.7j: an object an earlier part of the same effect moved to a public zone can
    /// be found by later parts of it ("Exile target creature and put two time counters on
    /// it"): the object it became, when that instruction recorded it.
    pub(crate) fn found_after_move(&self, e: Entity, ctx: &Ctx) -> Entity {
        let Entity::Object(o) = e else {
            return e;
        };
        if self.is_live(o) {
            return e;
        }
        let now = self.current(o);
        let moved = ctx
            .vars
            .get(&vars::IT)
            .is_some_and(|v| v.contains(&Entity::Object(now)));
        if moved && !matches!(self.obj(now).zone, Zone::Hand(_) | Zone::Library(_)) {
            Entity::Object(now)
        } else {
            e
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
    fn record_damaged(&mut self, srcs: &[ObjectId], before: usize, ctx: &mut Ctx) {
        let mut damaged: Vec<Entity> = Vec::new();
        for ev in &self.events[before.min(self.events.len())..] {
            if let Event::Damage {
                source,
                target: Entity::Object(o),
                amount,
                ..
            } = ev
            {
                if srcs.contains(source) && *amount > 0 && !damaged.contains(&Entity::Object(*o)) {
                    damaged.push(Entity::Object(*o));
                }
            }
        }
        ctx.prev_affected = damaged.clone();
        ctx.set_var(vars::DAMAGED, damaged);
    }

    /// The source of damage for an effect: the named object, or the resolving object's
    /// source (CR 120.2, 609.7). Uses last known information if it has left.
    pub(crate) fn damage_source(&mut self, sel: &Sel, ctx: &mut Ctx) -> Option<ObjectId> {
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
                // "gains all activated abilities of target creature until end of turn":
                // the abilities it has as the effect is created (CR 608.2h).
                Modification::AddAbilitiesOf { from, which } => {
                    crate::ability_grants::snapshot(self, from, which, ctx)
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
                // "Protection from each of your opponents" (CR 702.16k): protection from
                // players, who are determined as the effect is created; it doesn't follow
                // the permanent's controller.
                Modification::AddKeyword(k)
                    if k.kind == KeywordKind::Protection
                        && matches!(k.filter, Some(Filter::ControlledBy(PlayerRel::Opponent))) =>
                {
                    let opponents = self
                        .player_ids()
                        .into_iter()
                        .filter(|q| self.are_opponents(ctx.controller, *q))
                        .map(PlayerFilter::Is)
                        .collect();
                    let mut k = k.clone();
                    k.filter = Some(Filter::ControllerMatches(Box::new(PlayerFilter::Or(
                        opponents,
                    ))));
                    Modification::AddKeyword(k)
                }
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
                Modification::SetChosenColors => {
                    match ctx.source.and_then(|s| self.obj(s).choices.colors) {
                        Some(cs) => Modification::SetColors(cs),
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
                // "except it has this ability" (CR 707.9a): the resolving ability.
                // For a reflexive trigger, the ability that created it (CR 603.12).
                Modification::AddThisAbility => {
                    let ability = ctx
                        .reflexive_parent
                        .as_deref()
                        .cloned()
                        .or_else(|| self.resolving_ability(ctx));
                    match ability {
                        Some(a) => Modification::AddAbility(a),
                        None => m.clone(),
                    }
                }
                other => other.clone(),
            })
            .collect()
    }

    /// The activated or triggered ability on the stack that's resolving with `ctx`.
    fn resolving_ability(&self, ctx: &Ctx) -> Option<Ability> {
        let id = ctx.stack_obj?;
        match self.obj(id).stack.as_deref().map(|si| &si.kind) {
            Some(StackKind::Activated { ability, .. } | StackKind::Triggered { ability, .. }) => {
                Some(ability.clone())
            }
            _ => None,
        }
    }

    /// A restriction locked onto specific objects (see [`Self::lock_restriction_objects`])
    /// applies to exactly those objects: its filter named them through this resolution's
    /// targets or event ("target creature", "that creature"), which aren't available when
    /// the restriction is checked later, so it becomes "any object" (of the locked ones).
    fn fix_restriction(&self, r: &Restriction, ctx: &Ctx) -> Restriction {
        let mut r = r.clone();
        if let Some(f) = restriction_object_filter(&mut r) {
            self.fix_excluded_objects(f, ctx);
            if filter_references_specific(f) {
                *f = Filter::Any;
            } else {
                // "Creatures target player controls don't untap ...": the objects that
                // player controls, whichever they are later.
                *f = self.bind_target_players(f, ctx);
            }
        }
        // "Target creature blocks this creature this combat if able": both creatures are
        // the objects named as the effect began.
        // Likewise "target creature can't block this creature this turn".
        if let Restriction::MustBlockAttacker { attacker, .. } = &mut r {
            // The creature to be blocked is only named, not affected: the requirement
            // still refers to it if it has become an illegal target (CR 608.2b; Feral
            // Contest: "the second targeted creature is still affected by the blocking
            // restriction").
            if filter_references_specific(attacker) {
                let c2 = self.with_original_targets(ctx);
                *attacker = Filter::Objects(self.named_objects(attacker, &c2));
            }
        }
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

    /// "Creatures other than [specific objects]" (Intimidation Bolt: "other creatures
    /// can't attack this turn", other than its target) is a class of objects that can
    /// include objects arriving later (CR 611.2c); the excluded objects are those named as
    /// the effect begins.
    fn fix_excluded_objects(&self, f: &mut Filter, ctx: &Ctx) {
        match f {
            Filter::And(v) => {
                for x in v.iter_mut() {
                    if let Filter::Not(inner) = x {
                        if matches!(**inner, Filter::In(_)) {
                            **inner = Filter::Objects(self.named_objects(inner, ctx));
                        }
                    }
                }
            }
            Filter::Not(inner) if matches!(**inner, Filter::In(_)) => {
                **inner = Filter::Objects(self.named_objects(inner, ctx));
            }
            _ => {}
        }
    }

    /// Replaces "controlled by the target player" in a filter kept beyond this resolution
    /// with the player chosen as that target.
    fn bind_target_players(&self, f: &Filter, ctx: &Ctx) -> Filter {
        match f {
            Filter::ControlledBy(PlayerRel::Target(k)) => {
                let ps = ctx
                    .targets
                    .get(*k as usize)
                    .into_iter()
                    .flatten()
                    .filter_map(|e| match e {
                        Entity::Player(p) => Some(PlayerFilter::Is(*p)),
                        _ => None,
                    })
                    .collect();
                Filter::ControllerMatches(Box::new(PlayerFilter::Or(ps)))
            }
            Filter::And(v) => {
                Filter::And(v.iter().map(|x| self.bind_target_players(x, ctx)).collect())
            }
            Filter::Or(v) => {
                Filter::Or(v.iter().map(|x| self.bind_target_players(x, ctx)).collect())
            }
            other => other.clone(),
        }
    }

    /// Restrictions naming specific objects ("target creature can't block this turn")
    /// lock onto those objects.
    fn lock_restriction_objects(&self, r: &Restriction, ctx: &Ctx) -> Option<Vec<ObjectId>> {
        let mut r = r.clone();
        let f = restriction_object_filter(&mut r)?;
        self.fix_excluded_objects(f, ctx);
        if filter_references_specific(f) {
            Some(self.named_objects(f, ctx))
        } else {
            None
        }
    }

    /// `ctx` with each target slot left empty by illegal targets (CR 608.2b) refilled with
    /// the targets chosen for it, for wording that only names an illegal target.
    fn with_original_targets(&self, ctx: &Ctx) -> Ctx {
        let mut c = ctx.clone();
        let chosen = ctx
            .stack_obj
            .and_then(|s| self.obj(s).stack.as_deref())
            .map(|si| si.chosen.clone())
            .unwrap_or_default();
        if let [cm] = chosen.as_slice() {
            for (k, slot) in cm.targets.iter().enumerate() {
                if c.targets.len() == k {
                    c.targets.push(slot.clone());
                } else if let Some(t) = c.targets.get_mut(k).filter(|t| t.is_empty()) {
                    *t = slot.clone();
                }
            }
        }
        c
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
        let moves = self.destination_moves(objs, to, ctx);
        self.move_objects(moves).into_iter().flatten().collect()
    }

    /// The moves that put `objs` into `to` (see [`Self::move_to_destination`]), for
    /// moving them together with others at the same time.
    pub fn destination_moves(
        &mut self,
        objs: Vec<ObjectId>,
        to: &Destination,
        ctx: &mut Ctx,
    ) -> Vec<MoveEv> {
        let objs: Vec<ObjectId> = objs.into_iter().filter(|o| self.is_live(*o)).collect();
        if objs.is_empty() {
            return vec![];
        }
        // "Under its owner's control", "tapped", "with N counters", "your choice of the top
        // or bottom" ...: see `destinations.rs`.
        let dest = self.prepare_destination(to, ctx);
        let mut moves: Vec<MoveEv> = Vec::with_capacity(objs.len());
        for o in &objs {
            let owner = self.obj(*o).owner;
            let etb = self.destination_etb(&dest, owner, ctx);
            moves.push(MoveEv {
                obj: *o,
                to: dest.zone(owner),
                pos: dest.position(),
                cause: MoveCause::Effect,
                by: Some(ctx.controller),
                etb,
                source: ctx.source,
            });
        }
        moves
    }

    /// Creates `n` tokens for `p` (`spec`), "tapped and attacking" if `attacking`: as each
    /// token is created (including any more an effect such as Parallel Lives adds), its
    /// controller chooses what it's attacking (CR 508.4), by default what the source is
    /// attacking (see [`Self::attack_target_for_new_attacker`]).
    fn create_tokens_maybe_attacking(
        &mut self,
        p: PlayerId,
        mut spec: TokenCreate,
        n: u32,
        attacking: bool,
        ctx: &Ctx,
    ) -> Vec<ObjectId> {
        if !attacking {
            return self.create_tokens(p, spec, n, ctx.source);
        }
        let preferred = self
            .combat
            .as_ref()
            .and_then(|c| ctx.source.and_then(|src| c.attack_target(src)));
        let options = crate::combat::attack_target_options(self, preferred);
        spec.attacking = options.first().copied();
        let prev = std::mem::replace(&mut self.token_attack_options, options);
        let out = self.create_tokens(p, spec, n, ctx.source);
        self.token_attack_options = prev;
        out
    }

    /// What a creature put onto the battlefield attacking attacks when the effect doesn't
    /// say: its controller (`controller`) chooses (CR 508.4), by default what the source is attacking if
    /// it's attacking (Geist of Saint Traft's Angel needn't attack what Geist attacks).
    pub(crate) fn attack_target_for_new_attacker(
        &mut self,
        controller: PlayerId,
        ctx: &Ctx,
    ) -> Option<Entity> {
        let combat = self.combat.as_ref()?;
        let preferred = ctx.source.and_then(|src| combat.attack_target(src));
        crate::combat::choose_attack_target_preferring(self, controller, preferred)
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
        | Restriction::CantTurnFaceUp(f)
        | Restriction::CantBeCopied(f)
        | Restriction::SourceDamageCantBePrevented(f)
        | Restriction::AttackDespiteDefender(f)
        | Restriction::BlockAsThoughUntapped(f)
        | Restriction::Goaded(f)
        | Restriction::DamageByToughness(f)
        | Restriction::AssignsNoCombatDamage(f) => Some(f),
        Restriction::CantBeTargeted { what, .. } => Some(what),
        Restriction::MustAttackPlayer { attackers, .. }
        | Restriction::AttackAsThoughHaste { attackers, .. } => Some(attackers),
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
        Restriction::MustAttackPlayer { defender, .. }
        | Restriction::AttackAsThoughHaste {
            defender: Some(defender),
            ..
        } => Some(defender),
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

/// The controller of a delayed or reflexive triggered ability created now, and the player
/// who performs it when that's someone else: while another player performs part of the
/// resolving spell or ability ([`Effect::AsPlayer`], "they may pay {1}. If they do, they
/// draw a card at the beginning of the next end step"), the triggered ability is
/// controlled by the spell's or ability's controller (CR 603.7d–e, 603.12) and that
/// player performs it.
pub(crate) fn delayed_controller(ctx: &Ctx) -> (PlayerId, Option<PlayerId>) {
    match ctx.resolving_controller {
        Some(c) if c != ctx.controller => (c, Some(ctx.controller)),
        _ => (ctx.controller, None),
    }
}

/// `body` (of a triggered ability) performed by player `p` ("you" in it is `p`).
pub(crate) fn performed_by(mut body: Body, p: PlayerId) -> Body {
    let wrap = |e: &mut Effect| {
        let inner = std::mem::replace(e, Effect::Noop);
        *e = Effect::AsPlayer {
            who: PlayerRef::Player(p),
            effect: Box::new(inner),
        };
    };
    wrap(&mut body.effect);
    if let Some(m) = body.modal.as_mut() {
        for mode in &mut m.modes {
            wrap(&mut mode.effect);
        }
    }
    body
}
