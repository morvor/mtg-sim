//! Shared helpers for the tests of rulings batch P141 (`r_p141_*.rs`): repeatable artifact
//! tokens, recursion engines ("return it to your hand"), "reveal until" and cascade-style
//! free casts. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p057_common::{into_beginning_of_combat, into_upkeep, pump};
pub use crate::r_p079_common::back_face as transformed_back_face;
pub use crate::r_p104_common::{is_in, mill};
pub use crate::r_p108_common::{end_step, obj, put_counters};
pub use crate::r_p117_common::gain_control;
pub use crate::r_p146_common::{
    destroy_together, discard, enter_together, exile_together, fill_hand, n_asked,
    stacked_triggers,
};
pub use crate::r_p160_common::{activate_resolve, cast_resolve, yes};
pub use crate::r_p212_common::{back_face, grant};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, place, stack_library, supported, tokens,
    triggers_on_stack, with_subtype,
};
pub use crate::r_s02_common::{
    can_activate, can_cast, create_token, destroy, target_candidates,
};
pub use crate::r_s05_common::{enter, move_to, run_from};
pub use crate::r_s06_common::attach_new;
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// `p`'s count of the player counter `kind` (energy, experience, poison, ...).
pub fn player_counter(t: &TestGame, p: PlayerId, kind: &str) -> u32 {
    t.g.player(p).counters.get(kind).copied().unwrap_or(0)
}

/// The cards in `p`'s hand, by name.
pub fn hand_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .hand
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}

/// Whether some target choice offered to `p` since decision `from` had `e` as a candidate.
pub fn offered_target(t: &TestGame, p: PlayerId, from: usize, e: ObjectId) -> bool {
    target_candidates(t, p, from)
        .iter()
        .any(|c| c.contains(&Entity::Object(e)))
}

/// The object id the card (followed across zone changes) has now.
pub fn now(t: &TestGame, id: ObjectId) -> ObjectId {
    t.g.current(id)
}
