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
    fn after_damage(
        &self,
        g: &mut Game,
        source: ObjectId,
        target: Entity,
        amount: u32,
        combat: bool,
    ) {
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
}

// Every file in this directory is a module (generated by build.rs).
include!(concat!(env!("OUT_DIR"), "/kw_mods.rs"));

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
