//! The effect interpreter: performs [`Effect`]s as spells and abilities resolve
//! (CR 608.2c–h, 609, 610, 611).

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::mana::{Mana, ManaType};
use crate::object::*;
use crate::replacement::*;
use crate::types::*;

/// Resolution context saved with delayed triggers / stack objects.
pub type SavedCtx = Ctx;

impl Game {
    /// Executes an effect.
    pub fn exec(&mut self, e: &Effect, ctx: &mut Ctx) {
        if self.result.is_some() {
            return;
        }
        if self.dirty {
            self.recompute();
        }
        match e {
            Effect::Noop => {}
            Effect::Seq(v) => {
                for x in v {
                    self.exec(x, ctx);
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
                let yes = self.ask_yes_no(
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
            Effect::SetX { value } => {
                ctx.x = self.eval_value(value, ctx) as i32;
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
                        source: if *link { ctx.source } else { None },
                    })
                    .collect();
                let res: Vec<ObjectId> = self.move_objects(moves).into_iter().flatten().collect();
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
                for p in players {
                    let mut pctx = ctx.clone();
                    pctx.iter_player = Some(p);
                    let cands: Vec<ObjectId> = self
                        .objects_matching(filter, &pctx)
                        .into_iter()
                        .filter(|o| self.obj(*o).controller == p && !self.cant_be_sacrificed(*o))
                        .collect();
                    let k = n.min(cands.len() as u32);
                    let pick = self.ask_objects(
                        p,
                        ctx.source,
                        "Choose permanents to sacrifice",
                        cands,
                        k,
                        k,
                    );
                    for o in pick {
                        chosen.push((p, o));
                    }
                }
                for (p, o) in chosen {
                    if let Some(new) = self.sacrifice(o, p) {
                        all.push(Entity::Object(new));
                    }
                    ctx.prev_affected.push(Entity::Object(o));
                }
                ctx.prev_value = all.len() as i64;
                ctx.prev_happened = !all.is_empty();
                ctx.set_var(vars::IT, all);
            }
            Effect::SacrificeObjects { what } => {
                let objs = self.resolve_objects(what, ctx);
                let mut res = Vec::new();
                for o in objs {
                    if !self.is_live(o) {
                        continue;
                    }
                    let p = self.obj(o).controller;
                    if let Some(n) = self.sacrifice(o, p) {
                        res.push(Entity::Object(n));
                    }
                }
                ctx.prev_happened = !res.is_empty();
                ctx.set_var(vars::IT, res);
            }
            Effect::Move { what, to } => {
                let objs = self.resolve_objects(what, ctx);
                let res = self.move_to_destination(objs, to, ctx);
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
                    self.deal_damage_batch(evs, false);
                    ctx.prev_value = n as i64;
                }
            }
            Effect::DealDividedDamage { source, slot } => {
                if let Some(src) = self.damage_source(source, ctx) {
                    let targets = ctx.targets.get(*slot as usize).cloned().unwrap_or_default();
                    let div = ctx.divided.get(*slot as usize).cloned().unwrap_or_default();
                    // Divided among original targets; illegal targets were filtered, so
                    // match by position in the original choice where possible.
                    let evs: Vec<(ObjectId, Entity, u32)> = targets
                        .iter()
                        .enumerate()
                        .map(|(i, t)| (src, *t, div.get(i).copied().unwrap_or(0)))
                        .collect();
                    self.deal_damage_batch(evs, false);
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
                        self.deal_damage_batch(vec![(a, Entity::Object(a), pa)], false);
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
                        Some(kind) => total += self.remove_counters(t, kind, k),
                        None => {
                            let kinds: Vec<CounterKind> = match t {
                                Entity::Object(o) => self.obj(o).counters.keys().cloned().collect(),
                                Entity::Player(p) => {
                                    self.player(p).counters.keys().cloned().collect()
                                }
                            };
                            for kk in kinds {
                                total += self.remove_counters(t, &kk, k);
                            }
                        }
                    }
                }
                ctx.prev_value = total as i64;
            }
            Effect::Modify {
                what,
                mods,
                duration,
            } => {
                let objs: Vec<ObjectId> = self
                    .resolve_objects(what, ctx)
                    .into_iter()
                    .filter(|o| self.is_live(*o))
                    .collect();
                if objs.is_empty() {
                    return;
                }
                let fixed = self.fix_mods(mods, ctx);
                let id = self.new_effect_id();
                let ts = self.new_timestamp();
                self.effects.push(ContinuousEffect {
                    id,
                    source: ctx.source,
                    controller: ctx.controller,
                    timestamp: ts,
                    duration: duration.clone(),
                    affected: Affected::Objects(objs),
                    mods: fixed,
                    layer1: None,
                    created_turn: self.turn.number,
                });
                self.dirty = true;
            }
            Effect::AddRestriction {
                restriction,
                duration,
            } => {
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
                let remaining = match &def.action {
                    ReplacementAction::PreventAmount(v) => {
                        Some(self.eval_value(v, ctx).max(0) as u32)
                    }
                    _ => None,
                };
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
                let a = self
                    .resolve_objects(a, ctx)
                    .into_iter()
                    .find(|o| self.is_live(*o));
                let b = self
                    .resolve_objects(b, ctx)
                    .into_iter()
                    .find(|o| self.is_live(*o));
                if let (Some(a), Some(b)) = (a, b) {
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
                        card: None,
                        tapped: *tapped,
                        attacking: attack,
                        copy_of: None,
                        copy_exceptions: vec![],
                    };
                    created.extend(self.create_tokens(p, tc, n, ctx.source));
                }
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
                    crate::dfc::transform(self, o);
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
            Effect::RemoveFromCombat { what } => {
                for o in self.resolve_objects(what, ctx) {
                    crate::combat::remove_from_combat(self, o);
                }
            }
            Effect::Choose { who, kind } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                crate::choices::make_choice(self, p, kind, ctx);
            }

            // --- Players -----------------------------------------------------------
            Effect::Draw { who, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                let mut drawn = Vec::new();
                for p in self.eval_players(who, ctx) {
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
                for p in self.eval_players(who, ctx) {
                    let hand: Vec<ObjectId> = self
                        .player(p)
                        .hand
                        .clone()
                        .into_iter()
                        .filter(|c| self.matches(*c, filter, ctx))
                        .collect();
                    let k = k.min(hand.len() as u32);
                    let pick = if *random {
                        use rand::seq::SliceRandom;
                        let mut h = hand.clone();
                        h.shuffle(&mut self.rng);
                        h.into_iter().take(k as usize).collect()
                    } else {
                        self.ask_objects(p, ctx.source, "Choose cards to discard", hand, k, k)
                    };
                    for c in pick {
                        if let Some(n) = self.discard(p, c, ctx.source) {
                            discarded.push(Entity::Object(n));
                        }
                    }
                }
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
            Effect::AddMana {
                who,
                mana,
                restriction,
            } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let produced = self.produce_mana(p, mana, ctx);
                let snow = ctx
                    .source
                    .is_some_and(|s| self.obj(s).chars.has_supertype(Supertype::Snow));
                let units: Vec<Mana> = produced
                    .into_iter()
                    .map(|t| Mana {
                        ty: t,
                        snow,
                        source: ctx.source,
                        restriction: restriction.clone(),
                        persistent: false,
                    })
                    .collect();
                self.add_mana(p, units, ctx.source);
            }
            Effect::AddPlayerCounters { who, kind, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                for p in self.eval_players(who, ctx) {
                    self.add_counters(Entity::Player(p), kind, k, ctx.source);
                }
            }
            Effect::Scry { who, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                for p in self.eval_players(who, ctx) {
                    crate::library::scry(self, p, k);
                }
            }
            Effect::Surveil { who, n } => {
                let k = self.eval_value(n, ctx).max(0) as u32;
                for p in self.eval_players(who, ctx) {
                    crate::library::surveil(self, p, k);
                }
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
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                let owner = self.eval_player(whose, ctx).unwrap_or(p);
                let n = self.eval_value(count, ctx).max(0) as u32;
                let found = crate::library::search(self, p, owner, filter, n, ctx);
                let _ = reveal;
                let res = self.move_to_destination(found, to, ctx);
                if *shuffle {
                    self.shuffle_library(owner);
                }
                ctx.prev_affected = res.iter().map(|o| Entity::Object(*o)).collect();
                ctx.set_var(vars::IT, res.into_iter().map(Entity::Object).collect());
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
            Effect::RevealHand { .. } => {}
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
                    ctx: ctx.clone(),
                    created_turn: self.turn.number,
                    created_step: Some(self.turn.step),
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
                });
            }
            Effect::CreateEmblem { abilities } => {
                crate::tokens::create_emblem(self, ctx.controller, abilities.clone(), ctx.source);
            }
            Effect::WinGame { who } => {
                for p in self.eval_players(who, ctx) {
                    self.player_wins(p);
                }
            }
            Effect::LoseGame { who } => {
                for p in self.eval_players(who, ctx) {
                    for e in self.replace(ReplEvent::LoseGame { player: p }) {
                        self.execute_repl_event(e);
                    }
                }
            }
            Effect::CastCard {
                who,
                what,
                free,
                optional,
            } => {
                let p = self.eval_player(who, ctx).unwrap_or(ctx.controller);
                for o in self.resolve_objects(what, ctx) {
                    if *optional && !self.ask_yes_no(p, Some(o), "Cast this card?", true) {
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
                let cands = self.objects_matching(filter, ctx);
                let min = if *up_to { 0 } else { n.min(cands.len() as u32) };
                let picked: Vec<Entity> = self
                    .ask_objects(p, ctx.source, "Choose", cands, min, n)
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
            other => self.eval_sel(other, ctx),
        }
    }

    pub fn resolve_objects(&mut self, sel: &Sel, ctx: &mut Ctx) -> Vec<ObjectId> {
        self.resolve_sel(sel, ctx)
            .into_iter()
            .filter_map(|e| e.object())
            .collect()
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
                other => other.clone(),
            })
            .collect()
    }

    fn fix_restriction(&self, r: &Restriction, _ctx: &Ctx) -> Restriction {
        r.clone()
    }

    /// Restrictions naming specific objects ("target creature can't block this turn")
    /// lock onto those objects.
    fn lock_restriction_objects(&self, r: &Restriction, ctx: &Ctx) -> Option<Vec<ObjectId>> {
        let f = match r {
            Restriction::CantAttack(f)
            | Restriction::CantBlock(f)
            | Restriction::CantAttackOrBlock(f)
            | Restriction::MustAttack(f)
            | Restriction::MustBlock(f)
            | Restriction::MustBeBlocked(f)
            | Restriction::CantBeBlocked(f)
            | Restriction::DoesntUntap(f)
            | Restriction::CantBeCountered(f)
            | Restriction::CantBeSacrificed(f) => f,
            Restriction::CantBeTargeted { what, .. } => what,
            _ => return None,
        };
        if filter_references_specific(f) {
            Some(
                self.objects_matching(f, ctx)
                    .into_iter()
                    .chain(ctx.targets.iter().flatten().filter_map(|e| e.object()))
                    .collect(),
            )
        } else {
            None
        }
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
        let mut counters: Vec<(CounterKind, u32)> = Vec::new();
        for (k, v) in &to.with_counters {
            counters.push((k.clone(), self.eval_value(v, ctx).max(0) as u32));
        }
        let attack = if to.attacking {
            self.attack_target_for_new_attacker(ctx)
        } else {
            None
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
                        controller: if to.zone == ZoneKind::Battlefield {
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
        combat.defending_players.first().map(|p| Entity::Player(*p))
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
                (0..k)
                    .map(|_| {
                        self.choose_mana_color(
                            p,
                            ctx,
                            &[
                                ManaType::W,
                                ManaType::U,
                                ManaType::B,
                                ManaType::R,
                                ManaType::G,
                            ],
                        )
                    })
                    .collect()
            }
            ManaProduction::OneOf(opts) => vec![self.choose_mana_color(p, ctx, opts)],
            ManaProduction::ChosenColor(n) => {
                let k = self.eval_value(n, ctx).max(0) as usize;
                let c = ctx
                    .source
                    .and_then(|s| self.obj(s).choices.color)
                    .map(ManaType::from_color)
                    .unwrap_or(ManaType::C);
                vec![c; k]
            }
            ManaProduction::CouldProduce(f) => {
                let types = crate::mana_abilities::types_could_produce(self, f, ctx);
                if types.is_empty() {
                    vec![]
                } else {
                    vec![self.choose_mana_color(p, ctx, &types)]
                }
            }
            ManaProduction::AnyTypeProduced => {
                let mask = ctx.event.as_ref().map_or(0, |e| e.amount);
                let types = crate::mana::types_from_mask(mask);
                if types.is_empty() {
                    vec![]
                } else {
                    vec![self.choose_mana_color(p, ctx, &types)]
                }
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
        }
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

/// A [`PlayerRef`] that always refers to a specific player (locked in at resolution).
pub fn player_const(p: PlayerId) -> PlayerRef {
    PlayerRef::Player(p)
}

/// A player filter matching exactly one player.
pub fn player_filter_const(p: PlayerId) -> PlayerFilter {
    PlayerFilter::Is(p)
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
