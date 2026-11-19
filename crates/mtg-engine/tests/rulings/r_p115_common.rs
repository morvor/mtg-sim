//! Shared helpers for the tests of rulings batch P115 (`r_p115_*.rs`): pair commanders,
//! Bridge from Below, intervening "if" clauses, planeswalker statics, naming cards after
//! looking at a hand, per-player and per-upkeep triggers, phasing, excess damage and
//! pingers.

#![allow(dead_code)]

pub use crate::r_p009_common::{kill, obj, pl};
pub use crate::r_p016_common::{cast_at, lots};
pub use crate::r_p198_common::{act, act_only, ctl, pass_turn, to_upkeep};
pub use crate::r_s01_common::{
    asked_since, attack_with, creatures, stack_library, supported, tokens, with_subtype,
};
pub use crate::r_s06_common::give_control;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Moves an object to a zone (as an effect would), then checks triggers and SBAs.
pub fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) -> Option<ObjectId> {
    let id = t.g.current(id);
    let new = t.g.move_object(id, zone, MoveCause::Effect, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    new
}

/// The names of the cards in `p`'s graveyard.
pub fn gy_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .graveyard
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}

/// Number of creature tokens `p` controls with the given subtype.
pub fn tokens_of(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is_token() && o.chars.has_subtype(subtype))
        .count()
}

/// The texts of the objects on the stack, bottom first.
pub fn stack_texts(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .map(|id| {
            let o = t.g.obj(*id);
            match o.stack.as_ref().map(|s| &s.kind) {
                Some(mtg_engine::object::StackKind::Triggered { ability, .. })
                | Some(mtg_engine::object::StackKind::Activated { ability, .. }) => {
                    ability.text.to_string()
                }
                _ => o.chars.name.to_string(),
            }
        })
        .collect()
}
