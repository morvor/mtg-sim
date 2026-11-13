//! Shared helpers for the tests of CR 702.178–702.195 (max speed through storied).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// Asserts that each real card's oracle text compiled completely.
pub fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// A custom card compiled from oracle text with the real oracle compiler.
pub fn custom_card(
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
    let mana_cost = mtg_engine::mana::ManaCost::parse(cost);
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        colors: mana_cost.as_ref().map_or(ColorSet::NONE, |m| m.colors()),
        mana_cost,
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

/// Puts a custom card into a zone.
pub fn put(t: &mut TestGame, p: PlayerId, def: CardDef, zone: Zone) -> ObjectId {
    t.custom(p, def, zone)
}

/// A player's speed (`None`: no speed, CR 702.179b).
pub fn speed(t: &TestGame, p: PlayerId) -> Option<u32> {
    t.g.player(p).speed
}

/// Sets a player's speed.
pub fn set_speed(t: &mut TestGame, p: PlayerId, s: Option<u32>) {
    t.g.players[p.idx()].speed = s;
    t.recompute();
}

/// The decisions of a kind asked since `from` (an index into the decision log).
pub fn asked_since(t: &TestGame, from: usize) -> Vec<(PlayerId, Decision)> {
    t.asked()[from..].to_vec()
}

/// Answers the next "choose entities" decision of `p` with these objects.
pub fn choose_objects(t: &mut TestGame, p: PlayerId, objs: &[ObjectId]) {
    let es: Vec<Entity> = objs.iter().map(|o| Entity::Object(*o)).collect();
    t.answer(p, DecisionKind::Entities, Answer::Entities(es));
}

/// The ids of the permanents named `name`.
pub fn named(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
}

/// Ids of the objects in a player's graveyard.
pub fn graveyard(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.player(p).graveyard.clone()
}

/// Number of cards named `name` in exile.
pub fn exiled_count(t: &TestGame, name: &str) -> usize {
    t.g.find_in_zone(Zone::Exile, name).len()
}

/// The object a card became after it left the zone it was in, following it.
pub fn now(t: &TestGame, id: ObjectId) -> ObjectId {
    t.g.current(id)
}

/// Adds mana of the given types to a player's pool.
pub fn add_mana(t: &mut TestGame, p: PlayerId, types: &[mtg_engine::mana::ManaType]) {
    for ty in types {
        t.g.players[p.idx()]
            .mana_pool
            .add(mtg_engine::mana::Mana::new(*ty));
    }
}

/// Whether an ability with this text prefix is among the object's abilities.
pub fn has_ability_text(t: &TestGame, id: ObjectId, prefix: &str) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.starts_with(prefix))
}

/// Whether the ability is a triggered ability.
pub fn is_triggered(a: &AbilityDef) -> bool {
    matches!(a.kind, AbilityKind::Triggered(_))
}

/// Performs an effect as if a spell or ability `controller` controls resolved it (with
/// these targets in its first target slot), then detects triggers.
pub fn run(
    t: &mut TestGame,
    controller: PlayerId,
    source: Option<ObjectId>,
    effect: Effect,
    targets: &[Entity],
) -> mtg_engine::eval::Ctx {
    let mut ctx = mtg_engine::eval::Ctx::new(source, controller);
    ctx.targets = vec![targets.to_vec()];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
    ctx
}

/// Makes `p` lose `n` life (not from damage) and detects triggers.
pub fn lose_life(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.lose_life(p, n);
    t.g.flush_events();
}
