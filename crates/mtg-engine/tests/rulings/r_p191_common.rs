//! Shared helpers for rulings batch P191 (`r_p191_*.rs`): a grab bag of cards grouped by
//! "function: unique mana cost".

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// From P1's end step to P0's upkeep, with P0's upkeep triggers on the stack.
pub fn to_upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

/// The triggered abilities on the stack (bottom first) whose text contains `text`.
pub fn stacked_triggers(t: &TestGame, text: &str) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| {
            t.g.obj(*id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, StackKind::Triggered { ability, .. } if ability.text.contains(text))
            })
        })
        .collect()
}

/// Whether the stack object `id` is a triggered ability.
pub fn is_trigger(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(id)
        .stack
        .as_ref()
        .is_some_and(|si| matches!(si.kind, StackKind::Triggered { .. }))
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

/// The candidates of the entity choices asked since decision `from`.
pub fn entity_choices(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// The legal targets offered in the target choices asked since decision `from`.
pub fn target_choices(t: &TestGame, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}
