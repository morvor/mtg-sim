//! Characteristic computation: the layer system (CR 613), plus collection of the
//! rule-modifying static effects that apply after it (CR 613.10–613.11).
//!
//! Characteristics are computed in place: each object's `chars`/`controller` is reset
//! to its base values and then modified layer by layer, so filters evaluated during a
//! layer see the interim values produced by earlier layers (CR 613.5, 613.6).

use crate::ability::*;
use crate::eval::Ctx;
use crate::game::*;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smallvec::SmallVec;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::Arc;

/// Identifies a continuous effect for ordering and "already started" tracking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffKey {
    /// A resolved effect, by index into `Game::effects` at the time of computation.
    Resolved(usize),
    /// A static ability: (source object, ability uid).
    Static(ObjectId, u64),
}

#[derive(Clone, Debug)]
struct LayerEff {
    key: EffKey,
    timestamp: Timestamp,
    cda: bool,
}

impl Game {
    /// Objects whose characteristics are computed: every current object outside libraries
    /// (library cards keep their printed characteristics).
    pub fn live_objects(&self) -> Vec<ObjectId> {
        let mut v: Vec<ObjectId> =
            Vec::with_capacity(self.battlefield.len() + self.stack.len() + 32);
        v.extend(self.battlefield.iter().copied());
        v.extend(self.stack.iter().copied());
        v.extend(self.exile.iter().copied());
        v.extend(self.command.iter().copied());
        v.extend(self.ante.iter().copied());
        for p in &self.players {
            v.extend(p.hand.iter().copied());
            v.extend(p.graveyard.iter().copied());
            // CR 604.3: characteristic-defining abilities function in all zones, so cards
            // in libraries that have one are recomputed too.
            v.extend(p.library.iter().copied().filter(|id| {
                self.obj(*id)
                    .base
                    .abilities
                    .iter()
                    .any(|a| matches!(&a.kind, AbilityKind::Static(s) if s.is_cda))
            }));
        }
        v
    }

    /// Whether an ability of this object functions where the object currently is
    /// (CR 113.6).
    pub fn ability_functions(&self, obj: &GameObject, zone: FunctionZone, is_cda: bool) -> bool {
        if is_cda || zone == FunctionZone::Anywhere {
            return true;
        }
        match obj.zone {
            Zone::Battlefield => zone == FunctionZone::Battlefield && !obj.phased_out,
            Zone::Stack => zone == FunctionZone::Stack,
            Zone::Hand(_) => zone == FunctionZone::Hand,
            Zone::Graveyard(_) => zone == FunctionZone::Graveyard,
            Zone::Exile => zone == FunctionZone::Exile,
            Zone::Library(_) => zone == FunctionZone::Library,
            Zone::Command => {
                zone == FunctionZone::Command
                    || (zone == FunctionZone::Battlefield && self.command_object_functions(obj))
            }
            _ => false,
        }
    }

    /// Emblems, planes, phenomena, schemes, vanguards, face-up conspiracies and dungeons
    /// have abilities that function in the command zone (CR 114.4, 311, 312, 313, 314, 315, 309).
    fn command_object_functions(&self, obj: &GameObject) -> bool {
        obj.kind == ObjKind::Emblem
            || [
                CardType::Plane,
                CardType::Phenomenon,
                CardType::Scheme,
                CardType::Vanguard,
                CardType::Dungeon,
            ]
            .iter()
            .any(|t| obj.chars.is(*t))
            || (obj.chars.is(CardType::Conspiracy) && !obj.face_down)
    }

    /// Recomputes all characteristics and rule effects.
    pub fn recompute(&mut self) {
        self.dirty = false;
        self.expire_dependent_effects();
        let live = self.live_objects();
        let prev_controllers: Vec<(ObjectId, PlayerId)> = self
            .battlefield
            .iter()
            .map(|id| (*id, self.obj(*id).controller))
            .collect();

        // Layer 0: printed values.
        for id in &live {
            let o = &mut self.objects[id.0 as usize];
            o.chars = o.base.clone();
            o.controller = o.base_controller;
        }

        // Layer 1a: copy effects (CR 707), in timestamp order.
        let mut copy_effects: Vec<(Timestamp, usize)> = self
            .effects
            .iter()
            .enumerate()
            .filter(|(_, e)| matches!(e.layer1, Some(Layer1::Copy { .. })))
            .map(|(i, e)| (e.timestamp, i))
            .collect();
        copy_effects.sort();
        for (_, i) in copy_effects {
            let eff = self.effects[i].clone();
            let Affected::Objects(targets) = &eff.affected else {
                continue;
            };
            let Some(Layer1::Copy { values, exceptions }) = &eff.layer1 else {
                continue;
            };
            for t in targets {
                if !self.is_live(*t) {
                    continue;
                }
                let mut v = (**values).clone();
                v.abilities = v
                    .abilities
                    .iter()
                    .map(|a| copied_ability(a, eff.id))
                    .collect();
                let ctx = Ctx::new(eff.source, eff.controller);
                for m in exceptions {
                    apply_mod(&mut v, m, self, &ctx, *t);
                }
                self.objects[t.0 as usize].chars = v;
            }
        }

        // Layer 1b: face-down (CR 708.2).
        for id in &live {
            if self.obj(*id).face_down {
                let fd = crate::facedown::face_down_characteristics(self, *id);
                self.objects[id.0 as usize].chars = fd;
            }
        }
        for id in &live {
            let o = &mut self.objects[id.0 as usize];
            o.copiable = o.chars.clone();
        }

        // Layers 2–7.
        let mut started: HashMap<EffKey, Vec<ObjectId>> = HashMap::new();
        for layer in [
            Layer::L2Control,
            Layer::L3Text,
            Layer::L4Type,
            Layer::L5Color,
            Layer::L6Ability,
            Layer::L7aCda,
            Layer::L7bSet,
            Layer::L7cModify,
            Layer::L7dSwitch,
        ] {
            if layer == Layer::L7cModify {
                self.apply_pt_counters(&live);
            }
            self.apply_layer(layer, &live, &mut started);
            if layer == Layer::L4Type {
                // CR 305.6: basic land types have intrinsic mana abilities. They're
                // determined by the object's types after layer 4 (CR 305.7) and can be
                // removed by layer 6 effects.
                for id in &live {
                    let o = &self.objects[id.0 as usize];
                    if !o.chars.is_land() {
                        continue;
                    }
                    let extra: Vec<Ability> = o
                        .chars
                        .subtypes
                        .iter()
                        .filter_map(|s| intrinsic_mana_ability(s))
                        .collect();
                    self.objects[id.0 as usize].chars.abilities.extend(extra);
                }
            }
            if layer == Layer::L6Ability {
                self.apply_keyword_counters(&live);
                self.apply_cant_have(&live);
                // Keywords bring the abilities they stand for (CR 702).
                for id in &live {
                    let mut c = std::mem::take(&mut self.objects[id.0 as usize].chars);
                    crate::keyword_impls::expand_keywords(&mut c);
                    self.objects[id.0 as usize].chars = c;
                }
            }
        }

        // Post-processing.
        let turn = self.turn.number;
        for (id, prev) in prev_controllers {
            if !self.is_live(id) {
                continue;
            }
            let now = self.obj(id).controller;
            if now != prev {
                // CR 302.6: a control change resets summoning sickness; CR 506.4: removed from combat.
                self.objects[id.0 as usize].summoning_sick = true;
                crate::combat::remove_from_combat(self, id);
                self.events.push(crate::events::Event::ControlChanged {
                    obj: id,
                    from: prev,
                    to: now,
                });
            }
        }
        let _ = turn;
        for id in &live {
            let o = &mut self.objects[id.0 as usize];
            if o.chars.has_supertype(Supertype::World) {
                if o.world_since.is_none() {
                    o.world_since = Some(self.next_timestamp);
                }
            } else {
                o.world_since = None;
            }
        }
        self.collect_statics();
        self.compute_player_effects();
    }

    pub fn is_live(&self, id: ObjectId) -> bool {
        let o = self.obj(id);
        o.next.is_none() && o.zone != Zone::Nowhere
    }

    fn expire_dependent_effects(&mut self) {
        let mut remove: Vec<u32> = Vec::new();
        for e in &self.effects {
            if self.effect_expired(&e.duration, e.source, e.controller) {
                remove.push(e.id);
            } else if let (Duration::UntilHostLeaves, Affected::Objects(v)) =
                (&e.duration, &e.affected)
            {
                if v.iter()
                    .all(|o| !self.is_live(*o) || self.obj(*o).zone != Zone::Battlefield)
                {
                    remove.push(e.id);
                }
            }
        }
        let gone =
            |d: &Duration, s: Option<ObjectId>, c: PlayerId, g: &Game| g.effect_expired(d, s, c);
        let rm: Vec<u32> = self
            .rule_effects
            .iter()
            .filter(|e| gone(&e.duration, e.source, e.controller, self))
            .map(|e| e.id)
            .collect();
        let pm: Vec<u32> = self
            .player_effects
            .iter()
            .filter(|e| gone(&e.duration, e.source, e.controller, self))
            .map(|e| e.id)
            .collect();
        let rp: Vec<u32> = self
            .replacements
            .iter()
            .filter(|e| gone(&e.duration, e.source, e.controller, self))
            .map(|e| e.id)
            .collect();
        self.effects.retain(|e| !remove.contains(&e.id));
        self.rule_effects.retain(|e| !rm.contains(&e.id));
        self.player_effects.retain(|e| !pm.contains(&e.id));
        self.replacements.retain(|e| !rp.contains(&e.id));
    }

    fn effect_expired(&self, d: &Duration, source: Option<ObjectId>, controller: PlayerId) -> bool {
        match d {
            Duration::WhileSourceOnBattlefield => {
                source.is_none_or(|s| !self.is_live(s) || self.obj(s).zone != Zone::Battlefield)
            }
            Duration::WhileYouControlSource => source.is_none_or(|s| {
                !self.is_live(s)
                    || self.obj(s).zone != Zone::Battlefield
                    || self.obj(s).controller != controller
            }),
            Duration::WhileCondition(c) => !self.eval_cond(c, &Ctx::new(source, controller)),
            _ => false,
        }
    }

    /// Static abilities (with their sources) that could generate continuous effects,
    /// read from the objects' current (interim) abilities.
    fn static_sources(&self, live: &[ObjectId]) -> Vec<(ObjectId, Ability)> {
        let mut out = Vec::new();
        for id in live {
            let o = self.obj(*id);
            for a in &o.chars.abilities {
                if let AbilityKind::Static(s) = &a.kind {
                    if matches!(s.effect, StaticEffect::Continuous { .. })
                        && self.ability_functions(o, s.zone, s.is_cda)
                    {
                        out.push((*id, a.clone()));
                    }
                }
            }
        }
        out
    }

    fn apply_layer(
        &mut self,
        layer: Layer,
        live: &[ObjectId],
        started: &mut HashMap<EffKey, Vec<ObjectId>>,
    ) {
        // Gather effects with a modification in this layer.
        let mut effs: Vec<LayerEff> = Vec::new();
        for (i, e) in self.effects.iter().enumerate() {
            if e.layer1.is_none() && e.mods.iter().any(|m| m.layer() == layer) {
                effs.push(LayerEff {
                    key: EffKey::Resolved(i),
                    timestamp: e.timestamp,
                    cda: false,
                });
            }
        }
        let statics = self.static_sources(live);
        let mut static_map: HashMap<EffKey, Ability> = HashMap::new();
        for (src, a) in &statics {
            let AbilityKind::Static(s) = &a.kind else {
                continue;
            };
            let StaticEffect::Continuous { mods, .. } = &s.effect else {
                continue;
            };
            if !mods.iter().any(|m| m.layer() == layer) {
                continue;
            }
            let key = EffKey::Static(*src, a.uid);
            static_map.insert(key, a.clone());
            let ts = self.obj(*src).timestamp;
            effs.push(LayerEff {
                key,
                timestamp: ts,
                cda: s.is_cda,
            });
        }
        // Effects that started in an earlier layer continue even if the ability was
        // removed (CR 613.6).
        for (key, _) in started.iter() {
            if let EffKey::Static(src, uid) = key {
                if static_map.contains_key(key) {
                    continue;
                }
                if let Some(a) = self.find_static_ability(*src, *uid) {
                    let AbilityKind::Static(s) = &a.kind else {
                        continue;
                    };
                    let StaticEffect::Continuous { mods, .. } = &s.effect else {
                        continue;
                    };
                    if mods.iter().any(|m| m.layer() == layer) {
                        static_map.insert(*key, a.clone());
                        effs.push(LayerEff {
                            key: *key,
                            timestamp: self.obj(*src).timestamp,
                            cda: s.is_cda,
                        });
                    }
                }
            }
        }
        if effs.is_empty() {
            return;
        }
        // CR 613.3: CDAs first (layers 2–6, and 7a is CDA-only), then timestamp order.
        effs.sort_by_key(|e| (!e.cda, e.timestamp, key_order(&e.key)));

        let mut remaining = effs;
        while !remaining.is_empty() {
            let pick = if remaining.len() == 1 {
                0
            } else {
                self.pick_next_effect(&remaining, layer, live, started, &static_map)
            };
            let e = remaining.remove(pick);
            self.apply_effect_in_layer(&e.key, layer, live, started, &static_map);
        }
    }

    /// Chooses the next effect to apply within a layer, honoring dependencies (CR 613.8).
    fn pick_next_effect(
        &mut self,
        remaining: &[LayerEff],
        layer: Layer,
        live: &[ObjectId],
        started: &HashMap<EffKey, Vec<ObjectId>>,
        static_map: &HashMap<EffKey, Ability>,
    ) -> usize {
        // Only static effects with filters (or dynamic values) can be dependent; skip the
        // expensive trial application when nothing could depend.
        let any_dynamic = remaining
            .iter()
            .any(|e| matches!(e.key, EffKey::Static(..)));
        if !any_dynamic {
            return 0;
        }
        for (i, a) in remaining.iter().enumerate() {
            let mut depends_on_any = false;
            for (j, b) in remaining.iter().enumerate() {
                if i == j || a.cda != b.cda {
                    continue;
                }
                if self.depends_on(&a.key, &b.key, layer, live, started, static_map) {
                    // Dependency loop check: if b also depends on a, ignore (CR 613.8b).
                    if !self.depends_on(&b.key, &a.key, layer, live, started, static_map) {
                        depends_on_any = true;
                        break;
                    }
                }
            }
            if !depends_on_any {
                return i;
            }
        }
        0
    }

    /// Whether effect `a` depends on effect `b` in this layer (CR 613.8a): applying `b`
    /// would change `a`'s existence, what it applies to, or what it does.
    fn depends_on(
        &mut self,
        a: &EffKey,
        b: &EffKey,
        layer: Layer,
        live: &[ObjectId],
        started: &HashMap<EffKey, Vec<ObjectId>>,
        static_map: &HashMap<EffKey, Ability>,
    ) -> bool {
        let before = self.effect_footprint(a, layer, live, started, static_map);
        let snapshot: Vec<(ObjectId, Characteristics, PlayerId)> = live
            .iter()
            .map(|id| (*id, self.obj(*id).chars.clone(), self.obj(*id).controller))
            .collect();
        let mut scratch = started.clone();
        self.apply_effect_in_layer(b, layer, live, &mut scratch, static_map);
        let after = self.effect_footprint(a, layer, live, started, static_map);
        for (id, c, ctl) in snapshot {
            let o = &mut self.objects[id.0 as usize];
            o.chars = c;
            o.controller = ctl;
        }
        before != after
    }

    /// (exists, affected objects, evaluated numeric values) for dependency comparison.
    fn effect_footprint(
        &self,
        key: &EffKey,
        layer: Layer,
        live: &[ObjectId],
        started: &HashMap<EffKey, Vec<ObjectId>>,
        static_map: &HashMap<EffKey, Ability>,
    ) -> (bool, Vec<ObjectId>, Vec<i64>) {
        match key {
            EffKey::Resolved(i) => {
                let e = &self.effects[*i];
                let ctx = Ctx::new(e.source, e.controller);
                let affected = match &e.affected {
                    Affected::Objects(v) => v.clone(),
                    Affected::Filter(f) => live
                        .iter()
                        .copied()
                        .filter(|o| self.matches(*o, f, &ctx))
                        .collect(),
                };
                let vals = mod_values(self, &e.mods, layer, &ctx);
                (true, affected, vals)
            }
            EffKey::Static(src, uid) => {
                let exists = self.obj(*src).chars.abilities.iter().any(|a| a.uid == *uid);
                let Some(a) = static_map.get(key) else {
                    return (false, vec![], vec![]);
                };
                let AbilityKind::Static(s) = &a.kind else {
                    return (false, vec![], vec![]);
                };
                let StaticEffect::Continuous { affected, mods } = &s.effect else {
                    return (false, vec![], vec![]);
                };
                let ctx = Ctx::for_object(self, *src);
                let cond = s.condition.as_ref().is_none_or(|c| self.eval_cond(c, &ctx));
                let aff = match started.get(key) {
                    Some(v) => v.clone(),
                    None => self.static_affected(*src, affected, live, &ctx),
                };
                (exists && cond, aff, mod_values(self, mods, layer, &ctx))
            }
        }
    }

    fn static_affected(
        &self,
        src: ObjectId,
        affected: &Filter,
        live: &[ObjectId],
        ctx: &Ctx,
    ) -> Vec<ObjectId> {
        let zone = affected.zone();
        // A static ability that affects only its own object applies wherever that object
        // is when the ability functions there (e.g. characteristic-defining abilities,
        // CR 604.3; "this spell has flash as long as ...", CR 601.3d).
        let self_only = filter_is_self(affected);
        live.iter()
            .copied()
            .filter(|o| {
                let obj = self.obj(*o);
                let in_zone = match zone {
                    Some(z) => obj.zone.kind() == Some(z),
                    None => obj.zone == Zone::Battlefield || (self_only && *o == src),
                };
                in_zone && !obj.phased_out && self.matches(*o, affected, ctx)
            })
            .collect()
    }

    fn find_static_ability(&self, src: ObjectId, uid: u64) -> Option<Ability> {
        let o = self.obj(src);
        o.base
            .abilities
            .iter()
            .chain(o.copiable.abilities.iter())
            .find(|a| a.uid == uid)
            .cloned()
            .or_else(|| {
                self.effects
                    .iter()
                    .flat_map(|e| e.mods.iter())
                    .find_map(|m| match m {
                        Modification::AddAbility(a) if a.uid == uid => Some(a.clone()),
                        _ => None,
                    })
            })
    }

    fn apply_effect_in_layer(
        &mut self,
        key: &EffKey,
        layer: Layer,
        live: &[ObjectId],
        started: &mut HashMap<EffKey, Vec<ObjectId>>,
        static_map: &HashMap<EffKey, Ability>,
    ) {
        match key {
            EffKey::Resolved(i) => {
                let e = self.effects[*i].clone();
                let ctx = Ctx::new(e.source, e.controller);
                let affected = match started.get(key) {
                    Some(v) => v.clone(),
                    None => {
                        let v = match &e.affected {
                            Affected::Objects(v) => {
                                v.iter().copied().filter(|o| self.is_live(*o)).collect()
                            }
                            Affected::Filter(f) => self.static_affected(ObjectId(0), f, live, &ctx),
                        };
                        started.insert(*key, v.clone());
                        v
                    }
                };
                for t in affected {
                    if !self.is_live(t) {
                        continue;
                    }
                    for m in e.mods.iter().filter(|m| m.layer() == layer) {
                        self.apply_mod_to(t, m, &ctx);
                    }
                }
            }
            EffKey::Static(src, _uid) => {
                let Some(a) = static_map.get(key).cloned() else {
                    return;
                };
                let AbilityKind::Static(s) = &a.kind else {
                    return;
                };
                let StaticEffect::Continuous { affected, mods } = &s.effect else {
                    return;
                };
                let mut ctx = Ctx::for_object(self, *src);
                ctx.link = a.link;
                let affected_now = match started.get(key) {
                    Some(v) => v.clone(),
                    None => {
                        if let Some(c) = &s.condition {
                            if !self.eval_cond(c, &ctx) {
                                return;
                            }
                        }
                        let v = self.static_affected(*src, affected, live, &ctx);
                        started.insert(*key, v.clone());
                        v
                    }
                };
                for t in affected_now {
                    for m in mods.iter().filter(|m| m.layer() == layer) {
                        self.apply_mod_to(t, m, &ctx);
                    }
                }
            }
        }
    }

    fn apply_mod_to(&mut self, target: ObjectId, m: &Modification, ctx: &Ctx) {
        if let Modification::SetController(r) = m {
            if let Some(p) = self.eval_player(r, ctx) {
                self.objects[target.0 as usize].controller = p;
            }
            return;
        }
        let mut chars = std::mem::take(&mut self.objects[target.0 as usize].chars);
        apply_mod(&mut chars, m, self, ctx, target);
        self.objects[target.0 as usize].chars = chars;
    }

    /// +1/+1, -1/-1 and other P/T counters (CR 613.4c, 122.1a).
    fn apply_pt_counters(&mut self, live: &[ObjectId]) {
        for id in live {
            let o = &self.objects[id.0 as usize];
            if o.zone != Zone::Battlefield || o.counters.is_empty() {
                continue;
            }
            let mut dp = 0i32;
            let mut dt = 0i32;
            for (k, n) in &o.counters {
                if let Some((p, t)) = parse_pt_counter(k) {
                    dp += p * *n as i32;
                    dt += t * *n as i32;
                }
            }
            if dp != 0 || dt != 0 {
                let o = &mut self.objects[id.0 as usize];
                if let Some(p) = o.chars.power.as_mut() {
                    *p += dp;
                }
                if let Some(t) = o.chars.toughness.as_mut() {
                    *t += dt;
                }
            }
        }
    }

    /// Keyword counters (flying counters, etc.) grant abilities in layer 6 (CR 122.1b, 613.1f).
    fn apply_keyword_counters(&mut self, live: &[ObjectId]) {
        for id in live {
            let o = &self.objects[id.0 as usize];
            if o.zone != Zone::Battlefield || o.counters.is_empty() {
                continue;
            }
            let mut add: Vec<Keyword> = Vec::new();
            for (k, n) in &o.counters {
                if *n == 0 {
                    continue;
                }
                if let Some(kw) = keyword_counter(k) {
                    add.push(kw);
                }
            }
            for kw in add {
                let a = keyword_counter_ability(&kw);
                self.objects[id.0 as usize].chars.abilities.push(a);
            }
        }
    }

    /// "can't have [ability]" effects remove the ability after all layer 6 effects.
    fn apply_cant_have(&mut self, _live: &[ObjectId]) {}

    /// Collects functioning non-characteristic static abilities for rule queries.
    fn collect_statics(&mut self) {
        let mut st = ActiveStatics::default();
        let ids: Vec<ObjectId> = self.live_objects();
        for id in ids {
            let o = self.obj(id);
            for a in &o.chars.abilities {
                let AbilityKind::Static(s) = &a.kind else {
                    continue;
                };
                if !self.ability_functions(o, s.zone, s.is_cda) {
                    continue;
                }
                if let Some(c) = &s.condition {
                    if !self.eval_cond(c, &Ctx::new(Some(id), o.controller)) {
                        continue;
                    }
                }
                let ctl = o.controller;
                match &s.effect {
                    StaticEffect::Continuous { .. } => {}
                    StaticEffect::Restriction(r) => st.restrictions.push((id, ctl, r.clone())),
                    StaticEffect::CostModifier(c) => st.cost_modifiers.push((id, ctl, c.clone())),
                    StaticEffect::Replacement(r) => {
                        st.replacements
                            .push((id, ctl, o.timestamp, a.clone(), r.clone()))
                    }
                    StaticEffect::PlayPermission(p) => {
                        st.play_permissions.push((id, ctl, p.clone()))
                    }
                    StaticEffect::FlashPermission { who, what } => {
                        st.flash_permissions.push((id, ctl, *who, what.clone()))
                    }
                    StaticEffect::Custom(n) => st.customs.push((id, ctl, n.clone())),
                    other => st.other.push((id, ctl, other.clone())),
                }
            }
        }
        self.statics = st;
    }

    /// Player-affecting effects (CR 613.10): hexproof, hand size, land plays, ...
    fn compute_player_effects(&mut self) {
        let n = self.players.len();
        let mut mods: Vec<Vec<PlayerModification>> = vec![vec![]; n];
        for (src, ctl, e) in self.statics.other.clone() {
            match e {
                StaticEffect::PlayerEffect { affected, effect } => {
                    let ctx = Ctx::new(Some(src), ctl);
                    for p in self.player_ids() {
                        if self.player_filter_matches(&affected, p, &ctx) {
                            mods[p.idx()].push(effect.clone());
                        }
                    }
                }
                StaticEffect::AdditionalLandPlays(rel, k) => {
                    let ctx = Ctx::new(Some(src), ctl);
                    for p in self.player_ids() {
                        if self.player_rel_matches(rel, p, &ctx) {
                            mods[p.idx()].push(PlayerModification::AdditionalLandPlays(k));
                        }
                    }
                }
                _ => {}
            }
        }
        let mut pe = self.player_effects.clone();
        pe.sort_by_key(|e| e.timestamp);
        for e in pe {
            for p in e.players {
                mods[p.idx()].push(e.effect.clone());
            }
        }
        for (i, m) in mods.into_iter().enumerate() {
            let mut max_hand: Option<i32> = Some(7);
            let mut land_plays = 1u32;
            let mut delta = 0i32;
            for x in &m {
                match x {
                    PlayerModification::MaxHandSize(v) => {
                        max_hand = v
                            .as_ref()
                            .map(|v| self.eval_value(v, &Ctx::new(None, PlayerId(i as u8))) as i32)
                    }
                    PlayerModification::HandSizeDelta(d) => delta += d,
                    PlayerModification::AdditionalLandPlays(k) => land_plays += k,
                    _ => {}
                }
            }
            // Vanguard hand modifier (CR 902.3) handled by the variant module via HandSizeDelta.
            let p = &mut self.players[i];
            p.max_hand_size = max_hand.map(|h| h + delta);
            p.land_plays = land_plays;
            p.mods = m;
        }
    }
}

/// Whether a filter can only match the ability's own source object.
fn filter_is_self(f: &Filter) -> bool {
    match f {
        Filter::Source => true,
        Filter::And(v) => v.iter().any(filter_is_self),
        _ => false,
    }
}

fn key_order(k: &EffKey) -> (u8, u64) {
    match k {
        EffKey::Resolved(i) => (0, *i as u64),
        EffKey::Static(o, u) => (1, (o.0 as u64) << 32 | (*u & 0xffff_ffff)),
    }
}

fn mod_values(g: &Game, mods: &[Modification], layer: Layer, ctx: &Ctx) -> Vec<i64> {
    let mut out = Vec::new();
    for m in mods.iter().filter(|m| m.layer() == layer) {
        match m {
            Modification::CdaPT(p, t) | Modification::SetPT(p, t) => {
                out.push(p.as_ref().map_or(i64::MIN, |v| g.eval_value(v, ctx)));
                out.push(t.as_ref().map_or(i64::MIN, |v| g.eval_value(v, ctx)));
            }
            Modification::ModifyPT(p, t) => {
                out.push(g.eval_value(p, ctx));
                out.push(g.eval_value(t, ctx));
            }
            _ => {}
        }
    }
    out
}

/// Parses a P/T counter kind like "+1/+1", "-1/-1", "+1/+0", "-0/-1".
pub fn parse_pt_counter(k: &str) -> Option<(i32, i32)> {
    let (p, t) = k.split_once('/')?;
    let p: i32 = p.trim_start_matches('+').replace("−", "-").parse().ok()?;
    let t: i32 = t.trim_start_matches('+').replace("−", "-").parse().ok()?;
    if !k.starts_with('+') && !k.starts_with('-') {
        return None;
    }
    Some((p, t))
}

/// Keyword counters (CR 122.1b).
pub fn keyword_counter(k: &str) -> Option<Keyword> {
    let kind = match k {
        "flying" => KeywordKind::Flying,
        "first strike" => KeywordKind::FirstStrike,
        "double strike" => KeywordKind::DoubleStrike,
        "deathtouch" => KeywordKind::Deathtouch,
        "decayed" => KeywordKind::Decayed,
        "exalted" => KeywordKind::Exalted,
        "haste" => KeywordKind::Haste,
        "hexproof" => KeywordKind::Hexproof,
        "indestructible" => KeywordKind::Indestructible,
        "lifelink" => KeywordKind::Lifelink,
        "menace" => KeywordKind::Menace,
        "reach" => KeywordKind::Reach,
        "shadow" => KeywordKind::Shadow,
        "trample" => KeywordKind::Trample,
        "vigilance" => KeywordKind::Vigilance,
        _ => return None,
    };
    Some(Keyword::new(kind))
}

fn keyword_counter_ability(kw: &Keyword) -> Ability {
    use std::sync::OnceLock;
    static CACHE: OnceLock<std::sync::Mutex<HashMap<KeywordKind, Ability>>> = OnceLock::new();
    let m = CACHE.get_or_init(Default::default);
    let mut g = m.lock().unwrap();
    g.entry(kw.kind)
        .or_insert_with(|| AbilityDef::new(AbilityKind::Keyword(kw.clone()), kw.kind.name()))
        .clone()
}

/// Applies a single layer modification to a set of characteristics.
pub fn apply_mod(
    c: &mut Characteristics,
    m: &Modification,
    g: &Game,
    ctx: &Ctx,
    _target: ObjectId,
) {
    match m {
        Modification::SetController(_) => {}
        Modification::ChangeText { from, to } => crate::text_change::change_text(c, from, to),
        Modification::AddTypes(ts) => {
            for t in ts {
                c.card_types.insert(*t);
            }
        }
        Modification::RemoveTypes(ts) => {
            for t in ts {
                c.card_types.remove(*t);
            }
            // Subtypes that no longer correspond to a card type are removed (CR 205.1b-ish).
            let types = c.card_types;
            c.subtypes.retain(|s| subtype_still_valid(s, types));
        }
        Modification::AddSupertypes(ts) => {
            for t in ts {
                c.supertypes.insert(*t);
            }
        }
        Modification::RemoveSupertypes(ts) => {
            for t in ts {
                c.supertypes.remove(*t);
            }
        }
        Modification::AddSubtypes(ss) => {
            for s in ss {
                if !c.subtypes.contains(s) {
                    c.subtypes.push(s.clone());
                }
            }
        }
        Modification::RemoveSubtypes(ss) => c.subtypes.retain(|s| !ss.contains(s)),
        Modification::SetTypes { types, subtypes } => {
            c.card_types = types.iter().copied().collect();
            c.subtypes = subtypes.iter().cloned().collect::<SmallVec<[Subtype; 3]>>();
        }
        Modification::AllCreatureTypes => {
            // Represented by adding the Changeling marker ability semantics: we add all
            // creature types explicitly (CR 205.3m, 702.73a).
            for s in &subtype_lists().creature {
                let s = SmolStr::new(s);
                if !c.subtypes.contains(&s) {
                    c.subtypes.push(s);
                }
            }
        }
        Modification::RemoveAllCreatureTypes => c.subtypes.retain(|s| !is_creature_type(s)),
        Modification::SetBasicLandType(ts) => {
            // CR 305.7: loses all land types and abilities from its rules text, gains the
            // basic land type(s) and their intrinsic mana abilities.
            c.subtypes
                .retain(|s| !subtype_lists().land.contains(s.as_str()));
            for t in ts {
                c.subtypes.push(t.clone());
            }
            c.abilities.clear();
        }
        Modification::SetColors(cs) => c.colors = *cs,
        Modification::AddColors(cs) => c.colors = c.colors.union(*cs),
        Modification::AddAbility(a) => c.abilities.push(acquired_ability(a, ctx.source, _target)),
        Modification::AddKeyword(k) => {
            // "Protection from the chosen color": the choice is the granting ability's
            // (CR 607.2d); an undefined choice grants nothing (CR 607.5a).
            let mut k = k.clone();
            if let Some(f) = &k.filter {
                match resolve_chosen(f, g, ctx) {
                    Some(f) => k.filter = Some(f),
                    None => return,
                }
            }
            c.abilities.push(AbilityDef::new(
                AbilityKind::Keyword(k.clone()),
                k.kind.name(),
            ))
        }
        Modification::RemoveKeyword(k) => c
            .abilities
            .retain(|a| !matches!(&a.kind, AbilityKind::Keyword(kw) if kw.kind == *k)),
        Modification::RemoveAllAbilities => c.abilities.clear(),
        Modification::CantHaveKeyword(k) => c
            .abilities
            .retain(|a| !matches!(&a.kind, AbilityKind::Keyword(kw) if kw.kind == *k)),
        Modification::CdaPT(p, t) | Modification::SetPT(p, t) => {
            if let Some(p) = p {
                c.power = Some(g.eval_value(p, ctx) as i32);
            }
            if let Some(t) = t {
                c.toughness = Some(g.eval_value(t, ctx) as i32);
            }
        }
        Modification::ModifyPT(p, t) => {
            let dp = g.eval_value(p, ctx) as i32;
            let dt = g.eval_value(t, ctx) as i32;
            if let Some(x) = c.power.as_mut() {
                *x += dp;
            }
            if let Some(x) = c.toughness.as_mut() {
                *x += dt;
            }
        }
        Modification::SwitchPT => {
            if let (Some(p), Some(t)) = (c.power, c.toughness) {
                c.power = Some(t);
                c.toughness = Some(p);
            }
        }
    }
}

/// An ability an object acquires from another object. It's a distinct ability from
/// identically worded abilities the object has or acquires from other objects, so
/// restrictions on its use apply only to it as acquired from that object (CR 602.5c),
/// and linked abilities acquired together are linked only to each other (CR 607.5).
/// Derived abilities are cached so their identity is stable across recomputation.
pub fn acquired_ability(a: &Ability, from: Option<ObjectId>, target: ObjectId) -> Ability {
    use std::sync::{Mutex, OnceLock};
    let Some(src) = from.filter(|s| *s != target) else {
        return a.clone();
    };
    static CACHE: OnceLock<Mutex<HashMap<(u64, u32), Ability>>> = OnceLock::new();
    let m = CACHE.get_or_init(Default::default);
    let mut g = m.lock().unwrap();
    g.entry((a.uid, src.0))
        .or_insert_with(|| {
            // A link id distinct from those of printed abilities and of abilities acquired
            // from other objects (CR 607.5).
            let link = 0x8000 | ((a.link as u32 * 131 + src.0 * 31) % 0x7fff) as u16;
            AbilityDef::with_link(a.kind.clone(), a.text.clone(), link)
        })
        .clone()
}

/// Replaces "the chosen [value]" in a filter with the value chosen by the linked ability
/// of `ctx.source`. None if the choice is undefined (CR 607.5a).
fn resolve_chosen(f: &Filter, g: &Game, ctx: &Ctx) -> Option<Filter> {
    Some(match f {
        Filter::LinkedChosenColor => Filter::Color(g.linked_choice(ctx)?.color?),
        Filter::LinkedChosenCreatureType => Filter::Subtype(g.linked_choice(ctx)?.creature_type.clone()?),
        Filter::And(v) => Filter::And(
            v.iter()
                .map(|x| resolve_chosen(x, g, ctx))
                .collect::<Option<Vec<_>>>()?,
        ),
        Filter::Or(v) => Filter::Or(
            v.iter()
                .map(|x| resolve_chosen(x, g, ctx))
                .collect::<Option<Vec<_>>>()?,
        ),
        Filter::Not(x) => Filter::Not(Box::new(resolve_chosen(x, g, ctx)?)),
        other => other.clone(),
    })
}

/// Abilities an object has because of a copy effect are linked to each other, not to
/// abilities the object had from another copy effect (CR 607.5, 607.5a). Cached so their
/// identity is stable across recomputation.
fn copied_ability(a: &Ability, effect: u32) -> Ability {
    use std::sync::{Mutex, OnceLock};
    static CACHE: OnceLock<Mutex<HashMap<(u64, u32), Ability>>> = OnceLock::new();
    let m = CACHE.get_or_init(Default::default);
    let mut g = m.lock().unwrap();
    g.entry((a.uid, effect))
        .or_insert_with(|| {
            let link = 0x4000 | ((a.link as u32 * 131 + effect * 37) % 0x3fff) as u16;
            AbilityDef::with_link(a.kind.clone(), a.text.clone(), link)
        })
        .clone()
}

fn subtype_still_valid(s: &str, types: CardTypeSet) -> bool {
    match subtype_kind(s) {
        Some(SubtypeKind::Creature) => {
            types.contains(CardType::Creature) || types.contains(CardType::Kindred)
        }
        Some(SubtypeKind::Land) => types.contains(CardType::Land),
        Some(SubtypeKind::Artifact) => types.contains(CardType::Artifact),
        Some(SubtypeKind::Enchantment) => types.contains(CardType::Enchantment),
        Some(SubtypeKind::Planeswalker) => types.contains(CardType::Planeswalker),
        Some(SubtypeKind::Spell) => {
            types.contains(CardType::Instant) || types.contains(CardType::Sorcery)
        }
        Some(SubtypeKind::Battle) => types.contains(CardType::Battle),
        Some(SubtypeKind::Plane) => types.contains(CardType::Plane),
        _ => true,
    }
}

/// Convenience used by tests: build an ability granting a keyword.
pub fn keyword_ability(k: KeywordKind) -> Ability {
    AbilityDef::new(AbilityKind::Keyword(Keyword::new(k)), k.name())
}

pub fn arc_chars(c: Characteristics) -> Arc<Characteristics> {
    Arc::new(c)
}

/// The intrinsic "{T}: Add [mana]" ability of a basic land type (CR 305.6).
pub fn intrinsic_mana_ability(land_type: &str) -> Option<Ability> {
    use crate::mana::ManaType;
    use std::sync::OnceLock;
    static CACHE: OnceLock<[Ability; 5]> = OnceLock::new();
    let abilities = CACHE.get_or_init(|| {
        let make = |t: ManaType, sym: &str| {
            let mut a = ActivatedAbility::new(
                Cost::tap(),
                Body::effect(Effect::AddMana {
                    who: PlayerRef::You,
                    mana: ManaProduction::Fixed(vec![t]),
                    restriction: None,
                }),
            );
            a.is_mana_ability = true;
            AbilityDef::new(AbilityKind::Activated(a), format!("{{T}}: Add {{{sym}}}."))
        };
        [
            make(ManaType::W, "W"),
            make(ManaType::U, "U"),
            make(ManaType::B, "B"),
            make(ManaType::R, "R"),
            make(ManaType::G, "G"),
        ]
    });
    let i = match land_type {
        "Plains" => 0,
        "Island" => 1,
        "Swamp" => 2,
        "Mountain" => 3,
        "Forest" => 4,
        _ => return None,
    };
    Some(abilities[i].clone())
}
