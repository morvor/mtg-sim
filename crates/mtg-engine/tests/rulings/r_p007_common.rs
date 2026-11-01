//! Shared helpers for rulings batch P007 (`r_p007_*.rs`): burn spells and abilities that
//! damage players, planeswalkers and creatures.

#![allow(dead_code)]

use crate::r_s01_common::supported;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// The damage marked on the permanent now (0 if it left the battlefield).
pub fn dmg(t: &TestGame, id: ObjectId) -> u32 {
    if t.on_battlefield(id) {
        t.obj_now(id).damage
    } else {
        0
    }
}

/// Puts the planeswalker onto the battlefield for `p` with `loyalty` loyalty counters.
pub fn pw(t: &mut TestGame, p: PlayerId, name: &str, loyalty: u32) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize]
        .counters
        .insert("loyalty".into(), loyalty);
    id
}

/// Activates the planeswalker's `index`th loyalty ability (it stays on the stack).
pub fn loyalty(t: &mut TestGame, p: PlayerId, walker: ObjectId, index: usize, targets: &[Entity]) {
    t.activate(p, walker, index, targets)
        .unwrap_or_else(|e| panic!("loyalty ability {index}: {e:?}"));
}

/// Queues `p`'s division of damage (or counters) among the targets.
pub fn divide(t: &mut TestGame, p: PlayerId, amounts: &[i64]) {
    t.answer(p, DecisionKind::Divide, Answer::Numbers(amounts.to_vec()));
}

/// Puts the real card `name` into `p`'s hand with lands for its mana cost (plus `extra`
/// Mountains) and casts it with `method`, X = `x` (if any) and the given targets (one per
/// target slot). Returns the spell.
pub fn cast_x(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    method: CastMethod,
    x: Option<i64>,
    targets: &[Entity],
) -> ObjectId {
    supported(name);
    lands_for_cost(t, p, name);
    if let Some(x) = x {
        t.lands(p, "Wastes", x as usize);
    }
    let card = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    let mut b = t.cast(p, card).method(method);
    if let Some(x) = x {
        b = b.x(x);
    }
    b.try_go()
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// Gives `p` hexproof (a Leyline of Sanctity enters under their control), making them an
/// illegal target for their opponents' spells and abilities.
pub fn player_hexproof(t: &mut TestGame, p: PlayerId) {
    t.battlefield(p, "Leyline of Sanctity");
    t.g.recompute();
    t.settle();
}
