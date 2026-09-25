//! The stack (CR 405): creating stack objects for abilities, choosing modes and
//! targets (CR 601.2b–d, 603.3c–d, 115), checking target legality, and resolving the
//! top object (CR 608).

use crate::ability::*;
use crate::decision::{Answer, Decision};
use crate::eval::Ctx;
use crate::events::{Event, MoveCause};
use crate::game::*;
use crate::keywords::KeywordKind;
use crate::object::*;
use crate::replacement::*;
use crate::types::*;
use smol_str::SmolStr;

/// Key in [`StackInfo::chosen_values`] recording the opponent who made a choice for the
/// stack object (CR 601.7, 602.3); "that player" in its text refers to them.
pub const CHOOSER_KEY: u16 = u16::MAX;

/// Creates an ability object on top of the stack (CR 113.1b, 602.2a, 603.3a).
pub fn create_stack_ability(
    g: &mut Game,
    source: ObjectId,
    controller: PlayerId,
    ability: Ability,
    kind: StackKind,
    event: Option<EventInfo>,
    source_lki: Option<Box<Characteristics>>,
) -> ObjectId {
    // CR 602.2a, 603.3: the ability on the stack has the text of the ability that created
    // it and no other characteristics (its name here is only a label for logs).
    let src_name = g.obj(source).chars.name.clone();
    let chars = Characteristics {
        name: SmolStr::new(format!("{} ability", src_name)),
        rules_text: std::sync::Arc::from(ability.text.as_str()),
        ..Default::default()
    };
    let mut obj = GameObject::new(
        ObjectId(0),
        ObjKind::StackAbility,
        g.obj(source).owner,
        Zone::Stack,
        chars,
    );
    obj.controller = controller;
    obj.base_controller = controller;
    obj.stack = Some(Box::new(StackInfo {
        kind,
        chosen: vec![],
        x: None,
        cast: CastInfo::default(),
        event,
        source_lki: source_lki.or_else(|| Some(Box::new(g.obj(source).chars.clone()))),
        chosen_values: Default::default(),
    }));
    let _ = ability;
    let id = ObjectId(g.objects.len() as u32);
    obj.id = id;
    obj.timestamp = g.new_timestamp();
    g.objects.push(obj);
    g.stack.push(id);
    g.dirty = true;
    id
}

impl Game {
    /// The body (targets/effect/modes) of a stack object.
    pub fn stack_body(&self, id: ObjectId) -> Body {
        let o = self.obj(id);
        match o.stack.as_deref().map(|s| &s.kind) {
            Some(StackKind::Activated { ability, .. }) => match &ability.kind {
                AbilityKind::Activated(a) => a.body.clone(),
                _ => Body::default(),
            },
            Some(StackKind::Triggered { ability, .. }) => match &ability.kind {
                AbilityKind::Triggered(t) => t.body.clone(),
                _ => Body::default(),
            },
            Some(StackKind::Spell) => self.spell_body(id),
            None => Body::default(),
        }
    }

    /// The combined spell ability of a spell (CR 113.3a): the Spell abilities of its
    /// current characteristics. For fused split spells, both halves in order.
    pub fn spell_body(&self, id: ObjectId) -> Body {
        let o = self.obj(id);
        let mut bodies: Vec<Body> = o
            .chars
            .abilities
            .iter()
            .filter_map(|a| match &a.kind {
                AbilityKind::Spell(s) => Some(s.body.clone()),
                _ => None,
            })
            .collect();
        match bodies.len() {
            0 => Body::default(),
            1 => bodies.pop().unwrap(),
            _ => {
                // Multiple spell abilities (e.g. fused halves): concatenate target slots
                // and effects. Target slot indices of later bodies are offset.
                let mut out = Body::default();
                let mut effects = Vec::new();
                for b in bodies {
                    let offset = out.targets.len() as u8;
                    out.targets.extend(b.targets.clone());
                    effects.push(crate::oracle::offset_targets(&b.effect, offset));
                    if b.modal.is_some() && out.modal.is_none() {
                        out.modal = b.modal;
                    }
                }
                out.effect = Effect::seq(effects);
                out
            }
        }
    }

    /// Chooses modes and targets for a stack object as it's put on the stack. Returns
    /// false if a required choice can't be made (e.g. no legal targets).
    pub fn choose_modes_and_targets(&mut self, id: ObjectId, body: &Body, ctx: &mut Ctx) -> bool {
        let controller = ctx.controller;
        let mut chosen: Vec<ChosenMode> = Vec::new();
        if let Some(modal) = &body.modal {
            let min = self.eval_value(&modal.min, ctx).max(0) as u32;
            let max = self.eval_value(&modal.max, ctx).max(min as i64) as u32;
            let modes: Vec<String> = modal.modes.iter().map(|m| m.text.clone()).collect();
            // Only modes with legal targets can be chosen (CR 700.2a... via 601.2c legality).
            let available: Vec<usize> = (0..modal.modes.len())
                .filter(|i| self.targets_possible(&modal.modes[*i].targets, ctx, id))
                .collect();
            let picks: Vec<usize> = if modal.chooser == ModeChooser::Random {
                let i = self.random_range(0, modal.modes.len() as u32 - 1) as usize;
                vec![i]
            } else {
                let chooser = if modal.chooser == ModeChooser::Opponent {
                    // CR 601.7, 602.3: an opponent chooses the mode when the controller
                    // normally would; the controller decides which opponent (601.7a).
                    let o = self.deciding_opponent(controller, id, ctx);
                    if let Some(si) = self.objects[id.0 as usize].stack.as_mut() {
                        si.chosen_values.insert(CHOOSER_KEY, o.0 as i64);
                    }
                    o
                } else {
                    controller
                };
                let ans = self.ask(
                    chooser,
                    Decision::ChooseModes {
                        source: id,
                        modes: modes.clone(),
                        min,
                        max: max.min(if modal.allow_repeat {
                            u32::MAX
                        } else {
                            modes.len() as u32
                        }),
                        allow_repeat: modal.allow_repeat,
                    },
                );
                let valid = |v: &Vec<usize>| {
                    let mut s = v.clone();
                    s.sort_unstable();
                    let distinct = modal.allow_repeat || s.windows(2).all(|w| w[0] != w[1]);
                    v.len() as u32 >= min
                        && v.len() as u32 <= max
                        && distinct
                        && v.iter().all(|i| available.contains(i))
                };
                match ans {
                    Answer::Indices(v) if valid(&v) => v,
                    _ => available
                        .iter()
                        .copied()
                        .take(min.max(1) as usize)
                        .collect(),
                }
            };
            if (picks.len() as u32) < min {
                return false;
            }
            let mut picks = picks;
            // CR 700.2: modes are performed in the order printed.
            picks.sort_unstable();
            for m in picks {
                let mode = &modal.modes[m];
                let targets = match self.choose_targets_for(&mode.targets, ctx, id) {
                    Some(t) => t,
                    None => return false,
                };
                let divided = self.choose_divisions(&mode.targets, &targets, ctx, id);
                chosen.push(ChosenMode {
                    mode: Some(m),
                    targets,
                    divided,
                });
            }
        } else {
            let targets = match self.choose_targets_for(&body.targets, ctx, id) {
                Some(t) => t,
                None => return false,
            };
            let divided = self.choose_divisions(&body.targets, &targets, ctx, id);
            chosen.push(ChosenMode {
                mode: None,
                targets,
                divided,
            });
        }
        // Targets become targets: "becomes the target" triggers (CR 601.2c, 603.3d).
        let mut became: Vec<Entity> = Vec::new();
        for cm in &chosen {
            for slot in &cm.targets {
                for t in slot {
                    if !became.contains(t) {
                        became.push(*t);
                    }
                }
            }
        }
        if let Some(si) = self.objects[id.0 as usize].stack.as_mut() {
            si.chosen = chosen;
        }
        for t in became {
            self.emit(Event::BecameTarget {
                target: t,
                by: id,
                controller,
            });
        }
        true
    }

    /// Whether every required target slot has enough legal choices. Slots required only
    /// if some choice is made (e.g. a kicker cost is paid, CR 601.2c) are optional here.
    pub fn targets_possible(&self, specs: &[TargetSpec], ctx: &Ctx, stack_obj: ObjectId) -> bool {
        specs.iter().all(|s| {
            s.min == 0
                || s.condition.is_some()
                || self.legal_target_candidates(s, ctx, stack_obj).len() as u32 >= s.min
        })
    }

    /// The opponent who makes a choice that "an opponent" makes while a spell or ability
    /// is put on the stack: the controller decides which one (CR 601.7a, 602.3a).
    /// Remembered for the stack object, so every such choice is made by the same player.
    pub fn deciding_opponent(
        &mut self,
        controller: PlayerId,
        id: ObjectId,
        ctx: &mut Ctx,
    ) -> PlayerId {
        if let Some(p) = ctx.chosen_player {
            if self.are_opponents(controller, p) {
                return p;
            }
        }
        let opps = self.opponents(controller);
        let p = match opps.len() {
            0 => controller,
            1 => opps[0],
            _ => self
                .ask_entities(
                    controller,
                    Some(id),
                    "Choose the opponent who makes the choice",
                    opps.iter().map(|p| Entity::Player(*p)).collect(),
                    1,
                    1,
                )
                .first()
                .and_then(|e| e.player())
                .unwrap_or(opps[0]),
        };
        ctx.chosen_player = Some(p);
        p
    }

    /// All legal choices for a target slot (CR 115).
    pub fn legal_target_candidates(
        &self,
        spec: &TargetSpec,
        ctx: &Ctx,
        stack_obj: ObjectId,
    ) -> Vec<Entity> {
        let mut out = Vec::new();
        let obj_zone = |f: &Filter| f.zone().unwrap_or(ZoneKind::Battlefield);
        match &spec.what {
            TargetKind::Object(f) => {
                for o in self.objects_in_zone_kind(obj_zone(f)) {
                    if self.is_legal_target(spec, Entity::Object(o), ctx, stack_obj) {
                        out.push(Entity::Object(o));
                    }
                }
            }
            TargetKind::Spell(_) | TargetKind::Ability(_) | TargetKind::SpellOrAbility(_) => {
                for o in self.stack.clone() {
                    if o != stack_obj
                        && self.is_legal_target(spec, Entity::Object(o), ctx, stack_obj)
                    {
                        out.push(Entity::Object(o));
                    }
                }
            }
            TargetKind::Player(_) => {
                for p in self.players_in_game() {
                    if self.is_legal_target(spec, Entity::Player(p), ctx, stack_obj) {
                        out.push(Entity::Player(p));
                    }
                }
            }
            TargetKind::AnyTarget | TargetKind::ObjectOrPlayer(..) => {
                for o in self.permanent_ids() {
                    if self.is_legal_target(spec, Entity::Object(o), ctx, stack_obj) {
                        out.push(Entity::Object(o));
                    }
                }
                for p in self.players_in_game() {
                    if self.is_legal_target(spec, Entity::Player(p), ctx, stack_obj) {
                        out.push(Entity::Player(p));
                    }
                }
            }
        }
        out
    }

    /// Whether `e` is a legal target for the slot (CR 115.4, 702.11 hexproof,
    /// 702.18 shroud, 702.16 protection).
    pub fn is_legal_target(
        &self,
        spec: &TargetSpec,
        e: Entity,
        ctx: &Ctx,
        stack_obj: ObjectId,
    ) -> bool {
        let source_obj = if self.is_live(stack_obj) {
            Some(stack_obj)
        } else {
            ctx.source
        };
        match e {
            Entity::Player(p) => {
                if !self.player(p).in_game() {
                    return false;
                }
                let ok = match &spec.what {
                    TargetKind::Player(f) => self.player_filter_matches(f, p, ctx),
                    TargetKind::AnyTarget => true,
                    TargetKind::ObjectOrPlayer(_, f) => self.player_filter_matches(f, p, ctx),
                    _ => false,
                };
                ok && !self.player_untargetable(p, ctx.controller, source_obj)
            }
            Entity::Object(o) => {
                if !self.is_live(o) {
                    return false;
                }
                let ob = self.obj(o);
                if ob.phased_out {
                    return false;
                }
                let ok = match &spec.what {
                    TargetKind::Object(f) => {
                        let z = f.zone().unwrap_or(ZoneKind::Battlefield);
                        ob.zone.kind() == Some(z) && self.matches(o, f, ctx)
                    }
                    TargetKind::AnyTarget => {
                        ob.zone == Zone::Battlefield
                            && (ob.is_creature()
                                || ob.is(CardType::Planeswalker)
                                || ob.is(CardType::Battle))
                    }
                    TargetKind::ObjectOrPlayer(f, _) => {
                        ob.zone == Zone::Battlefield && self.matches(o, f, ctx)
                    }
                    TargetKind::Spell(f) => ob.is_spell() && self.matches(o, f, ctx),
                    TargetKind::Ability(f) => ob.is_stack_ability() && self.matches(o, f, ctx),
                    TargetKind::SpellOrAbility(f) => {
                        ob.zone == Zone::Stack && self.matches(o, f, ctx)
                    }
                    TargetKind::Player(_) => false,
                };
                ok && !self.object_untargetable(o, ctx.controller, source_obj)
            }
        }
    }

    /// Hexproof, shroud, protection and "can't be the target" restrictions on objects.
    pub fn object_untargetable(&self, o: ObjectId, by: PlayerId, source: Option<ObjectId>) -> bool {
        let ob = self.obj(o);
        // Hexproof, shroud and protection are abilities of permanents: a spell with them
        // on the stack can still be targeted (CR 113.6, 702.11b, 702.16b, 702.18a).
        if ob.zone == Zone::Battlefield {
            let c = &ob.chars;
            // CR 702.18a shroud
            if c.has_keyword(KeywordKind::Shroud) {
                return true;
            }
            // CR 702.11b hexproof (and "hexproof from", 702.11d). An ability's qualities
            // are those of its source (CR 113.7).
            let quality_source = source.map(|s| self.ability_source_of(s));
            for kw in c.keywords().filter(|k| k.kind == KeywordKind::Hexproof) {
                if self.are_opponents(by, ob.controller) {
                    match &kw.filter {
                        None => return true,
                        Some(f) => {
                            if let Some(s) = quality_source {
                                if self.matches(s, f, &Ctx::new(Some(o), ob.controller)) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            // CR 702.16b protection: can't be the target of spells/abilities from sources with the quality.
            if let Some(s) = source {
                if self.protected_from(o, s) {
                    return true;
                }
            }
        }
        // Static and resolved "can't be the target" restrictions.
        let check = |r: &Restriction, rs: Option<ObjectId>, rc: PlayerId| -> bool {
            if let Restriction::CantBeTargeted { what, by: tr } = r {
                if !self.matches(o, what, &Ctx::new(rs, rc)) {
                    return false;
                }
                match tr {
                    TargetRestriction::Any => true,
                    TargetRestriction::Opponents => self.are_opponents(by, rc),
                    TargetRestriction::Sources(f) => source.is_some_and(|s| {
                        self.matches(self.ability_source_of(s), f, &Ctx::new(rs, rc))
                    }),
                }
            } else {
                false
            }
        };
        self.statics
            .restrictions
            .iter()
            .any(|(s, c, r)| check(r, Some(*s), *c))
            || self.rule_effects.iter().any(|e| {
                e.objects.as_ref().is_none_or(|v| v.contains(&o))
                    && check(&e.restriction, e.source, e.controller)
            })
    }

    pub fn player_untargetable(&self, p: PlayerId, by: PlayerId, source: Option<ObjectId>) -> bool {
        let pl = self.player(p);
        for m in &pl.mods {
            match m {
                PlayerModification::Shroud => return true,
                PlayerModification::Hexproof if self.are_opponents(by, p) => return true,
                PlayerModification::ProtectionFrom(f) => {
                    if let Some(s) = source {
                        if self.matches(self.ability_source_of(s), f, &Ctx::new(None, p)) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        let check = |r: &Restriction, rs: Option<ObjectId>, rc: PlayerId| -> bool {
            if let Restriction::PlayerCantBeTargeted { who, by: tr } = r {
                if !self.player_filter_matches(who, p, &Ctx::new(rs, rc)) {
                    return false;
                }
                match tr {
                    TargetRestriction::Any => true,
                    TargetRestriction::Opponents => self.are_opponents(by, p),
                    TargetRestriction::Sources(f) => source.is_some_and(|s| {
                        self.matches(self.ability_source_of(s), f, &Ctx::new(rs, rc))
                    }),
                }
            } else {
                false
            }
        };
        self.statics
            .restrictions
            .iter()
            .any(|(s, c, r)| check(r, Some(*s), *c))
            || self
                .rule_effects
                .iter()
                .any(|e| check(&e.restriction, e.source, e.controller))
    }

    /// For an activated or triggered ability on the stack, the object that is its source
    /// (its last known information if it has left its zone); otherwise the object itself.
    pub fn ability_source_of(&self, id: ObjectId) -> ObjectId {
        match self.obj(id).stack.as_deref().map(|s| &s.kind) {
            Some(StackKind::Activated { source: s, .. })
            | Some(StackKind::Triggered { source: s, .. }) => *s,
            _ => id,
        }
    }

    /// Whether permanent/object `o` has protection from `source` (CR 702.16).
    pub fn protected_from(&self, o: ObjectId, source: ObjectId) -> bool {
        let ob = self.obj(o);
        let src_ctx = Ctx::new(Some(o), ob.controller);
        // For abilities on the stack, the source of the ability is its source object.
        let src = self.ability_source_of(source);
        ob.chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Protection)
            .any(|k| match &k.filter {
                None => true,
                Some(f) => self.matches(src, f, &src_ctx),
            })
    }

    /// Asks the controller to choose targets for each slot. Returns None if impossible.
    fn choose_targets_for(
        &mut self,
        specs: &[TargetSpec],
        ctx: &mut Ctx,
        stack_obj: ObjectId,
    ) -> Option<Vec<Vec<Entity>>> {
        let mut out: Vec<Vec<Entity>> = vec![vec![]; specs.len()];
        // Candidates and maximum per slot, for the "must target" check below.
        let mut slot_cands: Vec<Vec<Entity>> = vec![vec![]; specs.len()];
        let mut slot_max: Vec<u32> = vec![0; specs.len()];
        // CR 601.7b: when the controller and an opponent both choose targets, the
        // controller chooses first.
        let order: Vec<usize> = (0..specs.len())
            .filter(|i| !specs[*i].chosen_by_opponent)
            .chain((0..specs.len()).filter(|i| specs[*i].chosen_by_opponent))
            .collect();
        for i in order {
            let spec = &specs[i];
            ctx.targets = out.clone();
            // CR 601.2c: a target required only if some choice was made (e.g. a kicker
            // cost was paid) isn't required otherwise.
            if let Some(c) = &spec.condition {
                if !self.eval_cond(c, ctx) {
                    continue;
                }
            }
            let mut cands = self.legal_target_candidates(spec, ctx, stack_obj);
            // "another target": exclude entities chosen in the listed slots.
            for d in &spec.distinct_from {
                if let Some(prev) = out.get(*d as usize) {
                    cands.retain(|c| !prev.contains(c));
                }
            }
            let max = (self.eval_value(&spec.max, ctx).max(0) as u32).min(cands.len() as u32);
            if (cands.len() as u32) < spec.min {
                return None;
            }
            slot_cands[i] = cands.clone();
            slot_max[i] = max;
            let chooser = if spec.chosen_by_opponent {
                self.deciding_opponent(ctx.controller, stack_obj, ctx)
            } else {
                ctx.controller
            };
            let picked = if max == 0 {
                vec![]
            } else {
                match self.ask(
                    chooser,
                    Decision::ChooseTargets {
                        source: stack_obj,
                        text: spec.text.clone(),
                        candidates: cands.clone(),
                        min: spec.min,
                        max,
                    },
                ) {
                    Answer::Entities(v)
                        if v.len() as u32 >= spec.min
                            && v.len() as u32 <= max
                            && v.iter().all(|e| cands.contains(e))
                            && distinct(&v) =>
                    {
                        v
                    }
                    _ => cands
                        .iter()
                        .copied()
                        .take(spec.min.max(if spec.min == 0 { 0 } else { 1 }) as usize)
                        .collect(),
                }
            };
            out[i] = picked;
        }
        self.enforce_must_target(specs, &mut out, &slot_cands, &slot_max, ctx, stack_obj);
        ctx.targets = out.clone();
        Some(out)
    }

    /// CR 601.2c: "If any effects say that an object or player must be chosen as a
    /// target, the player chooses targets so that they obey the maximum possible number of
    /// such effects". Applies while casting a spell or activating an ability.
    fn enforce_must_target(
        &mut self,
        specs: &[TargetSpec],
        out: &mut [Vec<Entity>],
        cands: &[Vec<Entity>],
        maxes: &[u32],
        ctx: &Ctx,
        stack_obj: ObjectId,
    ) {
        let cast_or_activated = matches!(
            self.obj(stack_obj).stack.as_deref().map(|s| &s.kind),
            Some(StackKind::Spell) | Some(StackKind::Activated { .. })
        );
        if !cast_or_activated {
            return;
        }
        let chooser = ctx.controller;
        let mut reqs: Vec<(Filter, Ctx)> = Vec::new();
        let mut collect = |r: &Restriction, s: Option<ObjectId>, c: PlayerId, g: &Game| {
            if let Restriction::MustTarget { chooser: who, what } = r {
                let rctx = Ctx::new(s, c);
                if g.player_filter_matches(who, chooser, &rctx) {
                    reqs.push((what.clone(), rctx));
                }
            }
        };
        for (s, c, r) in self.statics.restrictions.clone() {
            collect(&r, Some(s), c, self);
        }
        for e in self.rule_effects.clone() {
            collect(&e.restriction, e.source, e.controller, self);
        }
        for (what, rctx) in reqs {
            let fits =
                |g: &Game, e: &Entity| e.object().is_some_and(|o| g.matches(o, &what, &rctx));
            if out.iter().flatten().any(|e| fits(self, e)) {
                continue;
            }
            // Find a slot that can take a required object ("if able").
            for i in 0..specs.len() {
                let Some(e) = cands[i]
                    .iter()
                    .find(|e| fits(self, e) && !out[i].contains(e))
                    .copied()
                else {
                    continue;
                };
                if (out[i].len() as u32) < maxes[i] {
                    out[i].push(e);
                } else if let Some(j) = out[i].iter().position(|x| !fits(self, x)) {
                    out[i][j] = e;
                } else {
                    continue;
                }
                break;
            }
        }
    }

    fn choose_divisions(
        &mut self,
        specs: &[TargetSpec],
        targets: &[Vec<Entity>],
        ctx: &Ctx,
        stack_obj: ObjectId,
    ) -> Vec<Vec<u32>> {
        let mut out = Vec::new();
        for (spec, slot) in specs.iter().zip(targets.iter()) {
            match &spec.divide {
                Some(total) if !slot.is_empty() => {
                    let total = self.eval_value(total, ctx).max(0) as u32;
                    let n = slot.len() as u32;
                    let default = {
                        let mut v = vec![total / n.max(1); slot.len()];
                        let rem = total - (total / n.max(1)) * n;
                        for x in v.iter_mut().take(rem as usize) {
                            *x += 1;
                        }
                        v
                    };
                    let ans = self.ask(
                        ctx.controller,
                        Decision::Divide {
                            source: stack_obj,
                            total,
                            recipients: slot.clone(),
                            min_each: 1,
                        },
                    );
                    let div = match ans {
                        Answer::Numbers(v)
                            if v.len() == slot.len()
                                && v.iter().all(|x| *x >= 1)
                                && v.iter().sum::<i64>() == total as i64 =>
                        {
                            v.into_iter().map(|x| x as u32).collect()
                        }
                        _ => default,
                    };
                    out.push(div);
                }
                _ => out.push(vec![]),
            }
        }
        out
    }

    // ------------------------------------------------------------------
    // Resolution (CR 608)
    // ------------------------------------------------------------------

    /// Resolves the top object of the stack.
    pub fn resolve_top(&mut self) {
        let Some(&top) = self.stack.last() else {
            return;
        };
        if self.dirty {
            self.recompute();
        }
        let is_spell = matches!(
            self.obj(top).stack.as_deref().map(|s| &s.kind),
            Some(StackKind::Spell)
        );
        self.log(|g| format!("Resolving {}", g.describe(top)));
        // CR 117.2e: no player has priority while a spell or ability is resolving.
        let priority = self.turn.priority.take();
        if is_spell {
            self.resolve_spell(top);
        } else {
            self.resolve_ability(top);
        }
        self.turn.priority = priority;
        self.flush_events();
    }

    /// Builds the resolution context for a stack object.
    pub fn stack_ctx(&self, id: ObjectId) -> Ctx {
        let o = self.obj(id);
        let si = o.stack.as_deref().expect("stack object without stack info");
        let (source, ability_uid, link) = match &si.kind {
            StackKind::Spell => (Some(id), 0, 0),
            StackKind::Activated { source, ability } | StackKind::Triggered { source, ability } => {
                (Some(*source), ability.uid, ability.link)
            }
        };
        let mut ctx = self.saved_ctx.get(&id).cloned().unwrap_or_default();
        ctx.source = source;
        ctx.controller = o.controller;
        ctx.stack_obj = Some(id);
        ctx.x = si.x.unwrap_or(ctx.x);
        ctx.event = si.event.clone().or(ctx.event);
        // An ability's "if it was kicked", "if you cast it from your hand" etc. refer to how
        // its source was cast.
        ctx.cast = match &si.kind {
            StackKind::Spell => Some(si.cast.clone()),
            _ => ctx
                .cast
                .or_else(|| source.and_then(|s| self.obj(s).cast.as_deref().cloned())),
        };
        ctx.ability_uid = ability_uid;
        ctx.link = link;
        ctx.source_lki = si.source_lki.clone();
        if let Some(p) = si.chosen_values.get(&CHOOSER_KEY) {
            ctx.chosen_player = Some(PlayerId(*p as u8));
        }
        ctx
    }

    /// Re-checks targets (CR 608.2b). Returns (chosen modes with only legal targets, any
    /// target existed, all targets illegal).
    fn recheck_targets(&self, id: ObjectId, body: &Body, ctx: &Ctx) -> (Vec<ChosenMode>, bool) {
        let si = self.obj(id).stack.as_deref().unwrap();
        let mut any_target = false;
        let mut any_legal = false;
        let mut out = Vec::new();
        for cm in &si.chosen {
            let specs: &[TargetSpec] = match (cm.mode, &body.modal) {
                (Some(m), Some(modal)) => &modal.modes[m].targets,
                _ => &body.targets,
            };
            let mut new_targets = Vec::new();
            let mut c2 = ctx.clone();
            c2.targets = cm.targets.clone();
            for (i, slot) in cm.targets.iter().enumerate() {
                let Some(spec) = specs.get(i.min(specs.len().saturating_sub(1))) else {
                    new_targets.push(slot.clone());
                    continue;
                };
                let mut legal = Vec::new();
                for t in slot {
                    any_target = true;
                    if self.is_legal_target(spec, *t, &c2, id) {
                        legal.push(*t);
                        any_legal = true;
                    }
                }
                new_targets.push(legal);
            }
            out.push(ChosenMode {
                mode: cm.mode,
                targets: new_targets,
                divided: cm.divided.clone(),
            });
        }
        (out, any_target && !any_legal)
    }

    fn resolve_ability(&mut self, id: ObjectId) {
        let body = self.stack_body(id);
        let mut ctx = self.stack_ctx(id);
        let si = self.obj(id).stack.as_deref().unwrap().clone();
        let (src, uid) = match &si.kind {
            StackKind::Triggered { source, ability } | StackKind::Activated { source, ability } => {
                (*source, ability.uid)
            }
            _ => (ObjectId(0), 0),
        };
        // CR 608.2a: intervening "if".
        if let StackKind::Triggered { ability, .. } = &si.kind {
            if let AbilityKind::Triggered(t) = &ability.kind {
                if let Some(c) = &t.intervening_if {
                    if !self.eval_cond(c, &ctx) {
                        self.remove_from_stack(id);
                        self.state_triggers_active.remove(&(src, uid));
                        return;
                    }
                }
            }
        }
        let (chosen, all_illegal) = self.recheck_targets(id, &body, &ctx);
        if all_illegal {
            // CR 608.2b: doesn't resolve.
            self.remove_from_stack(id);
            self.state_triggers_active.remove(&(src, uid));
            return;
        }
        // CR 603.7h: count the resolutions of this ability this turn (this one included).
        *self.objects[src.0 as usize]
            .triggers_this_turn
            .entry(uid | crate::triggers::turn_keys::RESOLVED)
            .or_insert(0) += 1;
        let trig = match &si.kind {
            StackKind::Triggered { ability, .. } => match &ability.kind {
                AbilityKind::Triggered(t) => Some(t.clone()),
                _ => None,
            },
            _ => None,
        };
        // CR 603.6e: an Aura's ability that triggers on the enchanted permanent leaving the
        // battlefield can find the new object the Aura card became in its owner's
        // graveyard.
        if let Some(t) = &trig {
            let about_enchanted = match &t.trigger {
                TriggerCond::LeavesBattlefield(f) | TriggerCond::Dies(f) => {
                    filter_mentions_attached(f)
                }
                _ => false,
            };
            let now = self.current(src);
            if about_enchanted
                && now != src
                && self.obj(src).chars.has_subtype("Aura")
                && matches!(self.obj(now).zone, Zone::Graveyard(_))
            {
                ctx.source = Some(now);
            }
        }
        self.exec_chosen(&body, &chosen, &mut ctx);
        // CR 603.2h: remember that a "do this only once each turn" action was taken.
        if trig.as_ref().is_some_and(|t| t.do_once_per_turn) && ctx.prev_happened {
            *self.objects[src.0 as usize]
                .triggers_this_turn
                .entry(uid | crate::triggers::turn_keys::DONE_ONCE)
                .or_insert(0) += 1;
        }
        // CR 608.2n: the ability ceases to exist.
        self.remove_from_stack(id);
        self.state_triggers_active.remove(&(src, uid));
        // CR 608.2p: then abilities that trigger when it resolves trigger.
        let controller = self.obj(id).controller;
        self.emit(Event::AbilityResolved {
            ability: id,
            source: src,
            controller,
        });
    }

    /// Executes a body's effect(s) for the chosen modes.
    pub fn exec_chosen(&mut self, body: &Body, chosen: &[ChosenMode], ctx: &mut Ctx) {
        for cm in chosen {
            ctx.targets = cm.targets.clone();
            ctx.divided = cm.divided.clone();
            let effect = match (cm.mode, &body.modal) {
                (Some(m), Some(modal)) => modal.modes[m].effect.clone(),
                _ => body.effect.clone(),
            };
            self.exec(&effect, ctx);
        }
    }

    /// CR 608.3g: static abilities of a resolving permanent spell that function on the
    /// stack and create delayed triggered abilities create them as it enters. "It" is the
    /// new permanent.
    fn delayed_triggers_as_enters(&mut self, spell: &GameObject, new: ObjectId) {
        for a in &spell.chars.abilities {
            let AbilityKind::Static(s) = &a.kind else {
                continue;
            };
            let StaticEffect::DelayedTriggerAsEnters {
                condition,
                trigger,
                body,
            } = &s.effect
            else {
                continue;
            };
            let mut ctx = Ctx::new(Some(new), spell.controller);
            ctx.cast = spell.stack.as_deref().map(|si| si.cast.clone());
            if let Some(c) = condition {
                let mut sctx = ctx.clone();
                sctx.source = Some(spell.id);
                if !self.eval_cond(c, &sctx) {
                    continue;
                }
            }
            ctx.set_var(vars::IT, vec![Entity::Object(new)]);
            let id = self.new_effect_id();
            self.delayed_triggers.push(crate::game::DelayedTrigger {
                id,
                source: Some(new),
                controller: spell.controller,
                trigger: trigger.clone(),
                body: body.clone(),
                once: true,
                ctx,
                created_turn: self.turn.number,
                created_step: Some(self.turn.step),
            });
        }
    }

    pub(crate) fn remove_from_stack(&mut self, id: ObjectId) {
        self.stack.retain(|x| *x != id);
        let o = &mut self.objects[id.0 as usize];
        if o.kind == ObjKind::StackAbility {
            o.zone = Zone::Nowhere;
            // CR 603.8: a state trigger can trigger again once it has left the stack
            // (resolved, been countered, or otherwise removed).
            if let Some(StackKind::Triggered { source, ability }) =
                o.stack.as_deref().map(|s| &s.kind)
            {
                let key = (*source, ability.uid);
                self.state_triggers_active.remove(&key);
            }
        }
        self.saved_ctx.remove(&id);
        self.dirty = true;
    }

    fn resolve_spell(&mut self, id: ObjectId) {
        let o = self.obj(id).clone();
        let controller = o.controller;
        let is_permanent = o.chars.is_permanent_type();
        let mut body = self.spell_body(id);
        // CR 303.4a / 608.3b: an Aura spell's target is what it will enchant, as when it
        // was cast.
        if o.chars.has_subtype("Aura") && body.targets.is_empty() && !o.face_down {
            if let Some(spec) = crate::attach::aura_target_spec(&o.chars) {
                body.targets.push(spec);
            }
        }
        let mut ctx = self.stack_ctx(id);
        let (chosen, all_illegal) = self.recheck_targets(id, &body, &ctx);
        let si = o.stack.as_deref().unwrap().clone();
        if is_permanent {
            // CR 608.3
            let mut etb = EtbInfo {
                cast: Some(si.cast.clone()),
                ..Default::default()
            };
            etb.controller = Some(controller);
            if o.face_down {
                etb.face_down = Some(match &si.cast.method {
                    CastMethod::FaceDown(k) => *k,
                    _ => KeywordKind::Morph,
                });
            }
            if matches!(o.face, FaceState::Back) {
                etb.face = Some(FaceState::Back);
            }
            let is_aura = o.chars.has_subtype("Aura");
            let bestowed = si.cast.paid.iter().any(|p| p == "bestow");
            if is_aura && !body.targets.is_empty()
                || (is_aura && si.chosen.iter().any(|c| !c.targets.is_empty()))
            {
                let target = chosen
                    .first()
                    .and_then(|c| c.targets.first())
                    .and_then(|v| v.first())
                    .copied();
                match target {
                    Some(t) => etb.attach_to = Some(t),
                    None if all_illegal && bestowed => {
                        // CR 702.103e: becomes a creature spell and resolves.
                        crate::keyword_impls::unbestow_on_stack(self, id);
                    }
                    None if all_illegal => {
                        // CR 608.3b: doesn't resolve.
                        self.counter_or_fizzle(id, MoveCause::Resolve);
                        return;
                    }
                    None => {}
                }
            } else if all_illegal && !crate::keyword_impls::is_mutating(self, id) {
                self.counter_or_fizzle(id, MoveCause::Resolve);
                return;
            }
            if crate::keyword_impls::is_mutating(self, id) && !all_illegal {
                crate::keyword_impls::resolve_mutate(self, id);
                return;
            }
            let copy = o.kind == ObjKind::SpellCopy || o.kind == ObjKind::CardCopy;
            if copy {
                // CR 608.3f: a copy of a permanent spell becomes a token.
                self.objects[id.0 as usize].kind = ObjKind::Token;
            }
            let res = self.move_object_ev(MoveEv {
                obj: id,
                to: Zone::Battlefield,
                pos: LibraryPosition::Top,
                cause: MoveCause::Resolve,
                by: Some(controller),
                etb,
                source: None,
            });
            if res.is_none() && self.is_live(id) {
                // CR 608.3e: can't be put onto the battlefield → graveyard.
                let owner = o.owner;
                self.move_object(
                    id,
                    Zone::Graveyard(owner),
                    MoveCause::Resolve,
                    Some(controller),
                );
            }
            if let Some(new) = res {
                // CR 607.2q: cards exiled to pay its cost are exiled with the permanent.
                for c in &si.cast.cost_objects {
                    let now = self.current(*c);
                    if self.is_live(now) && self.obj(now).zone == Zone::Exile {
                        self.objects[new.0 as usize]
                            .linked
                            .entry(0)
                            .or_default()
                            .push(now);
                    }
                }
                self.delayed_triggers_as_enters(&o, new);
                crate::keyword_impls::after_permanent_spell_resolves(self, id, new);
            }
            self.emit(Event::SpellResolved { spell: id });
            return;
        }
        // Instant or sorcery (CR 608.2).
        if all_illegal {
            self.counter_or_fizzle(id, MoveCause::Resolve);
            return;
        }
        self.exec_chosen(&body, &chosen, &mut ctx);
        // CR 608.2n: put into owner's graveyard (or wherever a replacement sends it).
        if self.is_live(id) && self.obj(id).zone == Zone::Stack {
            let dest = crate::keyword_impls::resolved_spell_destination(self, id);
            self.move_object_ev(MoveEv {
                obj: id,
                to: dest.0,
                pos: dest.1,
                cause: MoveCause::Resolve,
                by: Some(controller),
                etb: EtbInfo::default(),
                source: None,
            });
        }
        self.emit(Event::SpellResolved { spell: id });
    }

    /// Removes a spell that failed to resolve (or was countered) to its owner's graveyard,
    /// honoring replacement effects (e.g. flashback exile). Abilities just leave the stack.
    pub fn counter_or_fizzle(&mut self, id: ObjectId, cause: MoveCause) {
        let o = self.obj(id);
        if o.kind == ObjKind::StackAbility {
            self.remove_from_stack(id);
            return;
        }
        let owner = o.owner;
        let dest = crate::keyword_impls::countered_spell_destination(self, id);
        let _ = owner;
        self.move_object_ev(MoveEv {
            obj: id,
            to: dest.0,
            pos: dest.1,
            cause,
            by: None,
            etb: EtbInfo::default(),
            source: None,
        });
    }

    /// Counters a spell or ability (CR 701.6). Returns false if it can't be countered.
    pub fn counter(&mut self, id: ObjectId, by: Option<ObjectId>) -> bool {
        if !self.is_live(id) || self.obj(id).zone != Zone::Stack {
            return false;
        }
        if self.cant_be_countered(id) {
            return false;
        }
        let _ = by;
        self.counter_or_fizzle(id, MoveCause::Counter);
        self.emit(Event::Countered { what: id });
        true
    }

    pub fn cant_be_countered(&self, id: ObjectId) -> bool {
        // CR 603.1a: "This ability can't be countered."
        if let Some(StackKind::Triggered { ability, .. }) =
            self.obj(id).stack.as_deref().map(|s| &s.kind)
        {
            if matches!(&ability.kind, AbilityKind::Triggered(t) if t.cant_be_countered) {
                return true;
            }
        }
        self.statics.restrictions.iter().any(|(s, c, r)| match r {
            Restriction::CantBeCountered(f) => self.matches(id, f, &Ctx::new(Some(*s), *c)),
            _ => false,
        }) || self.rule_effects.iter().any(|e| match &e.restriction {
            Restriction::CantBeCountered(f) => {
                e.objects.as_ref().is_none_or(|v| v.contains(&id))
                    && self.matches(id, f, &Ctx::new(e.source, e.controller))
            }
            _ => false,
        })
    }
}

/// Whether a filter refers to the object the source is attached to ("enchanted creature").
fn filter_mentions_attached(f: &Filter) -> bool {
    match f {
        Filter::AttachedToSource => true,
        Filter::And(v) | Filter::Or(v) => v.iter().any(filter_mentions_attached),
        _ => false,
    }
}

fn distinct(v: &[Entity]) -> bool {
    let mut s = v.to_vec();
    s.sort();
    s.windows(2).all(|w| w[0] != w[1])
}
