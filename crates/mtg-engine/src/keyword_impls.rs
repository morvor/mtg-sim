//! Keyword ability implementations (CR 702) and the hook points the core engine calls.
//!
//! Keywords are implemented in two ways:
//!
//! 1. **Derived abilities** — [`derived_abilities`] expands a keyword into the triggered,
//!    activated, or static abilities it stands for (e.g. prowess → a triggered ability).
//!    Derived abilities are added to an object's characteristics at the end of layer 6,
//!    so keywords granted by effects work exactly like printed ones. A derived static
//!    ability that generates a continuous effect applies in that effect's own layers as
//!    soon as the object has the keyword (so a printed keyword's "has haste" applies in
//!    layer 6 and a keyword's characteristic-changing effect in layers 2–5), with the
//!    keyword's timestamp and taking part in dependencies (CR 613.1, 613.7a, 613.8).
//! 2. **Rule hooks** — functions below that the core calls at specific points (combat
//!    damage, casting options, spell destinations, special actions, ...).
//!
//! Simple static keywords (flying, reach, deathtouch, ...) are checked directly by the
//! rules code (combat, damage, targeting) and need nothing here.

use crate::ability::*;
use crate::casting::{CastOption, Illegal};
use crate::decision::{Action, SpecialAction};
use crate::eval::Ctx;
use crate::game::Game;
use crate::keywords::{Keyword, KeywordKind};
use crate::mana::ManaCost;
use crate::object::*;
use crate::types::*;
use smol_str::SmolStr;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// ---------------------------------------------------------------------------
// Derived abilities
// ---------------------------------------------------------------------------

fn cache() -> &'static Mutex<HashMap<String, Vec<Ability>>> {
    static C: OnceLock<Mutex<HashMap<String, Vec<Ability>>>> = OnceLock::new();
    C.get_or_init(Default::default)
}

/// The abilities a keyword instance stands for. Cached per distinct keyword instance so
/// ability uids are stable across recomputation.
pub fn derived_abilities(kw: &Keyword) -> Vec<Ability> {
    derived_abilities_keyed(kw, format!("{kw:?}"))
}

/// The abilities of the keyword instance cached under `key`. The `n`th of several
/// identical instances of a keyword on one object (n > 0) is keyed `"{kw:?}#{n}"`: each
/// instance's abilities are distinct abilities that work separately (e.g. CR 702.43b,
/// 702.44d), so they get their own uids.
fn derived_abilities_keyed(kw: &Keyword, key: String) -> Vec<Ability> {
    if let Some(v) = cache().lock().unwrap().get(&key) {
        return v.clone();
    }
    let v = build_derived(kw);
    // Another thread may have built the same instance meanwhile: keep the first one, so
    // every object gets the same abilities (and uids) for it.
    cache().lock().unwrap().entry(key).or_insert(v).clone()
}

fn build_derived(kw: &Keyword) -> Vec<Ability> {
    // CR 702.6 equip: see `kw/equip.rs`. CR 702.21 ward: see `kw/ward.rs`.
    // CR 702.29 cycling and typecycling: see `kw/cycling.rs`. CR 702.108 prowess:
    // `kw/prowess.rs`.
    crate::kw::derived(kw)
}

/// Appends derived abilities for every keyword on the object.
pub fn expand_keywords(chars: &mut Characteristics) {
    let extra = derived_by_keyword(chars);
    chars.abilities.extend(extra.into_iter().map(|(_, a)| a));
}

/// The abilities the keywords among `chars`' abilities stand for, each paired with the
/// uid of the keyword ability it comes from. The layer system applies derived static
/// abilities in their own layers as soon as the keyword exists (CR 613.1, 613.7a) and
/// adds all derived abilities to the characteristics after layer 6 (see
/// `Game::compute_characteristics`).
pub fn derived_by_keyword(chars: &Characteristics) -> Vec<(u64, Ability)> {
    let mut out: Vec<(u64, Ability)> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for a in &chars.abilities {
        if let AbilityKind::Keyword(k) = &a.kind {
            let base = format!("{k:?}");
            let nth = seen.iter().filter(|s| **s == base).count();
            let key = if nth == 0 {
                base.clone()
            } else {
                format!("{base}#{nth}")
            };
            seen.push(base);
            out.extend(
                derived_abilities_keyed(k, key)
                    .into_iter()
                    .map(|d| (a.uid, d)),
            );
        }
    }
    out
}

/// If the ability is derived from a keyword, which one.
pub fn ability_from_keyword(a: &AbilityDef) -> Option<KeywordKind> {
    KeywordKind::ALL
        .iter()
        .copied()
        .find(|k| a.text == k.name())
}

// ---------------------------------------------------------------------------
// Casting hooks
// ---------------------------------------------------------------------------

/// Keyword-granted ways to cast a card (flashback, escape, foretell, dash, evoke, ...).
pub fn keyword_cast_options(g: &Game, p: PlayerId, card: ObjectId) -> Vec<CastOption> {
    // Flashback (CR 702.34): see `kw/flashback.rs`.
    crate::kw::cast_options(g, p, card)
}

/// Optional additional costs announced while casting (CR 601.2b): (name, cost, repeatable).
pub fn optional_additional_costs(g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
    // Kicker (CR 702.33): see `kw/kicker.rs`; buyback (CR 702.27): `kw/buyback.rs`.
    crate::kw::optional_costs(g, spell)
}

/// Lets keywords adjust the spell's targets/effect as cast (entwine, mutate, ...).
pub fn adjust_spell_body(g: &Game, id: ObjectId, body: Body) -> Body {
    crate::kw::adjust_spell_body(g, id, body)
}

/// Cost reductions from keywords (affinity, convoke, delve, improvise, undaunted, ...).
pub fn cost_reductions_from_keywords(
    g: &Game,
    p: PlayerId,
    card: ObjectId,
    chars: &Characteristics,
    cost: &mut Cost,
    x: u32,
) {
    for kw in chars.keywords() {
        if kw.kind == KeywordKind::Affinity {
            // CR 702.41a: costs {1} less for each [filter] you control.
            if let Some(f) = &kw.filter {
                let n = g
                    .objects_matching(f, &Ctx::new(Some(card), p))
                    .into_iter()
                    .filter(|o| g.obj(*o).controller == p)
                    .count();
                if let Some(m) = cost.mana.as_mut() {
                    m.reduce_generic(n as u32);
                }
            }
        }
    }
    crate::kw::cost_reductions(g, p, card, chars, cost, x);
}

/// Where an instant/sorcery goes after resolving (CR 608.2n), considering replacement
/// effects tied to how it was cast.
pub fn resolved_spell_destination(g: &Game, id: ObjectId) -> (Zone, LibraryPosition) {
    let o = g.obj(id);
    // A spell cast as an Adventure or an Omen (CR 715.3d, 720.3d).
    if let Some(d) = crate::adventure::resolved_destination(g, id) {
        return d;
    }
    // Flashback (CR 702.34a), buyback (CR 702.27a), and other keywords: see `kw/`.
    crate::kw::resolved_destination(g, id)
        .unwrap_or((Zone::Graveyard(o.owner), LibraryPosition::Top))
}

/// After a resolved instant/sorcery was put where it goes (`new`), e.g. rebound's delayed
/// triggered ability (CR 702.88a).
pub fn after_spell_resolved(g: &mut Game, id: ObjectId, new: ObjectId) {
    crate::adventure::after_resolved(g, id, new);
    crate::kw::after_spell_resolved(g, id, new);
}

/// Where a countered spell (or one that fails to resolve) goes.
pub fn countered_spell_destination(g: &Game, id: ObjectId) -> (Zone, LibraryPosition) {
    let o = g.obj(id);
    // Flashback (CR 702.34a: exiled instead of anywhere else): see `kw/flashback.rs`.
    crate::kw::countered_destination(g, id)
        .unwrap_or((Zone::Graveyard(o.owner), LibraryPosition::Top))
}

pub fn after_permanent_spell_resolves(g: &mut Game, spell: ObjectId, new: ObjectId) {
    crate::kw::after_permanent_resolves(g, spell, new);
}

pub fn unbestow_on_stack(g: &mut Game, id: ObjectId) {
    crate::kw::unbestow(g, id);
}

pub fn is_mutating(g: &Game, id: ObjectId) -> bool {
    crate::kw::is_mutating(g, id)
}

pub fn resolve_mutate(g: &mut Game, id: ObjectId) {
    crate::kw::resolve_mutate(g, id);
}

// ---------------------------------------------------------------------------
// Special actions (CR 116)
// ---------------------------------------------------------------------------

pub fn special_actions(g: &Game, p: PlayerId) -> Vec<Action> {
    let mut v = crate::special_actions::available(g, p);
    v.extend(crate::kw::special_actions(g, p));
    // Rolling the planar die (CR 901.9).
    v.extend(crate::planechase::special_actions(g, p));
    v
}

pub fn perform_special_action(g: &mut Game, p: PlayerId, sa: SpecialAction) -> Result<(), Illegal> {
    if let Some(r) = crate::special_actions::perform(g, p, &sa) {
        return r;
    }
    if let Some(r) = crate::planechase::perform_special_action(g, p, &sa) {
        return r;
    }
    crate::kw::perform_special_action(g, p, sa)
}

// ---------------------------------------------------------------------------
// Combat hooks
// ---------------------------------------------------------------------------

/// Additional per-block legality from keywords; returns true if the block is allowed.
pub fn extra_block_restrictions(g: &Game, blocker: ObjectId, attacker: ObjectId) -> bool {
    crate::kw::block_allowed(g, blocker, attacker)
}

pub fn attack_declaration_extra_checks(g: &Game, decl: &[(ObjectId, Entity)]) -> bool {
    crate::kw::attack_declaration_ok(g, decl)
}

pub fn block_declaration_extra_checks(
    g: &Game,
    options: &[(ObjectId, Vec<ObjectId>)],
    decl: &[(ObjectId, ObjectId)],
) -> bool {
    crate::kw::block_declaration_ok(g, options, decl)
}

pub fn pay_attack_costs(g: &mut Game, ap: PlayerId, declared: &[(ObjectId, Entity)]) {
    crate::kw::pay_attack_costs(g, ap, declared);
}

pub fn pay_block_costs(g: &mut Game, blocks: &[(ObjectId, ObjectId)]) {
    crate::kw::pay_block_costs(g, blocks);
}

pub fn before_combat_damage(g: &mut Game, assignments: &mut Vec<(ObjectId, Entity, u32)>) {
    crate::kw::before_combat_damage(g, assignments);
}

/// How much combat damage a creature assigns (CR 510.1a): normally its power.
pub fn combat_damage_amount(g: &Game, id: ObjectId) -> u32 {
    crate::kw::combat_damage_amount(g, id).unwrap_or_else(|| g.obj(id).power().max(0) as u32)
}

/// "can deal combat damage as though it weren't blocked" (CR 510.1c exceptions).
pub fn assigns_as_though_unblocked(g: &mut Game, id: ObjectId) -> bool {
    crate::kw::assigns_as_though_unblocked(g, id)
}

// ---------------------------------------------------------------------------
// Other hooks
// ---------------------------------------------------------------------------

/// After damage is dealt (toxic, poisonous-like effects implemented as statics, etc.).
pub fn after_damage(g: &mut Game, source: ObjectId, target: Entity, amount: u32, combat: bool) {
    // CR 702.164c toxic: combat damage to a player also gives poison counters.
    if combat {
        if let Entity::Player(p) = target {
            let toxic: i32 = g
                .obj(source)
                .chars
                .keywords()
                .filter(|k| k.kind == KeywordKind::Toxic)
                .map(|k| k.n.unwrap_or(0))
                .sum();
            if toxic > 0 {
                g.add_counters(
                    Entity::Player(p),
                    counters::POISON,
                    toxic as u32,
                    Some(source),
                );
            }
        }
    }
    crate::kw::after_damage(g, source, target, amount, combat);
}

/// CR 502.1 / 702.26: phasing during the untap step.
pub fn phasing_untap_step(g: &mut Game, active: PlayerId) {
    crate::kw::phasing::untap_step(g, active);
}

/// Phases permanents out, along with everything attached to them (CR 702.26g).
pub fn phase_out(g: &mut Game, objs: Vec<ObjectId>) {
    crate::kw::phasing::phase_out(g, objs);
}

pub fn phase_in(g: &mut Game, id: ObjectId) {
    crate::kw::phasing::phase_in(g, id);
}

/// CR 702.145 daybound/nightbound transform when day/night changes.
pub fn day_night_changed(g: &mut Game) {
    crate::kw::day_night_changed(g);
}

/// Parses "{2}{R}" into a mana cost, for keyword costs.
pub fn mana(s: &str) -> Cost {
    Cost::mana(ManaCost::parse(s).unwrap_or_default())
}

/// Equip restrictions such as "Equip legendary creature" (CR 702.6) are enforced on the
/// equip ability's target; attachment legality otherwise only requires a creature.
pub fn equip_restriction_ok(_g: &Game, _equipment: ObjectId, _creature: ObjectId) -> bool {
    true
}
