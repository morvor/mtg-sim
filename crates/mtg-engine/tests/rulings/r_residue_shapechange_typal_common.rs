//! Shared helpers for the tests of item `rulings-residue-shapechange-typal`
//! (`r_residue_shapechange_typal_*.rs`): the open rulings left in batches P160, P161 and
//! P184 — changelings, shapeshifters (effects that set power and toughness, copies with
//! exceptions) and typal payoffs. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p108_common::obj;
pub use crate::r_p119_common::{giant_growth, stack};
pub use crate::r_p120_common::{give_plus1, plus1};
pub use crate::r_p121_common::{cast_resolve, damage};
pub use crate::r_residue_type_change_common::{
    creature_types, diminish, has_color, is_type, pt_now,
};
pub use crate::r_s01_common::{
    asked_since, attack_with, block_and_finish, place, stack_library, supported, triggers_on_stack,
};
pub use crate::r_s02_common::destroy;
pub use crate::r_s24_common::{choose_creature_type, controller, tapped};
pub use crate::r_s25_common::{cast_new, lands_for_cost, name_now};

/// A game with P0 in its precombat main phase with priority.
pub fn main_phase(n: usize) -> TestGame {
    let mut t = TestGame::new(n);
    t.set_step(P0, Step::PrecombatMain);
    t
}

/// The tokens `p` controls on the battlefield named `name`.
pub fn tokens_named_exactly(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}

/// Number of objects on the stack.
pub fn on_stack(t: &TestGame) -> usize {
    t.g.stack.len()
}
