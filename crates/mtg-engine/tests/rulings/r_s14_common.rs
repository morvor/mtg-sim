//! Shared helpers for the tests of rulings batch S14 (`r_s14_*.rs`): prototype, prowess,
//! radiance, raid, rally, ravenous, reach, read ahead, rebound, reconfigure, recruit,
//! regenerate, renew, renown, repartee, replicate, retrace, revolt, riot. (The helpers of
//! batches S01–S11 are used too.)

#![allow(dead_code)]

use crate::r_s01_common::give_mana_for;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts the real card `name` into `p`'s hand with the lands to pay its mana cost, and casts
/// it with the given targets (one per target slot). Returns the spell.
pub fn cast_from_hand(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    give_mana_for(t, p, name);
    let card = t.hand(p, name);
    t.cast_with(p, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// Number of triggered abilities on the stack whose source is `source`.
pub fn triggers_from(t: &TestGame, source: ObjectId) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            matches!(
                t.g.obj(**id).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Triggered { source: s, .. }) if *s == source
            )
        })
        .count()
}

/// Number of triggered abilities on the stack.
pub fn triggers_on_stack_now(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            matches!(
                t.g.obj(**id).stack.as_deref().map(|s| &s.kind),
                Some(StackKind::Triggered { .. })
            )
        })
        .count()
}
