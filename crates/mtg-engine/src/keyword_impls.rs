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
    // Instances of each keyword seen so far (an effect can grant hundreds, e.g. a {0}
    // ability activated again and again).
    let mut seen: HashMap<String, usize> = HashMap::new();
    for a in &chars.abilities {
        if let AbilityKind::Keyword(k) = &a.kind {
            let base = format!("{k:?}");
            let nth = seen.entry(base.clone()).or_default();
            let key = if *nth == 0 {
                base
            } else {
                format!("{base}#{nth}")
            };
            *nth += 1;
            let derived = derived_abilities_keyed(k, key);
            for d in &derived {
                crate::structure::note_derived(d, a, &chars.name);
            }
            out.extend(derived.into_iter().map(|d| (a.uid, d)));
        }
    }
    out
}

/// Whether a keyword among `abilities` stands for a static ability that functions in
/// every zone, such as devoid's characteristic-defining ability (CR 604.3, 702.114a).
/// Cached per keyword ability.
pub fn derives_ability_functioning_everywhere(abilities: &[Ability]) -> bool {
    static C: OnceLock<Mutex<HashMap<u64, bool>>> = OnceLock::new();
    abilities.iter().any(|a| {
        let AbilityKind::Keyword(k) = &a.kind else {
            return false;
        };
        let cache = C.get_or_init(Default::default);
        if let Some(b) = cache.lock().unwrap().get(&a.uid) {
            return *b;
        }
        let b = derived_abilities(k).iter().any(|d| {
            matches!(&d.kind, AbilityKind::Static(s)
                if s.is_cda || s.zone == FunctionZone::Anywhere)
        });
        cache.lock().unwrap().insert(a.uid, b);
        b
    })
}

/// If the ability is derived from a keyword, which one. A keyword that labels the ability
/// following it ("Boast — [cost]: [effect]", CR 702.142a) is found by that label.
pub fn ability_from_keyword(a: &AbilityDef) -> Option<KeywordKind> {
    KeywordKind::ALL.iter().copied().find(|k| {
        a.text == k.name()
            || a.text
                .strip_prefix(k.name())
                .is_some_and(|r| r.starts_with(" — "))
    })
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
        let n = affinity_reduction(g, p, card, kw);
        if let Some(m) = cost.mana.as_mut() {
            m.reduce_generic(n);
        }
    }
    crate::kw::cost_reductions(g, p, card, chars, cost, x);
}

/// How much less an affinity ability makes `card` cost to cast for `p` (CR 702.41a:
/// {1} less for each [filter] they control); 0 for other keywords.
pub fn affinity_reduction(g: &Game, p: PlayerId, card: ObjectId, kw: &Keyword) -> u32 {
    if kw.kind != KeywordKind::Affinity {
        return 0;
    }
    let Some(f) = &kw.filter else {
        return 0;
    };
    g.objects_matching(f, &Ctx::new(Some(card), p))
        .into_iter()
        .filter(|o| g.obj(*o).controller == p)
        .count() as u32
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

/// Where a resolving instant/sorcery goes (CR 608.2n), letting its controller choose
/// between a replacement effect tied to how it was cast (e.g. rebound, buyback) and other
/// replacement effects that would apply to it being put into its owner's graveyard (e.g.
/// Rest in Peace), CR 616.1. Returns the destination, and whether the player chose one
/// of those other effects instead (they then apply as it moves to the graveyard, and the
/// keyword's effect doesn't happen).
pub fn choose_resolved_spell_destination(
    g: &mut Game,
    id: ObjectId,
) -> ((Zone, LibraryPosition), bool) {
    let o = g.obj(id);
    let (owner, controller) = (o.owner, o.controller);
    let graveyard = (Zone::Graveyard(owner), LibraryPosition::Top);
    if crate::adventure::resolved_destination(g, id).is_some() {
        return (resolved_spell_destination(g, id), false);
    }
    let dests = crate::kw::resolved_destinations(g, id);
    if dests.is_empty() {
        return (graveyard, false);
    }
    // A destination that isn't a replacement effect the player chooses among (flashback's
    // "exile it instead of putting it anywhere else") applies.
    if let Some((dest, _)) = dests.iter().find(|(_, l)| l.is_none()) {
        return (*dest, false);
    }
    let others = g.applicable_replacements(&crate::replacement::ReplEvent::Move(
        crate::replacement::MoveEv {
            obj: id,
            to: graveyard.0,
            pos: graveyard.1,
            cause: crate::events::MoveCause::Resolve,
            by: Some(controller),
            etb: Default::default(),
            source: None,
        },
    ));
    if others.is_empty() && dests.len() == 1 {
        return (dests[0].0, false);
    }
    // The keywords' replacement effects (e.g. rebound and buyback) and the others: the
    // controller chooses one (CR 616.1).
    let k = dests.len();
    let n = k + others.len();
    let mut options: Vec<String> = dests.iter().filter_map(|(_, l)| l.clone()).collect();
    options.extend(others);
    match g.ask(
        controller,
        crate::decision::Decision::ChooseReplacement { options },
    ) {
        crate::decision::Answer::Index(i) if i < k => (dests[i].0, false),
        crate::decision::Answer::Index(i) if i < n => (graveyard, true),
        _ => (dests[0].0, false),
    }
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

/// After damage is dealt (poisonous-like effects implemented as statics, etc.).
pub fn after_damage(g: &mut Game, source: ObjectId, target: Entity, amount: u32, combat: bool) {
    crate::kw::after_damage(g, source, target, amount, combat);
}

/// CR 702.164c toxic: combat damage dealt to a player by a creature with toxic causes that
/// creature's controller to give the player poison counters equal to its total toxic value
/// (CR 702.164b), in addition to the damage's other results. The counters that creatures
/// one player controls give a player with simultaneous combat damage are put as a single
/// event ("If more than one creature with toxic deals combat damage to a player at the
/// same time, those counters are placed as a single event"); they're a result of damage,
/// not an effect.
pub fn toxic_counters(g: &mut Game, dealt: &[(ObjectId, Entity, u32)], combat: bool) {
    if !combat {
        return;
    }
    // (controller, player, total toxic value, first source)
    let mut groups: Vec<(PlayerId, PlayerId, u32, ObjectId)> = Vec::new();
    for (source, target, _) in dealt {
        let Entity::Player(p) = target else {
            continue;
        };
        let toxic: i32 = g
            .obj(*source)
            .chars
            .keywords()
            .filter(|k| k.kind == KeywordKind::Toxic)
            .map(|k| k.n.unwrap_or(0))
            .sum();
        if toxic <= 0 {
            continue;
        }
        let ctl = g.obj(*source).controller;
        match groups.iter_mut().find(|(c, q, _, _)| *c == ctl && q == p) {
            Some(gr) => gr.2 += toxic as u32,
            None => groups.push((ctl, *p, toxic as u32, *source)),
        }
    }
    for (_, p, n, source) in groups {
        let how = crate::event_causes::CounterPut::damage(g, source);
        g.put_counters(Entity::Player(p), counters::POISON, n, how);
    }
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
