//! Shared helpers for the CR 609–616 tests (effects, continuous effects, layers,
//! replacement and prevention effects): custom rules-level objects and a way to resolve
//! an arbitrary effect as a spell.

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// Characteristics of a custom card with a mana cost of {0} (an object with no mana
/// cost couldn't be cast, CR 118.6).
pub fn chars(name: &str) -> Characteristics {
    Characteristics {
        name: SmolStr::new(name),
        rules_text: Arc::from(""),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        ..Default::default()
    }
}

pub fn colors(cs: &[Color]) -> ColorSet {
    let mut s = ColorSet::NONE;
    for c in cs {
        s.insert(*c);
    }
    s
}

pub fn types(ts: &[CardType]) -> CardTypeSet {
    let mut s = CardTypeSet::default();
    for t in ts {
        s.insert(*t);
    }
    s
}

/// A creature card with the given P/T, colors, and abilities.
pub fn creature_with(name: &str, p: i32, t: i32, cs: &[Color], abilities: Vec<Ability>) -> CardDef {
    let mut c = chars(name);
    c.card_types = CardTypeSet::single(CardType::Creature);
    c.power = Some(p);
    c.toughness = Some(t);
    c.colors = colors(cs);
    c.abilities = abilities;
    CardDef::custom(c)
}

pub fn creature(name: &str, p: i32, t: i32, cs: &[Color]) -> CardDef {
    creature_with(name, p, t, cs, vec![])
}

/// A permanent card of the given types with the given abilities.
pub fn permanent(name: &str, ts: &[CardType], abilities: Vec<Ability>) -> CardDef {
    let mut c = chars(name);
    c.card_types = types(ts);
    c.abilities = abilities;
    CardDef::custom(c)
}

/// An instant with the given body (mana cost {0}).
pub fn instant(name: &str, body: Body) -> CardDef {
    let mut c = chars(name);
    c.card_types = CardTypeSet::single(CardType::Instant);
    c.abilities = vec![AbilityDef::new(
        AbilityKind::Spell(SpellAbility { body }),
        name,
    )];
    CardDef::custom(c)
}

pub fn static_ab(effect: StaticEffect) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(effect)), "static")
}

/// A static ability generating a continuous effect.
pub fn continuous(affected: Filter, mods: Vec<Modification>) -> Ability {
    static_ab(StaticEffect::Continuous { affected, mods })
}

/// A continuous effect that applies only "as long as" the condition holds.
pub fn continuous_if(cond: Condition, affected: Filter, mods: Vec<Modification>) -> Ability {
    let mut s = StaticAbility::new(StaticEffect::Continuous { affected, mods });
    s.condition = Some(cond);
    AbilityDef::new(AbilityKind::Static(s), "static")
}

/// A characteristic-defining ability (CR 604.3) of the object itself.
pub fn cda(mods: Vec<Modification>) -> Ability {
    let mut s = StaticAbility::new(StaticEffect::Continuous {
        affected: Filter::Source,
        mods,
    });
    s.is_cda = true;
    s.zone = FunctionZone::Anywhere;
    AbilityDef::new(AbilityKind::Static(s), "cda")
}

/// A static replacement or prevention effect.
pub fn replacement(event: ReplacementEvent, action: ReplacementAction) -> Ability {
    static_ab(StaticEffect::Replacement(ReplacementDef {
        event,
        action,
        self_replacement: false,
        optional: false,
    }))
}

pub fn restriction(r: Restriction) -> Ability {
    static_ab(StaticEffect::Restriction(r))
}

pub fn keyword(k: KeywordKind) -> Ability {
    AbilityDef::new(AbilityKind::Keyword(Keyword::new(k)), k.name())
}

pub fn triggered(trigger: TriggerCond, effect: Effect) -> Ability {
    AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility::new(trigger, Body::effect(effect))),
        "trigger",
    )
}

pub fn activated(cost: Cost, body: Body) -> Ability {
    AbilityDef::new(
        AbilityKind::Activated(ActivatedAbility::new(cost, body)),
        "activated",
    )
}

pub fn target_creature() -> TargetSpec {
    TargetSpec::object(Filter::creature(), "target creature")
}

pub fn target_any() -> TargetSpec {
    TargetSpec::any_target()
}

pub fn pt(p: i32, t: i32) -> Modification {
    Modification::ModifyPT(Value::c(p), Value::c(t))
}

pub fn set_pt(p: i32, t: i32) -> Modification {
    Modification::SetPT(Some(Value::c(p)), Some(Value::c(t)))
}

/// Casts a custom instant with the given body and targets (one per target slot) and
/// resolves it. Returns the id the spell had on the stack.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, body: Body, targets: &[Entity]) -> ObjectId {
    let card = t.custom(p, instant("Test Spell", body), Zone::Hand(p));
    let spell = t
        .cast_with(p, card, targets)
        .expect("casting the test spell failed");
    t.resolve();
    spell
}

/// Resolves a single effect as an instant with one target slot per `targets` entry.
pub fn resolve_effect(
    t: &mut TestGame,
    p: PlayerId,
    specs: Vec<TargetSpec>,
    targets: &[Entity],
    e: Effect,
) -> ObjectId {
    cast_resolve(t, p, Body::simple(specs, e), targets)
}

/// "Target creature gets +P/+T until end of turn" (or any mods) as a resolving spell.
pub fn modify_target(t: &mut TestGame, p: PlayerId, target: ObjectId, mods: Vec<Modification>) {
    resolve_effect(
        t,
        p,
        vec![target_creature()],
        &[Entity::Object(target)],
        Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
    );
}

pub fn has_kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

pub fn color_of(t: &TestGame, id: ObjectId) -> ColorSet {
    t.obj_now(id).chars.colors
}

pub fn put_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), kind, n, None);
    t.g.recompute();
}
