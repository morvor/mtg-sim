//! Shared helpers for the tests of item `rulings-residue-misc-2`, pass C
//! (`r_residue_misc_2c_*.rs`): the open rulings left in batches P201–P204, P206, P216, P218,
//! P219, P221, P223, P224 and P225. The helpers of passes A and B are reused.

#![allow(dead_code)]

pub use crate::r_residue_misc_2b_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The permanents on the battlefield named `name` that `p` controls.
pub fn named_by(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj_now(*id).controller == p)
        .collect()
}

/// The number of permanents `p` controls.
pub fn n_permanents(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents_controlled_by(p).len()
}

/// Whether the object (followed across zone changes) is a creature now.
pub fn is_creature(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).is(CardType::Creature)
}

/// The decisions asked since index `from` that match `f`, with who was asked.
pub fn asked_matching(
    t: &TestGame,
    from: usize,
    f: impl Fn(&Decision) -> bool,
) -> Vec<(PlayerId, Decision)> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| f(d))
        .cloned()
        .collect()
}

/// The index, among the decisions asked since `from`, of the first one matching `f`.
pub fn first_asked(t: &TestGame, from: usize, f: impl Fn(&(PlayerId, Decision)) -> bool) -> usize {
    t.asked()[from..]
        .iter()
        .position(f)
        .unwrap_or_else(|| panic!("no such decision asked"))
}

/// The prompt texts of the decisions asked since `from` (for debugging assertions).
pub fn prompts_since(t: &TestGame, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .map(|(p, d)| format!("{p:?}: {d:?}"))
        .collect()
}

/// The tokens `p` controls that have the subtype `subtype`.
pub fn tokens_with(t: &TestGame, p: PlayerId, subtype: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .map(|o| o.id)
        .collect()
}

/// Whether the object (followed across zone changes) is goaded by anyone now.
pub fn goaded(t: &TestGame, id: ObjectId) -> bool {
    !t.obj_now(id).goaded_by.is_empty()
}
