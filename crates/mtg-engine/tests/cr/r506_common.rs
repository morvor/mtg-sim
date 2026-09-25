//! Shared helpers for the combat and ending phase tests (CR 506–514).

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::CardDef;
use mtg_engine::decision::Decision;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

/// A custom card compiled from oracle text with the real oracle compiler.
pub fn custom_card(name: &str, type_line: &str, pt: Option<(i32, i32)>, text: &str) -> CardDef {
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
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
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

/// A custom card with hand-built abilities.
pub fn custom_with(
    name: &str,
    type_line: &str,
    pt: Option<(i32, i32)>,
    abilities: Vec<Ability>,
) -> CardDef {
    let tl = TypeLine::parse(type_line);
    CardDef::custom(Characteristics {
        name: SmolStr::new(name),
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

/// A vanilla creature.
pub fn vanilla(name: &str, p: i32, t: i32) -> CardDef {
    custom_with(name, "Creature — Test", Some((p, t)), vec![])
}

/// Puts a custom card onto the battlefield (not summoning sick).
pub fn bf(t: &mut TestGame, p: PlayerId, def: CardDef) -> ObjectId {
    t.custom(p, def, Zone::Battlefield)
}

pub fn static_ab(e: StaticEffect) -> Ability {
    AbilityDef::new(AbilityKind::Static(StaticAbility::new(e)), "static")
}

pub fn restriction(r: Restriction) -> Ability {
    static_ab(StaticEffect::Restriction(r))
}

pub fn triggered(cond: TriggerCond, effect: Effect) -> Ability {
    AbilityDef::new(
        AbilityKind::Triggered(TriggeredAbility::new(cond, Body::effect(effect))),
        "triggered",
    )
}

/// "Gain N life" effect for trigger bookkeeping.
pub fn gain(n: i32) -> Effect {
    Effect::GainLife {
        who: PlayerRef::You,
        n: Value::c(n),
    }
}

/// Moves the game to the beginning of combat of `ap`'s turn with combat state set up.
pub fn to_combat(t: &mut TestGame, ap: PlayerId) {
    t.set_step(ap, Step::BeginningOfCombat);
}

/// Queues the active player's attack declaration.
pub fn declare(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    t.answer(ap, DecisionKind::Attackers, Answer::Attackers(attackers.to_vec()));
}

/// Queues a defending player's block declaration.
pub fn block(t: &mut TestGame, dp: PlayerId, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(dp, DecisionKind::Blockers, Answer::Blockers(blocks.to_vec()));
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
    assert!(ok && t.g.turn.number == turn, "did not reach {step:?} this turn");
}

/// Advances until the given step begins (turn-based actions done), in any turn.
pub fn go_to_any(t: &mut TestGame, step: Step) {
    let ok = t.g.run_until(10_000, |g| g.turn.step == step && g.turn.stage == Stage::Priority);
    assert!(ok, "did not reach {step:?}");
}

/// Steps the game begun this turn, from the turn's step log.
pub fn steps_this_turn(t: &TestGame) -> Vec<Step> {
    t.g.turn.step_log.clone()
}

/// Number of times `p` was asked to declare attackers/blockers etc.
pub fn count_asked(t: &TestGame, p: PlayerId, f: impl Fn(&Decision) -> bool) -> usize {
    t.asked().iter().filter(|(q, d)| *q == p && f(d)).count()
}

/// The attackers and what they're attacking.
pub fn attacking(t: &TestGame) -> Vec<(ObjectId, Option<Entity>)> {
    t.g.combat
        .as_ref()
        .map(|c| c.attackers.iter().map(|a| (a.id, a.target)).collect())
        .unwrap_or_default()
}

pub fn is_blocked(t: &TestGame, a: ObjectId) -> bool {
    t.g.combat.as_ref().is_some_and(|c| c.is_blocked(a))
}

/// Puts a custom card onto the battlefield (from exile) as an effect would, attacking
/// and/or blocking and under the given controller (CR 508.4, 509.4). Returns the new
/// permanent.
pub fn enter_with(
    t: &mut TestGame,
    controller: PlayerId,
    def: CardDef,
    attacking: Option<Entity>,
    blocking: Option<ObjectId>,
) -> ObjectId {
    let id = t.custom(controller, def, Zone::Exile);
    let new = t
        .g
        .move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(controller),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(controller),
                attacking,
                blocking,
                ..Default::default()
            },
            source: None,
        })
        .expect("entered");
    t.g.flush_events();
    new
}
