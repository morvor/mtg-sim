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
    /// The keyword counters of one kind on an object (index into [`KEYWORD_COUNTERS`]),
    /// which grant their keyword in layer 6 at the counters' timestamp (CR 613.1f, 613.7c).
    KwCounter(ObjectId, u8),
}

#[derive(Clone, Debug)]
struct LayerEff {
    key: EffKey,
    /// Timestamp order (CR 613.7): the timestamp, then a secondary order for abilities
    /// granted to an object (see [`Game::static_timestamp`]).
    ts: (Timestamp, Timestamp),
    cda: bool,
}

/// Bookkeeping for one characteristic computation, shared across layers.
#[derive(Default)]
struct LayerState {
    /// Objects each effect applied to when it started to apply (CR 613.6); static
    /// abilities keep applying to them in later layers even if the ability is removed.
    started: HashMap<EffKey, Vec<ObjectId>>,
    /// The ability each static-ability effect comes from.
    abilities: HashMap<EffKey, Ability>,
    /// Timestamp of the effect that granted an ability to an object (CR 613.7a).
    grants: HashMap<(ObjectId, u64), Timestamp>,
}

/// Counter kinds that grant keyword abilities (CR 122.1b).
pub const KEYWORD_COUNTERS: [&str; 15] = [
    "flying",
    "first strike",
    "double strike",
    "deathtouch",
    "decayed",
    "exalted",
    "haste",
    "hexproof",
    "indestructible",
    "lifelink",
    "menace",
    "reach",
    "shadow",
    "trample",
    "vigilance",
];

fn is_cant_have(m: &Modification) -> bool {
    matches!(m, Modification::CantHaveKeyword(_))
}

/// Whether any of the modifications applies in `layer` (other than "can't have"
/// modifications, which are applied at the end of layer 6).
fn has_layer_mod(mods: &[Modification], layer: Layer) -> bool {
    mods.iter().any(|m| m.layer() == layer && !is_cant_have(m))
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
        if let FunctionZone::AnywhereExcept(z) = zone {
            return obj.zone.kind() != Some(z) && !obj.phased_out;
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
        // A face-down card in the command zone (a card in a planar or scheme deck, a
        // hidden agenda) has no functioning abilities.
        if obj.face_down {
            return false;
        }
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
        self.compute_characteristics(true);
    }

    /// Computes every object's characteristics, then the rule-modifying effects. With
    /// `side_effects` false (a hypothetical computation, e.g. of how a permanent would
    /// exist on the battlefield, CR 614.12), changes of control aren't acted on.
    pub(crate) fn compute_characteristics(&mut self, side_effects: bool) {
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
            .filter(|(_, e)| e.layer1.is_some())
            .map(|(i, e)| (e.timestamp, i))
            .collect();
        copy_effects.sort();
        for (_, i) in copy_effects {
            let eff = self.effects[i].clone();
            let Affected::Objects(targets) = &eff.affected else {
                continue;
            };
            if let Some(Layer1::Copiable(mods)) = &eff.layer1 {
                let ctx = Ctx::new(eff.source, eff.controller);
                for t in targets {
                    if self.is_live(*t) {
                        let mut c = self.objects[t.0 as usize].chars.clone();
                        for m in mods {
                            apply_mod(&mut c, m, self, &ctx, *t);
                        }
                        self.objects[t.0 as usize].chars = c;
                    }
                }
                continue;
            }
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
        let mut st = LayerState::default();
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
            self.apply_layer(layer, &live, &mut st);
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
                self.apply_cant_have(&live, &mut st);
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
        let prev_controllers = if side_effects {
            prev_controllers
        } else {
            vec![]
        };
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
        // CR 506.4: type changes can remove permanents from combat.
        crate::combat::update_combat_membership(self);
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
            } else if matches!(&e.affected, Affected::Objects(v) if v.iter().all(|o| !self.is_live(*o)))
            {
                // Every object it applied to has left its zone: it can never apply again
                // (CR 400.7).
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

    pub(crate) fn effect_expired(
        &self,
        d: &Duration,
        source: Option<ObjectId>,
        controller: PlayerId,
    ) -> bool {
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

    /// Gathers the continuous effects that apply in `layer` and haven't been applied yet
    /// in it: effects from resolved spells and abilities, effects of static abilities as
    /// the objects' interim characteristics currently stand (so an ability removed by an
    /// earlier effect in the layer no longer generates an effect, and one granted by an
    /// earlier effect does, CR 613.8a), effects that started to apply in an earlier layer
    /// (CR 613.6), and keyword counters (CR 613.1f, 613.7c).
    fn layer_candidates(
        &self,
        layer: Layer,
        live: &[ObjectId],
        st: &mut LayerState,
        done: &std::collections::HashSet<EffKey>,
    ) -> Vec<LayerEff> {
        let mut effs: Vec<LayerEff> = Vec::new();
        for (i, e) in self.effects.iter().enumerate() {
            let key = EffKey::Resolved(i);
            if e.layer1.is_none() && has_layer_mod(&e.mods, layer) && !done.contains(&key) {
                effs.push(LayerEff {
                    key,
                    ts: crate::stickers::effect_timestamp(self, e),
                    cda: false,
                });
            }
        }
        for id in live {
            let o = self.obj(*id);
            for a in &o.chars.abilities {
                let AbilityKind::Static(s) = &a.kind else {
                    continue;
                };
                let StaticEffect::Continuous { mods, .. } = &s.effect else {
                    continue;
                };
                if !has_layer_mod(mods, layer)
                    || !self.ability_functions(o, s.zone, s.is_cda)
                    || crate::next_spell::is_cast_grant(s)
                {
                    continue;
                }
                let key = EffKey::Static(*id, a.uid);
                if done.contains(&key) || effs.iter().any(|e| e.key == key) {
                    continue;
                }
                st.abilities.insert(key, a.clone());
                effs.push(LayerEff {
                    key,
                    ts: self.static_timestamp(*id, a.uid, st),
                    cda: s.is_cda,
                });
            }
        }
        // Effects that started in an earlier layer continue even if the ability generating
        // them was removed (CR 613.6).
        let started: Vec<EffKey> = st.started.keys().copied().collect();
        for key in started {
            let EffKey::Static(src, uid) = key else {
                continue;
            };
            if done.contains(&key) || effs.iter().any(|e| e.key == key) {
                continue;
            }
            let Some(a) = st.abilities.get(&key) else {
                continue;
            };
            let AbilityKind::Static(s) = &a.kind else {
                continue;
            };
            let StaticEffect::Continuous { mods, .. } = &s.effect else {
                continue;
            };
            if has_layer_mod(mods, layer) {
                effs.push(LayerEff {
                    key,
                    ts: self.static_timestamp(src, uid, st),
                    cda: s.is_cda,
                });
            }
        }
        if layer == Layer::L6Ability {
            for id in live {
                let o = self.obj(*id);
                if o.zone != Zone::Battlefield {
                    continue;
                }
                for (k, n) in &o.counters {
                    if *n == 0 {
                        continue;
                    }
                    let Some(idx) = KEYWORD_COUNTERS.iter().position(|c| *c == k.as_str()) else {
                        continue;
                    };
                    let key = EffKey::KwCounter(*id, idx as u8);
                    if done.contains(&key) {
                        continue;
                    }
                    let ts = o.counter_timestamps.get(k).copied().unwrap_or(o.timestamp);
                    effs.push(LayerEff {
                        key,
                        ts: (ts, 0),
                        cda: false,
                    });
                }
            }
        }
        effs
    }

    /// Timestamp order of a static ability's effect (CR 613.7a): the later of the
    /// object's timestamp and the timestamp of the effect that granted the ability. When
    /// the object gets a new timestamp, abilities it was granted keep their order after
    /// its own abilities.
    fn static_timestamp(&self, src: ObjectId, uid: u64, st: &LayerState) -> (Timestamp, Timestamp) {
        let obj_ts = self.obj(src).timestamp;
        match st.grants.get(&(src, uid)) {
            Some(g) => (obj_ts.max(*g), *g),
            None => (obj_ts, 0),
        }
    }

    fn apply_layer(&mut self, layer: Layer, live: &[ObjectId], st: &mut LayerState) {
        let mut done: std::collections::HashSet<EffKey> = std::collections::HashSet::new();
        // Each iteration re-gathers the effects and re-evaluates dependencies, since
        // applying an effect can change which effects exist and how they depend on each
        // other (CR 613.8c).
        for _ in 0..10_000 {
            let mut effs = self.layer_candidates(layer, live, st, &done);
            if effs.is_empty() {
                break;
            }
            // CR 613.3, 613.4a: characteristic-defining abilities apply first.
            if effs.iter().any(|e| e.cda) {
                effs.retain(|e| e.cda);
            }
            effs.sort_by_key(|e| (e.ts, key_order(&e.key)));
            let pick = if effs.len() == 1 {
                0
            } else {
                self.pick_next_effect(&effs, layer, live, st)
            };
            let e = effs.swap_remove(pick);
            done.insert(e.key);
            self.apply_effect_in_layer(&e, layer, live, st, false);
        }
    }

    /// Chooses the next effect to apply within a layer (CR 613.8b): an effect waits until
    /// every effect it depends on has been applied, except that effects in a dependency
    /// loop are applied in timestamp order. `effs` is sorted by timestamp.
    fn pick_next_effect(
        &mut self,
        effs: &[LayerEff],
        layer: Layer,
        live: &[ObjectId],
        st: &mut LayerState,
    ) -> usize {
        let n = effs.len();
        // Only static abilities can depend on other effects: resolved effects have a
        // locked set of affected objects and fixed values (CR 611.2c, 608.2h), and keyword
        // counters exist regardless of other effects.
        if !effs.iter().any(|e| matches!(e.key, EffKey::Static(..))) {
            return 0;
        }
        let mut dep = vec![vec![false; n]; n];
        for i in 0..n {
            if !matches!(effs[i].key, EffKey::Static(..)) {
                continue;
            }
            for j in 0..n {
                // CR 613.8a(c): an effect from a characteristic-defining ability and one
                // that isn't are independent of each other.
                if i != j
                    && effs[i].cda == effs[j].cda
                    && self.depends_on(&effs[i], &effs[j], layer, live, st)
                {
                    dep[i][j] = true;
                }
            }
        }
        // Transitive closure: reach[i][j] = i depends (possibly indirectly) on j.
        let mut reach = dep.clone();
        for k in 0..n {
            for i in 0..n {
                if reach[i][k] {
                    for j in 0..n {
                        if reach[k][j] {
                            reach[i][j] = true;
                        }
                    }
                }
            }
        }
        // An effect may apply once everything it depends on outside its own dependency
        // loop has been applied.
        (0..n)
            .find(|&i| (0..n).all(|j| !dep[i][j] || reach[j][i]))
            .unwrap_or(0)
    }

    /// Whether effect `a` depends on effect `b` in this layer (CR 613.8a): applying `b`
    /// would change `a`'s text or existence, what it applies to, or what it does to any of
    /// the things it applies to.
    fn depends_on(
        &mut self,
        a: &LayerEff,
        b: &LayerEff,
        layer: Layer,
        live: &[ObjectId],
        st: &mut LayerState,
    ) -> bool {
        let before = self.effect_footprint(a, layer, live, st);
        // Applying `b` only changes the objects it affects.
        let touched: Vec<ObjectId> = self.effect_footprint(b, layer, live, st).1;
        let snapshot: Vec<(ObjectId, Characteristics, PlayerId)> = touched
            .iter()
            .map(|id| (*id, self.obj(*id).chars.clone(), self.obj(*id).controller))
            .collect();
        self.apply_effect_in_layer(b, layer, live, st, true);
        let after = self.effect_footprint(a, layer, live, st);
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
        e: &LayerEff,
        layer: Layer,
        live: &[ObjectId],
        st: &LayerState,
    ) -> (bool, Vec<ObjectId>, Vec<i64>) {
        match &e.key {
            EffKey::Resolved(i) => {
                let eff = &self.effects[*i];
                let ctx = Ctx::new(eff.source, eff.controller);
                let affected = match st.started.get(&e.key) {
                    Some(v) => v.clone(),
                    None => match &eff.affected {
                        Affected::Objects(v) => v.clone(),
                        Affected::Filter(f) => self.static_affected(ObjectId(0), f, live, &ctx),
                    },
                };
                let vals = mod_values(self, &eff.mods, layer, &ctx);
                (true, affected, vals)
            }
            EffKey::Static(src, uid) => {
                let Some(a) = st.abilities.get(&e.key) else {
                    return (false, vec![], vec![]);
                };
                let AbilityKind::Static(s) = &a.kind else {
                    return (false, vec![], vec![]);
                };
                let StaticEffect::Continuous { affected, mods } = &s.effect else {
                    return (false, vec![], vec![]);
                };
                // Same context as when the effect is applied (see `apply_effect_in_layer`).
                let mut ctx = Ctx::for_object(self, *src);
                ctx.link = a.link;
                let started = st.started.get(&e.key);
                let o = self.obj(*src);
                let exists = started.is_some()
                    || (o.chars.abilities.iter().any(|x| x.uid == *uid)
                        && self.ability_functions(o, s.zone, s.is_cda)
                        && s.condition.as_ref().is_none_or(|c| self.eval_cond(c, &ctx)));
                let aff = match started {
                    Some(v) => v.clone(),
                    None => self.static_affected(*src, affected, live, &ctx),
                };
                (exists, aff, mod_values(self, mods, layer, &ctx))
            }
            EffKey::KwCounter(obj, _) => (true, vec![*obj], vec![]),
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

    /// Applies one effect's modifications for this layer. A trial application (for
    /// dependency checks) doesn't record anything.
    fn apply_effect_in_layer(
        &mut self,
        e: &LayerEff,
        layer: Layer,
        live: &[ObjectId],
        st: &mut LayerState,
        trial: bool,
    ) {
        match &e.key {
            EffKey::Resolved(i) => {
                let eff = self.effects[*i].clone();
                let ctx = Ctx::new(eff.source, eff.controller);
                let affected = match st.started.get(&e.key) {
                    Some(v) => v.clone(),
                    None => {
                        let v = match &eff.affected {
                            Affected::Objects(v) => {
                                v.iter().copied().filter(|o| self.is_live(*o)).collect()
                            }
                            Affected::Filter(f) => self.static_affected(ObjectId(0), f, live, &ctx),
                        };
                        if !trial {
                            st.started.insert(e.key, v.clone());
                        }
                        v
                    }
                };
                for t in affected {
                    if !self.is_live(t) {
                        continue;
                    }
                    self.apply_mods_to(t, &eff.mods, layer, &ctx, e.ts.0, st, trial);
                }
            }
            EffKey::Static(src, _uid) => {
                let Some(a) = st.abilities.get(&e.key).cloned() else {
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
                let affected_now = match st.started.get(&e.key) {
                    Some(v) => v.clone(),
                    None => {
                        if let Some(c) = &s.condition {
                            if !self.eval_cond(c, &ctx) {
                                return;
                            }
                        }
                        let v = self.static_affected(*src, affected, live, &ctx);
                        if !trial {
                            st.started.insert(e.key, v.clone());
                        }
                        v
                    }
                };
                for t in affected_now {
                    self.apply_mods_to(t, mods, layer, &ctx, e.ts.0, st, trial);
                }
            }
            EffKey::KwCounter(obj, idx) => {
                if let Some(kw) = KEYWORD_COUNTERS
                    .get(*idx as usize)
                    .and_then(|k| keyword_counter(k))
                {
                    let a = keyword_counter_ability(&kw);
                    self.objects[obj.0 as usize].chars.abilities.push(a);
                }
            }
        }
    }

    /// Applies an effect's modifications for `layer` to one object, recording the
    /// timestamp of any ability it grants (CR 613.7a). "Can't have" modifications are
    /// applied after all other layer 6 effects (see [`Game::apply_cant_have`]).
    #[allow(clippy::too_many_arguments)]
    fn apply_mods_to(
        &mut self,
        t: ObjectId,
        mods: &[Modification],
        layer: Layer,
        ctx: &Ctx,
        ts: Timestamp,
        st: &mut LayerState,
        trial: bool,
    ) {
        for m in mods
            .iter()
            .filter(|m| m.layer() == layer && !is_cant_have(m))
        {
            let before = self.obj(t).chars.abilities.len();
            self.apply_mod_to(t, m, ctx);
            if !trial && matches!(m, Modification::AddAbility(_)) {
                let after = self.obj(t).chars.abilities.len();
                for k in before..after {
                    let uid = self.obj(t).chars.abilities[k].uid;
                    st.grants.insert((t, uid), ts);
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
        // Values are evaluated against the current interim characteristics, including the
        // object's own (e.g. a CDA counting the creatures its controller controls).
        let mut chars = self.objects[target.0 as usize].chars.clone();
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

    /// "Can't have [ability]" effects (CR 613.1f) are applied after every other layer 6
    /// effect, including keyword counters, so the object doesn't have the ability no
    /// matter when it was granted.
    fn apply_cant_have(&mut self, live: &[ObjectId], st: &mut LayerState) {
        let mut work: Vec<(Vec<ObjectId>, Vec<Modification>, Ctx)> = Vec::new();
        for (i, e) in self.effects.iter().enumerate() {
            if e.layer1.is_some() || !e.mods.iter().any(is_cant_have) {
                continue;
            }
            let ctx = Ctx::new(e.source, e.controller);
            let affected = match st.started.get(&EffKey::Resolved(i)) {
                Some(v) => v.clone(),
                None => match &e.affected {
                    Affected::Objects(v) => v.clone(),
                    Affected::Filter(f) => self.static_affected(ObjectId(0), f, live, &ctx),
                },
            };
            work.push((affected, e.mods.clone(), ctx));
        }
        for id in live {
            let o = self.obj(*id);
            for a in &o.chars.abilities {
                let AbilityKind::Static(s) = &a.kind else {
                    continue;
                };
                let StaticEffect::Continuous { affected, mods } = &s.effect else {
                    continue;
                };
                if !mods.iter().any(is_cant_have) || !self.ability_functions(o, s.zone, s.is_cda) {
                    continue;
                }
                let mut ctx = Ctx::for_object(self, *id);
                ctx.link = a.link;
                let key = EffKey::Static(*id, a.uid);
                let aff = match st.started.get(&key) {
                    Some(v) => v.clone(),
                    None => {
                        if s.condition
                            .as_ref()
                            .is_some_and(|c| !self.eval_cond(c, &ctx))
                        {
                            continue;
                        }
                        self.static_affected(*id, affected, live, &ctx)
                    }
                };
                work.push((aff, mods.clone(), ctx));
            }
        }
        for (affected, mods, ctx) in work {
            for t in affected {
                if !self.is_live(t) {
                    continue;
                }
                for m in mods.iter().filter(|m| is_cant_have(m)) {
                    self.apply_mod_to(t, m, &ctx);
                }
            }
        }
    }

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
        // CR 613.10: effects on players apply after objects' characteristics are
        // determined, in timestamp order; CR 613.11: rule-modifying effects such as
        // maximum hand size changes likewise apply in timestamp order.
        let n = self.players.len();
        let mut ordered: Vec<(Timestamp, usize, PlayerId, PlayerModification, Ctx)> = Vec::new();
        for (k, (src, ctl, e)) in self.statics.other.clone().into_iter().enumerate() {
            let ts = self.obj(src).timestamp;
            let ctx = Ctx::new(Some(src), ctl);
            match e {
                StaticEffect::PlayerEffect { affected, effect } => {
                    for p in self.player_ids() {
                        if self.player_filter_matches(&affected, p, &ctx) {
                            ordered.push((ts, k, p, effect.clone(), ctx.clone()));
                        }
                    }
                }
                StaticEffect::AdditionalLandPlays(rel, x) => {
                    for p in self.player_ids() {
                        if self.player_rel_matches(rel, p, &ctx) {
                            ordered.push((
                                ts,
                                k,
                                p,
                                PlayerModification::AdditionalLandPlays(x),
                                ctx.clone(),
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        let base = self.statics.other.len();
        for (k, e) in self.player_effects.clone().into_iter().enumerate() {
            let ctx = Ctx::new(e.source, e.controller);
            for p in e.players {
                ordered.push((e.timestamp, base + k, p, e.effect.clone(), ctx.clone()));
            }
        }
        ordered.sort_by_key(|(ts, k, ..)| (*ts, *k));
        let mut mods: Vec<Vec<(PlayerModification, Ctx)>> = vec![vec![]; n];
        for (_, _, p, m, ctx) in ordered {
            mods[p.idx()].push((m, ctx));
        }
        for (i, m) in mods.into_iter().enumerate() {
            let mut max_hand: Option<i32> = Some(7);
            let mut land_plays = 1u32;
            for (x, ctx) in &m {
                match x {
                    PlayerModification::MaxHandSize(v) => {
                        max_hand = v.as_ref().map(|v| self.eval_value(v, ctx) as i32)
                    }
                    PlayerModification::HandSizeDelta(d) => max_hand = max_hand.map(|h| h + d),
                    PlayerModification::AdditionalLandPlays(k) => land_plays += k,
                    _ => {}
                }
            }
            // Vanguard hand modifier (CR 902.3) handled by the variant module via HandSizeDelta.
            let p = &mut self.players[i];
            p.max_hand_size = max_hand;
            p.land_plays = land_plays;
            p.mods = m.into_iter().map(|(x, _)| x).collect();
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
        EffKey::KwCounter(o, i) => (2, (o.0 as u64) << 8 | *i as u64),
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
        Modification::SetName(n) => {
            c.name = n.clone();
            c.all_creature_names = false;
        }
        Modification::AllCreatureNames => c.all_creature_names = true,
        Modification::NameSticker { word, position } => {
            c.name = crate::stickers::add_name_word(&c.name, word, *position as usize).into();
        }
        // Becomes `SetText` for each object as the effect is created.
        Modification::ExchangeText => {}
        Modification::SetText { abilities, text } => {
            c.abilities = abilities.clone();
            c.rules_text = std::sync::Arc::from(text.as_str());
        }
        Modification::FullTextOf(sel) => {
            if let Some(t) = g.eval_sel_objects(sel, ctx).first() {
                crate::text_change::take_full_text(c, &g.obj(*t).base);
            }
        }
        Modification::AddText { abilities, text } => {
            c.abilities.extend(abilities.iter().cloned());
            let own = c.rules_text.trim_end();
            let joined = if own.is_empty() {
                text.to_string()
            } else {
                format!("{own}\n{text}")
            };
            c.rules_text = std::sync::Arc::from(joined.as_str());
        }
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
        Modification::AddChosenType => {
            // CR 607.2d / 607.5a: the type chosen for the effect's source, if any.
            if let Some(ch) = g.source_choices(ctx) {
                if let Some(t) = ch.creature_type.clone().or(ch.basic_land_type.clone()) {
                    if !c.subtypes.contains(&t) {
                        c.subtypes.push(t);
                    }
                }
            }
        }
        Modification::SetChosenBasicLandType => {
            if let Some(t) = g
                .source_choices(ctx)
                .and_then(|ch| ch.basic_land_type.clone())
            {
                // CR 305.7, as for SetBasicLandType.
                c.subtypes
                    .retain(|s| !subtype_lists().land.contains(s.as_str()));
                c.subtypes.push(t);
                c.abilities.clear();
            }
        }
        Modification::SetColors(cs) => c.colors = *cs,
        Modification::SetLinkedChosenColor => {
            if let Some(col) = g.linked_choice(ctx).and_then(|ch| ch.color) {
                let mut cs = ColorSet::NONE;
                cs.insert(col);
                c.colors = cs;
            }
        }
        Modification::SetChosenColor => {
            if let Some(col) = g.source_choices(ctx).and_then(|ch| ch.color) {
                c.colors = ColorSet::single(col);
            }
        }
        Modification::AddColors(cs) => c.colors = c.colors.union(*cs),
        Modification::AddAbility(a) => c.abilities.push(acquired_ability(a, ctx.source, _target)),
        Modification::AddKeyword(k) => {
            // "Protection from the chosen color": the choice is the granting ability's,
            // not the object gaining it's (CR 607.2d); an undefined linked choice grants
            // nothing (CR 607.5a).
            let mut k = k.clone();
            if let Some(f) = &k.filter {
                match resolve_chosen(f, g, ctx) {
                    Some(f) => k.filter = Some(f),
                    None => return,
                }
            }
            if let (Some(f), Some(ch)) = (k.filter.as_ref(), g.source_choices(ctx)) {
                if crate::choices::filter_mentions_choice(f) {
                    k.filter = Some(crate::choices::bind_choices(f, ch));
                }
            }
            // CR 702.16n: "This effect doesn't remove [this Aura]" — remember which
            // object the protection doesn't remove.
            if let (Some(t), Some(src)) = (k.text.as_deref(), ctx.source) {
                if t == crate::choices::DOESNT_REMOVE_SOURCE {
                    k.text = Some(crate::choices::doesnt_remove_marker(src));
                }
            }
            let name = k.kind.name();
            c.abilities
                .push(AbilityDef::new(AbilityKind::Keyword(k), name))
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
        Filter::LinkedChosenCreatureType => {
            Filter::Subtype(g.linked_choice(ctx)?.creature_type.clone()?)
        }
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
            AbilityDef::with_link(a.kind.clone(), a.text.clone(), copied_link(a.link, effect))
        })
        .clone()
}

/// The link id an ability with link `link` has when an object has it through the copy
/// effect `effect` (see [`copied_ability`]).
pub(crate) fn copied_link(link: u16, effect: u32) -> u16 {
    0x4000 | ((link as u32 * 131 + effect * 37) % 0x3fff) as u16
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

/// CR 613.2a: an "as [this] enters" or "as [this] is turned face up" ability that sets
/// power and toughness generates a copiable effect: continuous effects it created (those
/// after index `from` in `Game::effects`) that apply only to the permanent and set its
/// power and toughness become part of its copiable values.
pub fn as_enters_copiable(g: &mut Game, obj: ObjectId, from: usize) {
    for e in g.effects.iter_mut().skip(from) {
        let only_it = matches!(&e.affected, Affected::Objects(v) if v.as_slice() == [obj]);
        if only_it
            && e.layer1.is_none()
            && e.mods.iter().any(|m| matches!(m, Modification::SetPT(..)))
        {
            let mods = std::mem::take(&mut e.mods);
            e.layer1 = Some(Layer1::Copiable(mods));
            e.duration = Duration::Permanent;
        }
    }
    g.dirty = true;
}
