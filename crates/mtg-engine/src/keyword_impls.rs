//! Keyword ability implementations (CR 702) and the hook points the core engine calls.
//!
//! Keywords are implemented in two ways:
//!
//! 1. **Derived abilities** — [`derived_abilities`] expands a keyword into the triggered,
//!    activated, or static abilities it stands for (e.g. prowess → a triggered ability).
//!    Derived abilities are added to an object's characteristics at the end of layer 6,
//!    so keywords granted by effects work exactly like printed ones.
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
    let key = format!("{kw:?}");
    if let Some(v) = cache().lock().unwrap().get(&key) {
        return v.clone();
    }
    let v = build_derived(kw);
    cache().lock().unwrap().insert(key, v.clone());
    v
}

fn this_creature() -> Filter {
    Filter::Source
}

fn build_derived(kw: &Keyword) -> Vec<Ability> {
    use KeywordKind as K;
    let text = kw.kind.name();
    match kw.kind {
        // CR 702.108a: "Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn."
        K::Prowess => vec![AbilityDef::new(
            AbilityKind::Triggered(TriggeredAbility::new(
                TriggerCond::CastSpell {
                    who: PlayerRel::You,
                    filter: Filter::not(Filter::creature()),
                },
                Body::effect(Effect::Modify {
                    what: Sel::This,
                    mods: vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
                    duration: Duration::EndOfTurn,
                }),
            )),
            text,
        )],
        // CR 702.21a: "Whenever this permanent becomes the target of a spell or ability an
        // opponent controls, counter it unless that player pays [cost]."
        K::Ward => {
            let cost = kw.cost.clone().unwrap_or_default();
            vec![AbilityDef::new(
                AbilityKind::Triggered(TriggeredAbility::new(
                    TriggerCond::BecomesTarget {
                        filter: this_creature(),
                        by: PlayerRel::Opponent,
                    },
                    Body::effect(Effect::PayOptional {
                        who: PlayerRef::TriggerPlayer,
                        cost,
                        then: Box::new(Effect::Noop),
                        otherwise: Box::new(Effect::CounterSpell {
                            what: Sel::TriggerSpell,
                        }),
                    }),
                )),
                format!("Ward"),
            )]
        }
        // CR 702.6a: "[Cost]: Attach to target creature you control. Equip only as a sorcery."
        K::Equip => {
            let cost = kw.cost.clone().unwrap_or_default();
            let filter = kw.filter.clone().unwrap_or(Filter::creature());
            let mut act = ActivatedAbility::new(
                cost,
                Body::simple(
                    vec![TargetSpec::object(
                        Filter::and(vec![
                            filter,
                            Filter::ControlledBy(PlayerRel::You),
                            Filter::Other,
                        ]),
                        "target creature you control",
                    )],
                    Effect::Attach {
                        what: Sel::This,
                        to: Sel::Target(0),
                    },
                ),
            );
            act.timing = ActivationTiming::Sorcery;
            vec![AbilityDef::new(AbilityKind::Activated(act), "Equip")]
        }
        // CR 702.29a: "[Cost], Discard this card: Draw a card."
        K::Cycling => {
            let mut cost = kw.cost.clone().unwrap_or_default();
            cost.parts.push(CostPart::DiscardSelf);
            let mut act = ActivatedAbility::new(
                cost,
                Body::effect(Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::c(1),
                }),
            );
            act.zone = FunctionZone::Hand;
            vec![AbilityDef::new(AbilityKind::Activated(act), "Cycling")]
        }
        _ => crate::kw::derived(kw),
    }
}

/// Appends derived abilities for every keyword on the object (called after layer 6).
pub fn expand_keywords(chars: &mut Characteristics) {
    let mut extra: Vec<Ability> = Vec::new();
    for a in &chars.abilities {
        if let AbilityKind::Keyword(k) = &a.kind {
            extra.extend(derived_abilities(k));
        }
    }
    chars.abilities.extend(extra);
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
    let o = g.obj(card);
    let mut out = Vec::new();
    for kw in o.chars.keywords() {
        match kw.kind {
            // CR 702.34a: cast from graveyard by paying the flashback cost; exiled after.
            KeywordKind::Flashback if o.zone == crate::object::Zone::Graveyard(p) => {
                let mut opt = CastOption::normal(FaceState::Front);
                opt.method = CastMethod::Keyword(KeywordKind::Flashback);
                opt.alt_cost = kw.cost.clone();
                opt.tag = Some("flashback");
                out.push(opt);
            }
            _ => {}
        }
    }
    out.extend(crate::kw::cast_options(g, p, card));
    out
}

/// Optional additional costs announced while casting (CR 601.2b): (name, cost, repeatable).
pub fn optional_additional_costs(g: &Game, spell: ObjectId) -> Vec<(SmolStr, Cost, bool)> {
    let o = g.obj(spell);
    let mut out = Vec::new();
    for kw in o.chars.keywords() {
        match kw.kind {
            // CR 702.33a kicker; multikicker is the same keyword with "multikicker" text.
            KeywordKind::Kicker => {
                let repeatable = kw
                    .text
                    .as_deref()
                    .is_some_and(|t| t.to_lowercase().starts_with("multikicker"));
                if let Some(c) = &kw.cost {
                    out.push((
                        if repeatable {
                            "multikicker".into()
                        } else {
                            "kicker".into()
                        },
                        c.clone(),
                        repeatable,
                    ));
                }
                for c in &kw.costs {
                    out.push(("kicker".into(), c.clone(), false));
                }
            }
            // CR 702.27a buyback
            KeywordKind::Buyback => {
                if let Some(c) = &kw.cost {
                    out.push(("buyback".into(), c.clone(), false));
                }
            }
            _ => {}
        }
    }
    out.extend(crate::kw::optional_costs(g, spell));
    out
}

/// Lets keywords adjust the spell's targets/effect as cast (overload, bestow, ...).
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
    let si = o.stack.as_deref();
    let paid = |name: &str| si.is_some_and(|s| s.cast.paid.iter().any(|p| p == name));
    if matches!(
        si.map(|s| &s.cast.method),
        Some(CastMethod::Keyword(KeywordKind::Flashback))
    ) {
        return (Zone::Exile, LibraryPosition::Top); // CR 702.34a
    }
    if paid("buyback") {
        return (Zone::Hand(o.owner), LibraryPosition::Top); // CR 702.27a
    }
    crate::kw::resolved_destination(g, id)
        .unwrap_or((Zone::Graveyard(o.owner), LibraryPosition::Top))
}

/// Where a countered spell (or one that fails to resolve) goes.
pub fn countered_spell_destination(g: &Game, id: ObjectId) -> (Zone, LibraryPosition) {
    let o = g.obj(id);
    let si = o.stack.as_deref();
    if matches!(
        si.map(|s| &s.cast.method),
        Some(CastMethod::Keyword(KeywordKind::Flashback))
    ) {
        return (Zone::Exile, LibraryPosition::Top); // CR 702.34a: exiled instead of anywhere else
    }
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
    crate::kw::special_actions(g, p)
}

pub fn perform_special_action(g: &mut Game, p: PlayerId, sa: SpecialAction) -> Result<(), Illegal> {
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
    let out: Vec<ObjectId> = g
        .battlefield
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            !o.phased_out && o.controller == active && o.has_keyword(KeywordKind::Phasing)
        })
        .collect();
    let back: Vec<ObjectId> = g
        .battlefield
        .iter()
        .copied()
        .filter(|id| {
            let o = g.obj(*id);
            o.phased_out && o.controller == active && !o.phased_out_indirectly
        })
        .collect();
    phase_out(g, out);
    for id in back {
        phase_in(g, id);
    }
}

/// Phases permanents out, along with everything attached to them (CR 702.26g).
pub fn phase_out(g: &mut Game, objs: Vec<ObjectId>) {
    for id in objs {
        if g.obj(id).phased_out {
            continue;
        }
        g.objects[id.0 as usize].phased_out = true;
        crate::combat::remove_from_combat(g, id);
        g.emit(crate::events::Event::PhasedOut { obj: id });
        for a in g.attachments_of(Entity::Object(id)) {
            if !g.obj(a).phased_out {
                g.objects[a.0 as usize].phased_out = true;
                g.objects[a.0 as usize].phased_out_indirectly = true;
                // They phase out too, so "phases out" abilities see them (CR 603.10b).
                g.emit(crate::events::Event::PhasedOut { obj: a });
            }
        }
    }
    g.dirty = true;
}

pub fn phase_in(g: &mut Game, id: ObjectId) {
    g.objects[id.0 as usize].phased_out = false;
    for a in g.battlefield.clone() {
        if g.obj(a).phased_out_indirectly && g.obj(a).attached_to == Some(Entity::Object(id)) {
            let o = &mut g.objects[a.0 as usize];
            o.phased_out = false;
            o.phased_out_indirectly = false;
        }
    }
    g.emit(crate::events::Event::PhasedIn { obj: id });
    g.dirty = true;
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
