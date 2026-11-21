//! Shared helpers for the CR 105–113 tests (colors, mana, numbers and symbols, cards,
//! objects, permanents, tokens, spells, abilities).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::CardDef;
use mtg_engine::eval::Ctx;
use mtg_engine::mana::{Mana, ManaCost, ManaType};
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// A custom card compiled from oracle text with the real oracle compiler. Panics if any
/// of the text isn't understood. The mana cost (if any) also defines the card's colors
/// (CR 105.2).
pub fn card_from_text(
    name: &str,
    cost: &str,
    type_line: &str,
    pt: Option<(i32, i32)>,
    text: &str,
) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let p = pt.map(|x| x.0.to_string());
    let tt = pt.map(|x| x.1.to_string());
    let ctx = CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: p.as_deref(),
        toughness: tt.as_deref(),
    };
    let compiled = oracle::compile(text, &ctx);
    assert!(
        compiled.unsupported.is_empty(),
        "{name}: unsupported text {:?}",
        compiled.unsupported
    );
    let mana_cost = ManaCost::parse(cost);
    let colors = mana_cost.as_ref().map_or(ColorSet::NONE, |m| m.colors());
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        mana_cost,
        colors,
        supertypes: tl.supertypes,
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        abilities: compiled.abilities,
        power: pt.map(|x| x.0),
        toughness: pt.map(|x| x.1),
        rules_text: Arc::from(text),
        ..Default::default()
    })
}

/// A custom card with hand-built abilities. Colors come from the mana cost.
pub fn card_with(
    name: &str,
    cost: &str,
    type_line: &str,
    pt: Option<(i32, i32)>,
    abilities: Vec<Ability>,
) -> CardDef {
    let tl = TypeLine::parse(type_line);
    let mana_cost = ManaCost::parse(cost);
    let colors = mana_cost.as_ref().map_or(ColorSet::NONE, |m| m.colors());
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        mana_cost,
        colors,
        supertypes: tl.supertypes,
        card_types: tl.card_types,
        subtypes: tl.subtypes.into_iter().collect(),
        abilities,
        power: pt.map(|x| x.0),
        toughness: pt.map(|x| x.1),
        rules_text: Arc::from(""),
        ..Default::default()
    })
}

/// A vanilla creature with the given mana cost.
pub fn vanilla(name: &str, cost: &str, p: i32, t: i32) -> CardDef {
    card_with(name, cost, "Creature — Bear", Some((p, t)), vec![])
}

pub fn put(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    t.custom(p, def, Zone::Battlefield)
}

pub fn put_in_hand(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    t.custom(p, def, Zone::Hand(p))
}

pub fn put_in_graveyard(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    t.custom(p, def, Zone::Graveyard(p))
}

pub fn spell_ab(targets: Vec<TargetSpec>, effect: Effect) -> Ability {
    AbilityDef::new(
        AbilityKind::Spell(SpellAbility {
            body: Body::simple(targets, effect),
        }),
        "spell",
    )
}

pub fn static_ab(e: StaticEffect) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(e)), "static")
}

pub fn triggered_ab(cond: TriggerCond, effect: Effect) -> Ability {
    AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility::new(cond, Body::effect(effect))),
        "triggered",
    )
}

pub fn activated_ab(cost: Cost, effect: Effect) -> Ability {
    AbilityDef::new(
        AbilityKind::Activated(ActivatedAbility::new(cost, Body::effect(effect))),
        "activated",
    )
}

pub fn mana_ab(cost: Cost, effect: Effect) -> Ability {
    let mut a = ActivatedAbility::new(cost, Body::effect(effect));
    a.is_mana_ability = true;
    AbilityDef::new(AbilityKind::Activated(a), "mana ability")
}

pub fn gain_life(n: i32) -> Effect {
    Effect::GainLife {
        who: PlayerRef::You,
        n: Value::c(n),
    }
}

/// Adds unrestricted mana of the given types to a player's pool.
pub fn add_pool(t: &mut TestGame, p: PlayerId, types: &[ManaType]) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add(Mana::new(*ty));
    }
}

pub fn pool_count(t: &TestGame, p: PlayerId, ty: ManaType) -> usize {
    t.g.player(p).mana_pool.count(ty)
}

pub fn pool_total(t: &TestGame, p: PlayerId) -> usize {
    t.g.player(p).mana_pool.total()
}

pub fn colors(t: &TestGame, id: ObjectId) -> ColorSet {
    t.obj_now(id).chars.colors
}

pub fn cs(letters: &str) -> ColorSet {
    letters
        .chars()
        .filter_map(Color::from_letter)
        .collect::<ColorSet>()
}

/// Whether the object currently matches a filter, evaluated for `controller`.
pub fn matches(t: &TestGame, id: ObjectId, f: &Filter, controller: PlayerId) -> bool {
    t.g.matches(t.g.current(id), f, &Ctx::new(None, controller))
}

/// The `n`th activated ability uid of an object.
pub fn activated_uid(t: &TestGame, id: ObjectId, n: usize) -> u64 {
    t.g.obj(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .nth(n)
        .map(|a| a.uid)
        .expect("no such activated ability")
}

/// Options offered by the most recent `ChooseOption` decision asked of `p`.
pub fn last_options(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseOption { options, .. } if q == p => Some(options),
            _ => None,
        })
        .unwrap_or_default()
}

/// Legal targets for target slot `slot` of the spell ability of `card` (not yet cast),
/// as chosen by `p`.
pub fn spell_target_candidates(
    t: &TestGame,
    p: PlayerId,
    card: ObjectId,
    slot: usize,
) -> Vec<Entity> {
    let body = t.g.spell_body(card);
    let spec = body.targets[slot].clone();
    t.g.legal_target_candidates(&spec, &Ctx::new(Some(card), p), card)
}

/// Tokens currently on the battlefield controlled by `p`.
pub fn tokens_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.battlefield
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).is_token() && t.g.obj(*id).controller == p)
        .collect()
}
