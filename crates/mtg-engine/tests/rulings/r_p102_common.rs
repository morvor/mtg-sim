//! Shared helpers for the tests of rulings batch P102 (`r_p102_*.rs`): alternative costs
//! and cards with no mana cost, "each player sacrifices" creatures, sacrifice-as-a-cost
//! creatures, land-type changers and untap restrictions, mass reanimation, mass -X/-0
//! effects, and assorted odd cards. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p016_common::{lots, o, stack_targets};
pub use crate::r_s01_common::{creatures, supported};
pub use crate::r_s08_common::mana_value;

/// The alternative-cost methods `p` could begin casting `card` with now.
pub fn alternatives(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    crate::r_s07_common::cast_methods(t, p, card)
        .into_iter()
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .collect()
}

/// Whether `p` may cast `card` normally (paying its mana cost) right now.
pub fn castable_normally(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    crate::r_s07_common::cast_methods(t, p, card).contains(&CastMethod::Normal)
}

/// The candidates of every entity choice asked of `p` since decision `from`.
pub fn entity_choices(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter(|(q, _)| *q == p)
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// Queues `p`'s answer to the next entity choice: the given objects.
pub fn choose(t: &mut TestGame, p: PlayerId, ids: &[ObjectId]) {
    t.answer(
        p,
        DecisionKind::Entities,
        Answer::Entities(ids.iter().map(|i| Entity::Object(*i)).collect()),
    );
}

/// Whether the object (followed across zone changes) is in the graveyard of its owner.
pub fn in_gy(t: &TestGame, id: ObjectId) -> bool {
    matches!(t.zone(id), mtg_engine::object::Zone::Graveyard(_))
}
