//! Shared helpers for the tests of CR 702.1–702.10 (general keyword rules, deathtouch,
//! defender, double strike, enchant, equip, first strike, flash, flying, haste).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::CardDef;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// A custom card compiled from oracle text with the real oracle compiler. Panics if any
/// of the text isn't understood.
pub fn custom_card(
    name: &str,
    type_line: &str,
    cost: &str,
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
    let m = mtg_engine::mana::ManaCost::parse(cost);
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
        colors: m.as_ref().map_or(ColorSet::NONE, |m| m.colors()),
        mana_cost: m,
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

/// Puts a custom card onto the battlefield (not summoning sick).
pub fn bf(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    t.custom(p, def, Zone::Battlefield)
}

/// Asserts that a real card's oracle text is fully understood by the compiler.
pub fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// The keyword instances of `kind` a real card's compiled front face has.
pub fn printed_keywords(name: &str, kind: KeywordKind) -> Vec<Keyword> {
    card(name).faces[0]
        .chars
        .keywords()
        .filter(|k| k.kind == kind)
        .cloned()
        .collect()
}

/// Number of instances of a keyword an object currently has.
pub fn instances(t: &TestGame, id: ObjectId, kind: KeywordKind) -> usize {
    t.obj_now(id)
        .chars
        .keywords()
        .filter(|k| k.kind == kind)
        .count()
}

/// Queues the active player's attack declaration.
pub fn declare(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
}

/// Queues a defending player's block declaration.
pub fn block(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        dp,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
}

/// Advances until `step` of the current turn begins and the active player has priority.
/// Panics if the step doesn't happen this turn.
pub fn go_to(t: &mut TestGame, step: Step) {
    let ap = t.g.turn.active;
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == step && g.turn.stage == Stage::Priority && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(
        ok && t.g.turn.number == turn,
        "did not reach {step:?} this turn"
    );
}

/// Whether `step` happened this turn.
pub fn step_happened(t: &TestGame, step: Step) -> bool {
    t.g.turn.step_log.contains(&step)
}

/// The creatures that could be declared as attackers now.
pub fn can_attack(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    mtg_engine::combat::attack_options(&t.g)
        .iter()
        .any(|(c, _)| *c == id)
}

pub fn is_attacking(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_attacking(id)
}

pub fn is_blocked(t: &TestGame, a: ObjectId) -> bool {
    t.g.combat.as_ref().is_some_and(|c| c.is_blocked(a))
}

/// Executes an effect as if a spell/ability controlled by `controller` resolved with
/// `targets` in target slot 0.
pub fn apply(t: &mut TestGame, controller: PlayerId, effect: Effect, targets: &[ObjectId]) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, controller);
    ctx.targets = vec![targets.iter().map(|o| Entity::Object(*o)).collect()];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// "[Target] gains [keyword] until end of turn."
pub fn grant(t: &mut TestGame, id: ObjectId, kw: Keyword) {
    apply(
        t,
        PlayerId(0),
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(kw)],
            duration: Duration::EndOfTurn,
        },
        &[id],
    );
}

/// "[Target] loses [keyword] until end of turn."
pub fn remove_kw(t: &mut TestGame, id: ObjectId, kind: KeywordKind) {
    apply(
        t,
        PlayerId(0),
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveKeyword(kind)],
            duration: Duration::EndOfTurn,
        },
        &[id],
    );
}

/// Damage marked on an object (following it if it changed zones).
pub fn damage(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}
