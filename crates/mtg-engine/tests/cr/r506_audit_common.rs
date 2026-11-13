//! Shared helpers for the combat audit tests (`r50x_audit_*.rs`, CR 506–511): card
//! support checks, keyword grants and removals, and quick views of the combat state.

#![allow(dead_code)]

use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Asserts the compiler fully supports `name` (a test on a half-supported card would
/// test the wrong thing).
pub fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn exec(t: &mut TestGame, controller: PlayerId, effect: Effect, target: ObjectId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, controller);
    ctx.targets = vec![vec![Entity::Object(target)]];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// "Target creature gains [keyword] until end of turn."
pub fn give(t: &mut TestGame, id: ObjectId, kw: KeywordKind) {
    exec(
        t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddKeyword(Keyword::new(kw))],
            duration: Duration::EndOfTurn,
        },
        id,
    );
}

/// "Target creature loses [keyword] until end of turn."
pub fn take(t: &mut TestGame, id: ObjectId, kw: KeywordKind) {
    exec(
        t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveKeyword(kw)],
            duration: Duration::EndOfTurn,
        },
        id,
    );
}

/// "Target creature gets +p/+0 until end of turn."
pub fn pump(t: &mut TestGame, id: ObjectId, p: i32) {
    exec(
        t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(0))],
            duration: Duration::EndOfTurn,
        },
        id,
    );
}

/// "Remove target creature from combat."
pub fn remove_from_combat(t: &mut TestGame, id: ObjectId) {
    mtg_engine::combat::remove_from_combat(&mut t.g, id);
    t.g.recompute();
}

/// Queues `p`'s answer to the next combat damage assignment.
pub fn assign(t: &mut TestGame, p: PlayerId, v: &[i64]) {
    t.answer(p, DecisionKind::Damage, Answer::Numbers(v.to_vec()));
}

/// The combat damage assignment decisions asked so far: (player, creature, recipients,
/// lethal amounts, trample).
pub fn damage_decisions(t: &TestGame) -> Vec<(PlayerId, ObjectId, Vec<Entity>, Vec<u32>, bool)> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::AssignCombatDamage {
                creature,
                recipients,
                lethal,
                trample,
                ..
            } => Some((p, creature, recipients, lethal, trample)),
            _ => None,
        })
        .collect()
}

/// What `a` is attacking, if it's attacking.
pub fn attacks(t: &TestGame, a: ObjectId) -> Option<Entity> {
    t.g.combat.as_ref().and_then(|c| c.attack_target(a))
}

/// The attacking creatures.
pub fn attackers(t: &TestGame) -> Vec<ObjectId> {
    t.g.attackers()
}

/// The creatures `b` is blocking.
pub fn blocking(t: &TestGame, b: ObjectId) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| c.blocking(b))
        .unwrap_or_default()
}

/// The creatures blocking `a`.
pub fn blockers_of(t: &TestGame, a: ObjectId) -> Vec<ObjectId> {
    t.g.combat
        .as_ref()
        .map(|c| c.blockers_of(a))
        .unwrap_or_default()
}

pub fn blocked(t: &TestGame, a: ObjectId) -> bool {
    t.g.combat.as_ref().is_some_and(|c| c.is_blocked(a))
}

pub fn damage(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}
