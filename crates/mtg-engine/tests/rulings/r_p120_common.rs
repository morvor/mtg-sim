//! Shared helpers for the tests of rulings batch P120 (`r_p120_*.rs`): "+1/+1 counters
//! matter" — replacement effects that add to or multiply the counters put on permanents,
//! abilities that look back at the counters a permanent had as it left the battlefield,
//! triggers on counters being put, and spells and abilities whose targets must have
//! +1/+1 counters. (The helpers of batches S01–S30 and P108 are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p108_common::{end_step, obj, put_counters, resolved, two_headed_giant};
pub use crate::r_s01_common::{attack_with, block_and_finish, supported, triggers_on_stack};
pub use crate::r_s02_common::{create_token, destroy};
pub use crate::r_s25_common::{cast_new, creature_tokens, lands_for_cost};

pub const PLUS1: &str = "+1/+1";
pub const MINUS1: &str = "-1/-1";

/// The +1/+1 counters on the object (followed across zone changes).
pub fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(t.g.current(id), PLUS1)
}

/// Puts `n` +1/+1 counters on the object as one event and settles.
pub fn give_plus1(t: &mut TestGame, id: ObjectId, n: u32) {
    put_counters(t, id, PLUS1, n);
}

/// Puts `n` -1/-1 counters on the object as one event, then settles (state-based actions
/// and triggers).
pub fn give_minus1(t: &mut TestGame, id: ObjectId, n: u32) {
    put_counters(t, id, MINUS1, n);
}

/// The creature tokens `p` controls with the given subtype.
pub fn tokens_named(t: &TestGame, p: PlayerId, subtype: &str) -> usize {
    crate::r_p108_common::tokens_with(t, p, subtype)
}

/// Gains `n` life for `p` as one event (as a resolving effect would), then settles.
pub fn gain_life(t: &mut TestGame, p: PlayerId, n: i32) {
    use mtg_engine::ability::{Effect, PlayerRef, Value};
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::GainLife {
            who: PlayerRef::You,
            n: Value::Const(n),
        },
        &mut ctx,
    );
    t.g.flush_events();
    t.settle();
}
