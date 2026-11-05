//! Shared helpers for the tests of rulings batch P195 (`r_p195_*.rs`): permanents that
//! untap themselves or other permanents, "enters untapped/tapped" effects, unprinted
//! tokens and effects that last indefinitely.

#![allow(dead_code)]

pub use crate::r_p009_common::{kill, obj, pl};
pub use crate::r_p016_common::{cast_at, cast_multi, lots, stack_targets, target_choices};
pub use crate::r_p194_common::{end_step, mana};
pub use crate::r_p198_common::{act, act_only, ctl, pass_turn, to_upkeep};
pub use crate::r_s01_common::{creatures, supported, tokens, with_subtype};
pub use crate::r_s33_common::{set_life, two_headed_giant};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether the permanent is tapped now.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// Taps the permanent (as an effect would) and settles.
pub fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.tap(id);
    t.g.flush_events();
    t.settle();
}

/// The name of the object on top of the stack.
pub fn top_name(t: &TestGame) -> String {
    let id = *t.g.stack.last().expect("empty stack");
    t.g.obj(id).chars.name.to_string()
}

/// Whether the object on top of the stack is a triggered ability of `src`.
pub fn top_is_trigger_of(t: &TestGame, src: ObjectId) -> bool {
    let Some(&id) = t.g.stack.last() else {
        return false;
    };
    matches!(
        t.g.obj(id).stack.as_ref().map(|si| &si.kind),
        Some(mtg_engine::object::StackKind::Triggered { source, .. })
            if *source == src || *source == t.g.current(src)
    )
}

/// Whether `id` is on the stack.
pub fn on_stack(t: &TestGame, id: ObjectId) -> bool {
    t.g.stack.contains(&id)
}

/// Whether `id` is an attacking creature now.
pub fn attacking(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.combat
        .as_ref()
        .is_some_and(|c| c.attackers.iter().any(|a| a.id == id))
}
