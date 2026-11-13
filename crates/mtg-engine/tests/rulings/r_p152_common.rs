//! Shared helpers for the tests of rulings batch P152 (`r_p152_*.rs`): permanents that
//! sacrifice themselves (as a cost or as part of a triggered ability's effect), and
//! abilities scaling with damage dealt or with mana value. (The helpers of earlier
//! batches are used too.)

#![allow(dead_code)]

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, put_counters, two_headed_giant};
pub use crate::r_p117_common::{exile, gain_control};
pub use crate::r_p151_common::{act, damage, next_upkeep, reanimate, steal_for_good};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, stack_library, supported, tokens, triggers_on_stack,
};
pub use crate::r_s02_common::{can_activate, destroy, target_candidates};
pub use crate::r_s13_common::{commander, commander_game};
pub use crate::r_s25_common::cast_new;

/// Adds `n` mana of each of `types` to `p`'s pool.
pub fn pool(t: &mut TestGame, p: PlayerId, types: &[ManaType], n: u32) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add_type(*ty, n);
    }
}

/// Sets `p`'s life total.
pub fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    t.g.players[p.idx()].life = life;
}

/// The number of permanents named `name` on the battlefield.
pub fn count_named(t: &TestGame, name: &str) -> usize {
    t.named_on_battlefield(name).len()
}

/// Returns a permanent to its owner's hand (as a spell or ability would) and settles.
pub fn bounce(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    let owner = t.g.obj(id).owner;
    t.g.move_object(
        id,
        mtg_engine::object::Zone::Hand(owner),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// `p` sacrifices the permanent (as an effect or cost would), then the game settles.
pub fn sacrifice(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let id = t.g.current(id);
    t.g.sacrifice(id, p);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// `p` gains `n` life, then the game settles.
pub fn gain(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.gain_life(p, n);
    t.g.flush_events();
    t.settle();
}

/// Whether the decision log since `from` holds a yes/no question for `p`.
pub fn asked_yes_no(t: &TestGame, p: PlayerId, from: usize) -> bool {
    t.asked()[from..].iter().any(|(q, d)| {
        *q == p && matches!(d, mtg_engine::decision::Decision::YesNo { .. })
    })
}

/// The source of the ability on top of the stack, if the top object is an ability.
pub fn top_source(t: &TestGame) -> Option<ObjectId> {
    use mtg_engine::object::StackKind;
    let top = *t.g.stack.last()?;
    match &t.g.obj(top).stack.as_deref()?.kind {
        StackKind::Activated { source, .. } | StackKind::Triggered { source, .. } => {
            Some(*source)
        }
        StackKind::Spell => None,
    }
}
