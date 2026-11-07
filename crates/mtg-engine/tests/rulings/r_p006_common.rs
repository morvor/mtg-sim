//! Shared helpers for rulings batch P006 (`r_p006_*.rs`): sacrifice-for-damage
//! ("bombard") abilities, bottom-of-library manipulation, permanents that return
//! themselves (or another permanent) to hand, and assorted burn.

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Number of decisions asked so far (a mark for the `*_since` helpers).
pub fn mark(t: &TestGame) -> usize {
    t.asked().len()
}

/// The entity choices (candidates, min, max) asked of `p` since decision `from`.
pub fn choices_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<(Vec<Entity>, u32, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities {
                candidates,
                min,
                max,
                ..
            } if *q == p => Some((candidates.clone(), *min, *max)),
            _ => None,
        })
        .collect()
}

/// The players asked for priority since decision `from`.
pub fn priority_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Priority { .. }))
        .map(|(p, _)| *p)
        .collect()
}

/// The yes/no questions asked of `p` since decision `from`.
pub fn yes_no_since(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

/// The top object of the stack.
pub fn top(t: &TestGame) -> ObjectId {
    *t.g.stack.last().expect("empty stack")
}

/// Every target chosen for the stack object `id`.
pub fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.g.obj(id)
        .stack
        .as_ref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|c| c.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

/// Triggered abilities on the stack whose source is (or was) `source`.
pub fn triggers_from(t: &TestGame, source: ObjectId) -> usize {
    let source = t.g.current(source);
    t.g.stack
        .iter()
        .filter(|id| {
            t.g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, StackKind::Triggered { source: s, .. }
                    if *s == source || t.g.current(*s) == source)
            })
        })
        .count()
}

/// Whether `id` is in its owner's hand (following it through zone changes).
pub fn in_hand(t: &TestGame, id: ObjectId) -> bool {
    let o = t.obj_now(id);
    o.zone == mtg_engine::object::Zone::Hand(o.owner)
}

/// Whether `id` is in its owner's graveyard (following it through zone changes).
pub fn in_graveyard(t: &TestGame, id: ObjectId) -> bool {
    let o = t.obj_now(id);
    o.zone == mtg_engine::object::Zone::Graveyard(o.owner)
}

/// The creatures `p` controls named `name`.
pub fn named_of(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// Sets the permanent's damage-free, older-than-this-turn state (as if it had been on the
/// battlefield since an earlier turn).
pub fn old(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].entered_turn = 0;
    t.g.objects[id.0 as usize].summoning_sick = false;
}
