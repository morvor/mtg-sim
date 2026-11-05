//! Shared helpers for rulings batch P027 (`r_p027_*.rs`): permanents that enter as or
//! become copies of creatures (CR 707.2, 707.9, 614.1c, 603.6a), tokens that enter
//! attacking (CR 508.4), and copies of spells and abilities (CR 707.10).

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The candidates of every "choose objects" decision asked since `from` (an index into
/// `t.asked()`), with the decision's minimum number of choices.
pub fn choices_since(t: &TestGame, from: usize) -> Vec<(Vec<Entity>, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                candidates, min, ..
            } => Some((candidates.clone(), *min)),
            _ => None,
        })
        .collect()
}

/// The number of abilities the object has whose text contains `s`.
pub fn abilities_with(t: &TestGame, id: ObjectId, s: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains(s))
        .count()
}

pub fn has_subtype(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.has_subtype(s)
}

pub fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

/// Puts `n` +1/+1 counters on the permanent (as a resolving effect would).
pub fn plus1(t: &mut TestGame, id: ObjectId, n: u32) {
    let id = t.g.current(id);
    t.g.add_counters(Entity::Object(id), counters::PLUS1, n, None);
    t.g.recompute();
}

/// Moves the permanent to its owner's graveyard as a destroy effect would, then settles.
pub fn kill(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
}
