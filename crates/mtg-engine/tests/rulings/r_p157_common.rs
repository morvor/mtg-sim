//! Shared helpers for the tests of rulings batch P157 (`r_p157_*.rs`): "selective group
//! hug" — cards that give an opponent something (cards, tokens, life, control of a
//! permanent), and the cards the batch groups with them.

#![allow(dead_code)]

pub use crate::r_p009_common::{gain, kill, lose, obj, pl};
pub use crate::r_p016_common::{cast_at, cast_multi, lots, stack_targets, target_choices};
pub use crate::r_p194_common::{damage, end_step, mana};
pub use crate::r_p195_common::{on_stack, tap, tapped, top_is_trigger_of, top_name};
pub use crate::r_p198_common::{act, act_only, ctl, pass_turn, to_upkeep};
pub use crate::r_s01_common::{
    creatures, place, stack_library, supported, tokens, triggers_on_stack, with_subtype,
};
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The candidates of the non-targeted choices asked of `p` since decision `from` whose
/// prompt contains `text` (all of them when `text` is empty).
pub fn choices(t: &TestGame, p: PlayerId, from: usize, text: &str) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities {
                candidates, prompt, ..
            } if *q == p && prompt.contains(text) => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// Number of decisions asked so far (a mark for [`choices`] and [`target_choices`]).
pub fn mark(t: &TestGame) -> usize {
    t.asked().len()
}

/// The controllers of the objects on the stack, bottom first.
pub fn stack_controllers(t: &TestGame) -> Vec<PlayerId> {
    t.g.stack.iter().map(|id| t.g.obj(*id).controller).collect()
}

/// `p` leaves the game (loses it), then state-based actions and triggers settle.
pub fn leave(t: &mut TestGame, p: PlayerId) {
    t.g.lose_game(p);
    t.settle();
    t.g.recompute();
}

/// Untaps the permanent (as an effect would) and settles.
pub fn untap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.untap(id);
    t.g.flush_events();
    t.settle();
}

/// Whether some permanent named `name` is on the battlefield.
pub fn named_here(t: &TestGame, name: &str) -> bool {
    !t.named_on_battlefield(name).is_empty()
}

/// The permanents `p` controls named `name`.
pub fn named_of(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}
