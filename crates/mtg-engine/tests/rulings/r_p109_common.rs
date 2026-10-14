//! Shared helpers for rulings batch P109 (`r_p109_*.rs`): multi-target removal, divided
//! damage, land ramp, multiplayer and multiple-body effects.

#![allow(dead_code)]

use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Queues `p`'s division of damage.
pub fn divide(t: &mut TestGame, p: PlayerId, amounts: &[i64]) {
    t.answer(p, DecisionKind::Divide, Answer::Numbers(amounts.to_vec()));
}

/// The division of a stack object (all slots).
pub fn division_of(t: &TestGame, id: ObjectId) -> Vec<u32> {
    t.g.obj(id)
        .stack
        .as_deref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|m| m.divided.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

/// Casts the real card `name` for P0 (lands for its mana cost plus `x` Mountains for {X})
/// with the given targets and division (empty: no division answer). Returns the result.
pub fn try_cast_divided(
    t: &mut TestGame,
    name: &str,
    x: Option<i64>,
    targets: &[Entity],
    division: &[i64],
) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    lands_for_cost(t, P0, name);
    if let Some(x) = x {
        t.lands(P0, "Mountain", x as usize);
    }
    if !division.is_empty() {
        divide(t, P0, division);
    }
    let card = t.hand(P0, name);
    let mut b = t.cast(P0, card).targets(targets);
    if let Some(x) = x {
        b = b.x(x);
    }
    let r = b.try_go();
    t.clear_answers();
    r
}

/// As [`try_cast_divided`], panicking if casting fails.
pub fn cast_divided(
    t: &mut TestGame,
    name: &str,
    x: Option<i64>,
    targets: &[Entity],
    division: &[i64],
) -> ObjectId {
    try_cast_divided(t, name, x, targets, division)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}
