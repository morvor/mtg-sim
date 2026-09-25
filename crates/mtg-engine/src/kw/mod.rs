//! Registry of keyword rule implementations (CR 702).
//!
//! Each keyword (or small family of related keywords) lives in its own file in this
//! directory and implements [`KeywordRules`]. Register it in [`registry`] with one line.
//! Every hook has a no-op default, so an implementation only overrides what it needs.
//!
//! Hooks come in two flavors:
//! * **per-instance** hooks receive the [`Keyword`] instance (e.g. `derived`,
//!   `cast_options`, `optional_costs`) and are only called for objects that have the
//!   keyword;
//! * **global** hooks (combat checks, special actions, damage) are called on every
//!   registered implementation, which inspects the game itself.

use crate::ability::*;
use crate::casting::{CastOption, Illegal};
use crate::decision::{Action, SpecialAction};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;
use std::sync::OnceLock;

/// Rules behavior for one keyword ability. All methods have no-op defaults.
#[allow(unused_variables)]
pub trait KeywordRules: Sync + Send {
    /// The keyword(s) this implementation handles.
    fn kinds(&self) -> &'static [KeywordKind];

    /// Abilities the keyword stands for (triggered/activated/static). Called once per
    /// distinct keyword instance and cached.
    fn derived(&self, kw: &Keyword) -> Option<Vec<Ability>> {
        None
    }
    /// Whether a variable X the granting effect defines ("has ward {X}, where X is ...")
    /// is determined as the keyword's ability resolves (kept in [`Keyword::x`]) rather than
    /// whenever characteristics are computed (CR 702.21b).
    fn x_determined_on_resolution(&self) -> bool {
        false
    }
    /// Additional ways to cast `card` because it has `kw`.
    fn cast_options(&self, g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> Vec<CastOption> {
        vec![]
    }
    /// Ways to cast `card` that don't depend on a keyword it currently has, e.g. a
    /// foretold card face down in exile (CR 702.143a) or a plotted card (CR 702.170d).
    /// Called for every registered implementation.
    fn global_cast_options(&self, g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
        vec![]
    }
    /// Optional additional costs announced while casting (name, cost, repeatable).
    fn optional_costs(
        &self,
        g: &Game,
        spell: ObjectId,
        kw: &Keyword,
    ) -> Vec<(SmolStr, Cost, bool)> {
        vec![]
    }
    /// Adjust the targets/effect of a spell being cast (e.g. overload).
    fn adjust_spell_body(&self, g: &Game, spell: ObjectId, kw: &Keyword, body: Body) -> Body {
        body
    }
    /// Reduce/modify the total cost of a spell being cast.
    fn cost_reduction(
        &self,
        g: &Game,
        p: PlayerId,
        card: ObjectId,
        kw: &Keyword,
        cost: &mut Cost,
        x: u32,
    ) {
    }
    /// Where a resolved instant/sorcery goes, if the keyword changes it.
    fn resolved_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        None
    }
    /// Where a countered spell goes, if the keyword changes it.
    fn countered_destination(
        &self,
        g: &Game,
        spell: ObjectId,
        kw: &Keyword,
    ) -> Option<(Zone, LibraryPosition)> {
        None
    }
    /// After a permanent spell with this keyword resolves (`new` is the permanent).
    fn after_permanent_resolves(&self, g: &mut Game, spell: ObjectId, new: ObjectId, kw: &Keyword) {
    }

    // --- global hooks ---
    fn special_actions(&self, g: &Game, p: PlayerId) -> Vec<Action> {
        vec![]
    }
    /// Return Some(result) if this implementation handles the special action.
    fn perform_special_action(
        &self,
        g: &mut Game,
        p: PlayerId,
        sa: &SpecialAction,
    ) -> Option<Result<(), Illegal>> {
        None
    }
    fn block_allowed(&self, g: &Game, blocker: ObjectId, attacker: ObjectId) -> bool {
        true
    }
    /// Whether `p` may activate the activated ability `a` of `src` as far as this
    /// implementation is concerned (e.g. "Players can't cycle cards", CR 702.29f).
    fn activation_allowed(&self, g: &Game, p: PlayerId, src: ObjectId, a: &Ability) -> bool {
        true
    }
    fn attack_declaration_ok(&self, g: &Game, decl: &[(ObjectId, Entity)]) -> bool {
        true
    }
    fn block_declaration_ok(
        &self,
        g: &Game,
        options: &[(ObjectId, Vec<ObjectId>)],
        decl: &[(ObjectId, ObjectId)],
    ) -> bool {
        true
    }
    fn pay_attack_costs(&self, g: &mut Game, ap: PlayerId, declared: &[(ObjectId, Entity)]) {}
    fn pay_block_costs(&self, g: &mut Game, blocks: &[(ObjectId, ObjectId)]) {}
    fn before_combat_damage(&self, g: &mut Game, assignments: &mut Vec<(ObjectId, Entity, u32)>) {}
    fn combat_damage_amount(&self, g: &Game, creature: ObjectId) -> Option<u32> {
        None
    }
    fn assigns_as_though_unblocked(&self, g: &mut Game, creature: ObjectId) -> bool {
        false
    }
    /// The player who assigns this attacking or blocking creature's combat damage instead
    /// of its controller, dividing it freely among the creatures it's blocked by or
    /// blocking (banding, CR 702.22j–k).
    fn combat_damage_assigner(&self, g: &Game, creature: ObjectId) -> Option<PlayerId> {
        None
    }
    /// Other attacking creatures that become blocked by the same blocking creature when
    /// `attacker` becomes blocked by it (or become blocked when an effect blocks it), e.g.
    /// the rest of its band (CR 702.22h–i).
    fn also_blocked(&self, g: &Game, attacker: ObjectId) -> Vec<ObjectId> {
        vec![]
    }
    fn after_damage(
        &self,
        g: &mut Game,
        source: ObjectId,
        target: Entity,
        amount: u32,
        combat: bool,
    ) {
    }
    /// Prevention effects (CR 615) that keyword abilities generate for a proposed damage
    /// event, each preventing all of that damage (e.g. protection, CR 702.16e).
    fn damage_prevention(&self, g: &Game, source: ObjectId, target: Entity) -> Vec<KeywordShield> {
        vec![]
    }
    fn day_night_changed(&self, g: &mut Game) {}
    /// A player drew `card` (the `nth` card they drew this turn), as it's drawn: e.g.
    /// "you may reveal this card as you draw it" (CR 121.9, 702.94a).
    fn after_draw(&self, g: &mut Game, p: PlayerId, card: ObjectId, nth: u32) {}
    fn is_mutating(&self, g: &Game, spell: ObjectId) -> bool {
        false
    }
    fn resolve_mutate(&self, g: &mut Game, spell: ObjectId) -> bool {
        false
    }
    fn unbestow(&self, g: &mut Game, spell: ObjectId) -> bool {
        false
    }
    /// Evaluates a named [`Condition::Custom`] this implementation defines, if it's one.
    fn custom_condition(&self, g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
        None
    }
    /// Performs a named [`Effect::Custom`] this implementation defines; returns true if it
    /// was one.
    fn custom_effect(&self, g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
        false
    }
    /// Matches a named [`TriggerCond::Custom`] this implementation defines against an
    /// event, for the triggered ability of `src` controlled by `ctl`.
    fn custom_trigger(
        &self,
        g: &Game,
        name: &str,
        src: ObjectId,
        ctl: PlayerId,
        ev: &Event,
    ) -> Option<Vec<EventInfo>> {
        None
    }
    /// A named value (`Value::Custom(name)`) computed by this implementation, e.g. the
    /// number of spells cast before a storm spell (CR 702.40a).
    fn custom_value(&self, g: &Game, name: &str, ctx: &crate::eval::Ctx) -> Option<i64> {
        None
    }
    /// A named object filter (`Filter::Custom(name)`) evaluated by this implementation,
    /// e.g. "creature that convoked it" (CR 702.51c).
    fn custom_filter(
        &self,
        g: &Game,
        name: &str,
        id: ObjectId,
        ctx: &crate::eval::Ctx,
    ) -> Option<bool> {
        None
    }
    /// Once the total cost of `spell` is locked in (CR 601.2f), ways this keyword lets its
    /// controller pay part of it other than with mana, performed as the total cost is paid
    /// (CR 601.2h): e.g. tapping creatures for convoke (CR 702.51a–b), or sacrificing the
    /// permanent offered for offering, which reduces the mana to pay by its mana cost
    /// (CR 702.48a–c). Takes what was paid out of `cost`.
    fn pay_mana_otherwise(
        &self,
        g: &mut Game,
        p: PlayerId,
        spell: ObjectId,
        kw: &Keyword,
        cost: &mut Cost,
    ) -> Result<(), Illegal> {
        Ok(())
    }
    /// For the check whether `card` could be cast with `method`: takes out of `cost` what
    /// this keyword could pay other than with mana (see
    /// [`KeywordRules::pay_mana_otherwise`]).
    fn payable_otherwise(
        &self,
        g: &Game,
        p: PlayerId,
        card: ObjectId,
        kw: &Keyword,
        method: &CastMethod,
        cost: &mut Cost,
    ) {
    }
}

// Every file in this directory is a module (generated by build.rs).
include!(concat!(env!("OUT_DIR"), "/kw_mods.rs"));

/// A prevention effect generated by a keyword ability of a permanent or player (see
/// [`KeywordRules::damage_prevention`]).
#[derive(Clone, Debug)]
pub struct KeywordShield {
    /// The permanent or player whose ability generates the effect.
    pub holder: Entity,
    /// Distinguishes the holder's abilities: each applies only once to an event (CR 614.5).
    pub id: u64,
    /// The player who controls the effect.
    pub controller: PlayerId,
    pub text: String,
}

/// Registration of a keyword implementation. In a file in `src/kw/`:
///
/// ```ignore
/// pub struct Prowess;
/// impl KeywordRules for Prowess { fn kinds(&self) -> &'static [KeywordKind] { &[KeywordKind::Prowess] } ... }
/// inventory::submit! { KeywordRegistration(&Prowess) }
/// ```
pub struct KeywordRegistration(pub &'static dyn KeywordRules);
inventory::collect!(KeywordRegistration);

/// All registered keyword implementations.
pub fn registry() -> &'static [&'static dyn KeywordRules] {
    static R: OnceLock<Vec<&'static dyn KeywordRules>> = OnceLock::new();
    R.get_or_init(|| {
        inventory::iter::<KeywordRegistration>
            .into_iter()
            .map(|r| r.0)
            .collect()
    })
}

fn impls_for(kind: KeywordKind) -> impl Iterator<Item = &'static &'static dyn KeywordRules> {
    registry().iter().filter(move |r| r.kinds().contains(&kind))
}

// ---------------------------------------------------------------------------
// Dispatchers used by keyword_impls.rs
// ---------------------------------------------------------------------------

pub fn x_determined_on_resolution(kind: KeywordKind) -> bool {
    impls_for(kind).any(|r| r.x_determined_on_resolution())
}

pub fn derived(kw: &Keyword) -> Vec<Ability> {
    for r in impls_for(kw.kind) {
        if let Some(v) = r.derived(kw) {
            return v;
        }
    }
    vec![]
}

pub fn cast_options(g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
    let mut out: Vec<CastOption> = registry()
        .iter()
        .flat_map(|r| r.global_cast_options(g, p, card))
        .collect();
    let kws: Vec<Keyword> = g.obj(card).chars.keywords().cloned().collect();
    for kw in &kws {
        for r in impls_for(kw.kind) {
            out.extend(r.cast_options(g, p, card, kw));
        }
    }
    out
}

pub fn optional_costs(g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
    let mut out = Vec::new();
    let kws: Vec<Keyword> = g.obj(spell).chars.keywords().cloned().collect();
    for kw in &kws {
        for r in impls_for(kw.kind) {
            out.extend(r.optional_costs(g, spell, kw));
        }
    }
    out
}

pub fn adjust_spell_body(g: &Game, spell: ObjectId, mut body: Body) -> Body {
    let kws: Vec<Keyword> = g.obj(spell).chars.keywords().cloned().collect();
    for kw in &kws {
        for r in impls_for(kw.kind) {
            body = r.adjust_spell_body(g, spell, kw, body);
        }
    }
    body
}

pub fn cost_reductions(
    g: &Game,
    p: PlayerId,
    card: ObjectId,
    chars: &Characteristics,
    cost: &mut Cost,
    x: u32,
) {
    let kws: Vec<Keyword> = chars.keywords().cloned().collect();
    for kw in &kws {
        for r in impls_for(kw.kind) {
            r.cost_reduction(g, p, card, kw, cost, x);
        }
    }
}

pub fn resolved_destination(g: &Game, spell: ObjectId) -> Option<(Zone, LibraryPosition)> {
    let kws: Vec<Keyword> = g.obj(spell).chars.keywords().cloned().collect();
    for kw in &kws {
        for r in impls_for(kw.kind) {
            if let Some(d) = r.resolved_destination(g, spell, kw) {
                return Some(d);
            }
        }
    }
    None
}

pub fn countered_destination(g: &Game, spell: ObjectId) -> Option<(Zone, LibraryPosition)> {
    let kws: Vec<Keyword> = g.obj(spell).chars.keywords().cloned().collect();
    for kw in &kws {
        for r in impls_for(kw.kind) {
            if let Some(d) = r.countered_destination(g, spell, kw) {
                return Some(d);
            }
        }
    }
    None
}

pub fn after_permanent_resolves(g: &mut Game, spell: ObjectId, new: ObjectId) {
    let kws: Vec<Keyword> = g.obj(new).chars.keywords().cloned().collect();
    for kw in &kws {
        for r in impls_for(kw.kind) {
            r.after_permanent_resolves(g, spell, new, kw);
        }
    }
}

pub fn special_actions(g: &Game, p: PlayerId) -> Vec<Action> {
    registry()
        .iter()
        .flat_map(|r| r.special_actions(g, p))
        .collect()
}

pub fn perform_special_action(g: &mut Game, p: PlayerId, sa: SpecialAction) -> Result<(), Illegal> {
    for r in registry() {
        if let Some(res) = r.perform_special_action(g, p, &sa) {
            return res;
        }
    }
    Err(Illegal(format!("unsupported special action {sa:?}")))
}

pub fn block_allowed(g: &Game, blocker: ObjectId, attacker: ObjectId) -> bool {
    registry()
        .iter()
        .all(|r| r.block_allowed(g, blocker, attacker))
}

pub fn activation_allowed(g: &Game, p: PlayerId, src: ObjectId, a: &Ability) -> bool {
    registry()
        .iter()
        .all(|r| r.activation_allowed(g, p, src, a))
}

pub fn attack_declaration_ok(g: &Game, decl: &[(ObjectId, Entity)]) -> bool {
    registry().iter().all(|r| r.attack_declaration_ok(g, decl))
}

pub fn block_declaration_ok(
    g: &Game,
    options: &[(ObjectId, Vec<ObjectId>)],
    decl: &[(ObjectId, ObjectId)],
) -> bool {
    registry()
        .iter()
        .all(|r| r.block_declaration_ok(g, options, decl))
}

pub fn pay_attack_costs(g: &mut Game, ap: PlayerId, declared: &[(ObjectId, Entity)]) {
    for r in registry() {
        r.pay_attack_costs(g, ap, declared);
    }
}

pub fn pay_block_costs(g: &mut Game, blocks: &[(ObjectId, ObjectId)]) {
    for r in registry() {
        r.pay_block_costs(g, blocks);
    }
}

pub fn before_combat_damage(g: &mut Game, assignments: &mut Vec<(ObjectId, Entity, u32)>) {
    for r in registry() {
        r.before_combat_damage(g, assignments);
    }
}

pub fn combat_damage_amount(g: &Game, id: ObjectId) -> Option<u32> {
    registry()
        .iter()
        .find_map(|r| r.combat_damage_amount(g, id))
}

pub fn also_blocked(g: &Game, attacker: ObjectId) -> Vec<ObjectId> {
    let mut out: Vec<ObjectId> = Vec::new();
    for r in registry() {
        for x in r.also_blocked(g, attacker) {
            if x != attacker && !out.contains(&x) {
                out.push(x);
            }
        }
    }
    out
}

pub fn combat_damage_assigner(g: &Game, id: ObjectId) -> Option<PlayerId> {
    registry()
        .iter()
        .find_map(|r| r.combat_damage_assigner(g, id))
}

pub fn assigns_as_though_unblocked(g: &mut Game, id: ObjectId) -> bool {
    registry()
        .iter()
        .any(|r| r.assigns_as_though_unblocked(g, id))
}

pub fn after_damage(g: &mut Game, source: ObjectId, target: Entity, amount: u32, combat: bool) {
    for r in registry() {
        r.after_damage(g, source, target, amount, combat);
    }
}

pub fn damage_prevention(g: &Game, source: ObjectId, target: Entity) -> Vec<KeywordShield> {
    registry()
        .iter()
        .flat_map(|r| r.damage_prevention(g, source, target))
        .collect()
}

pub fn day_night_changed(g: &mut Game) {
    for r in registry() {
        r.day_night_changed(g);
    }
}

pub fn after_draw(g: &mut Game, p: PlayerId, card: ObjectId, nth: u32) {
    for r in registry() {
        if !g.is_live(card) {
            return;
        }
        r.after_draw(g, p, card, nth);
    }
}

pub fn is_mutating(g: &Game, spell: ObjectId) -> bool {
    registry().iter().any(|r| r.is_mutating(g, spell))
}

pub fn resolve_mutate(g: &mut Game, spell: ObjectId) {
    for r in registry() {
        if r.resolve_mutate(g, spell) {
            return;
        }
    }
}

pub fn unbestow(g: &mut Game, spell: ObjectId) {
    for r in registry() {
        if r.unbestow(g, spell) {
            return;
        }
    }
}

pub fn custom_condition(g: &Game, name: &str, ctx: &Ctx) -> Option<bool> {
    registry()
        .iter()
        .find_map(|r| r.custom_condition(g, name, ctx))
}

pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut Ctx) -> bool {
    registry().iter().any(|r| r.custom_effect(g, name, ctx))
}

pub fn custom_trigger(
    g: &Game,
    name: &str,
    src: ObjectId,
    ctl: PlayerId,
    ev: &Event,
) -> Option<Vec<EventInfo>> {
    registry()
        .iter()
        .find_map(|r| r.custom_trigger(g, name, src, ctl, ev))
}

/// A keyword ability's cost that `p` pays, after the effects that modify that keyword's
/// costs ("Buyback costs cost {2} less", "All morph costs cost {2} more", CR 601.2f):
/// generic mana only, never below zero.
pub fn modified_keyword_cost(g: &Game, p: PlayerId, kind: KeywordKind, cost: &Cost) -> Cost {
    let mut cost = cost.clone();
    for (s, ctl, cm) in &g.statics.cost_modifiers {
        if !matches!(cm.applies_to, CostTarget::Keyword(k) if k == kind) {
            continue;
        }
        let ctx = Ctx::new(Some(*s), *ctl);
        if !g.player_rel_matches(cm.who, p, &ctx) {
            continue;
        }
        match &cm.change {
            CostChange::ReduceGeneric(v) => {
                let n = g.eval_value(v, &ctx).max(0) as u32;
                if let Some(m) = cost.mana.as_mut() {
                    m.reduce_generic(n);
                }
            }
            CostChange::IncreaseGeneric(v) => {
                let n = g.eval_value(v, &ctx).max(0) as u32;
                cost.mana
                    .get_or_insert_with(crate::mana::ManaCost::default)
                    .add(&crate::mana::ManaCost::generic(n));
            }
            _ => {}
        }
    }
    cost
}

pub fn custom_value(g: &Game, name: &str, ctx: &crate::eval::Ctx) -> Option<i64> {
    registry().iter().find_map(|r| r.custom_value(g, name, ctx))
}

pub fn custom_filter(g: &Game, name: &str, id: ObjectId, ctx: &crate::eval::Ctx) -> Option<bool> {
    registry()
        .iter()
        .find_map(|r| r.custom_filter(g, name, id, ctx))
}

/// Keyword instances of `chars` with distinct kinds: several instances of a payment
/// keyword are redundant (e.g. CR 702.51d).
fn distinct_kinds(chars: &Characteristics) -> Vec<Keyword> {
    let mut out: Vec<Keyword> = Vec::new();
    for kw in chars.keywords() {
        if !out.iter().any(|k| k.kind == kw.kind) {
            out.push(kw.clone());
        }
    }
    out
}

/// CR 601.2h: lets keywords of the spell pay part of its total cost other than with mana.
pub fn pay_mana_otherwise(
    g: &mut Game,
    p: PlayerId,
    spell: ObjectId,
    cost: &mut Cost,
) -> Result<(), Illegal> {
    let kws = distinct_kinds(&g.obj(spell).chars);
    for kw in &kws {
        for r in impls_for(kw.kind) {
            r.pay_mana_otherwise(g, p, spell, kw, cost)?;
        }
    }
    Ok(())
}

/// A card seen as the spell its caster would put on the stack.
struct AsSpell<'c> {
    id: ObjectId,
    chars: &'c Characteristics,
    caster: PlayerId,
}

impl crate::eval::View for AsSpell<'_> {
    fn chars<'a>(&'a self, g: &'a Game, id: ObjectId) -> &'a Characteristics {
        if id == self.id {
            self.chars
        } else {
            &g.obj(id).chars
        }
    }
    fn controller(&self, g: &Game, id: ObjectId) -> PlayerId {
        if id == self.id {
            self.caster
        } else {
            g.obj(id).controller
        }
    }
    fn controller_override(&self, id: ObjectId) -> Option<PlayerId> {
        (id == self.id).then_some(self.caster)
    }
}

/// The characteristics `card` would have as a spell `p` casts, including the keywords
/// that static abilities give such spells ("Artifact spells you cast have convoke"), for
/// checking whether it could be cast (CR 601.3e). As it's cast, the spell on the stack
/// gets them from the layer system.
pub fn with_granted_spell_keywords(
    g: &Game,
    p: PlayerId,
    card: ObjectId,
    chars: &Characteristics,
) -> Characteristics {
    let mut out = chars.clone();
    let sources: Vec<ObjectId> = g
        .permanents()
        .map(|o| o.id)
        .chain(g.command.iter().copied())
        .collect();
    for src in sources {
        let o = g.obj(src);
        for a in &o.chars.abilities {
            let AbilityKind::Static(s) = &a.kind else {
                continue;
            };
            let StaticEffect::Continuous { affected, mods } = &s.effect else {
                continue;
            };
            if !g.ability_functions(o, s.zone, s.is_cda) {
                continue;
            }
            let ctx = crate::eval::Ctx::new(Some(src), o.controller);
            if s.condition.as_ref().is_some_and(|c| !g.eval_cond(c, &ctx)) {
                continue;
            }
            let granted: Vec<&Keyword> = mods
                .iter()
                .filter_map(|m| match m {
                    Modification::AddKeyword(k) => Some(k),
                    _ => None,
                })
                .collect();
            // Judged as the spell it would be: with these characteristics, controlled by
            // its caster.
            let view = AsSpell {
                id: card,
                chars,
                caster: p,
            };
            if granted.is_empty()
                || !g.matches_view(
                    &view,
                    card,
                    &crate::casting::as_spell_filter(affected),
                    &ctx,
                )
            {
                continue;
            }
            for k in granted {
                out.abilities.push(AbilityDef::new(
                    AbilityKind::Keyword(k.clone()),
                    k.kind.name(),
                ));
            }
        }
    }
    // "The next [quality] spell you cast this turn has [keyword]" (CR 611.2f).
    let view = AsSpell {
        id: card,
        chars,
        caster: p,
    };
    for e in &g.next_spell_effects {
        let expired = matches!(e.expires, Duration::EndOfTurn | Duration::ThisTurn)
            && e.created_turn != g.turn.number;
        let ctx = crate::eval::Ctx::new(e.source, e.player);
        if e.player != p
            || expired
            || !g.matches_view(
                &view,
                card,
                &crate::casting::as_spell_filter(&e.filter),
                &ctx,
            )
        {
            continue;
        }
        for m in &e.mods {
            if let Modification::AddKeyword(k) = m {
                out.abilities.push(AbilityDef::new(
                    AbilityKind::Keyword(k.clone()),
                    k.kind.name(),
                ));
            }
        }
    }
    out
}

/// What of `cost` the keywords of `chars` could pay other than with mana.
pub fn payable_otherwise(
    g: &Game,
    p: PlayerId,
    card: ObjectId,
    chars: &Characteristics,
    method: &CastMethod,
    cost: &mut Cost,
) {
    for kw in &distinct_kinds(chars) {
        for r in impls_for(kw.kind) {
            r.payable_otherwise(g, p, card, kw, method, cost);
        }
    }
}
