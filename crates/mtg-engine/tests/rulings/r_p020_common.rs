//! Shared helpers for the tests of rulings batch P020 (`r_p020_*.rs`): creatures and tokens
//! that are copies of creatures — what the copy's "enters" and "enters with" abilities do
//! (CR 707.2, 603.6a, 614.1c), tokens that enter attacking (CR 508.4), and copy effects on
//! permanents already on the battlefield (CR 707.2, 613.2).

#![allow(dead_code)]

use crate::r_s01_common::tokens;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The creature every copy in these tests copies: Aven Riftwatcher, a 2/3 flyer with
/// vanishing 3 ("This creature enters with three time counters on it.") and "When this
/// creature enters or leaves the battlefield, you gain 2 life."
pub const RIFTWATCHER: &str = "Aven Riftwatcher";

/// Asserts that `id` (followed across zone changes) is an Aven Riftwatcher that entered
/// as one: its "enters with" ability (vanishing) put three time counters on it.
pub fn entered_as_riftwatcher(t: &TestGame, id: ObjectId) {
    let o = t.obj_now(id);
    assert_eq!(o.chars.name, RIFTWATCHER, "{id:?} isn't a copy of {RIFTWATCHER}");
    assert_eq!(
        t.counters(id, counters::TIME),
        3,
        "the copy's \"enters with\" ability didn't apply"
    );
}

/// The tokens `p` controls that aren't in `before`.
pub fn new_tokens_of(t: &TestGame, p: PlayerId, before: &[ObjectId]) -> Vec<ObjectId> {
    tokens(t, p)
        .into_iter()
        .filter(|id| !before.contains(id))
        .collect()
}

/// Asserts that `n` copies of Aven Riftwatcher entered for `p` since the snapshot of
/// tokens `before` and the life total `life`: each has its time counters and each one's
/// "enters" trigger gained `p` 2 life (the stack must have been resolved).
pub fn riftwatcher_tokens_entered(
    t: &TestGame,
    p: PlayerId,
    before: &[ObjectId],
    life: i32,
    n: usize,
) -> Vec<ObjectId> {
    let new = new_tokens_of(t, p, before);
    assert_eq!(new.len(), n, "expected {n} new tokens");
    for id in &new {
        entered_as_riftwatcher(t, *id);
    }
    assert_eq!(
        t.life(p),
        life + 2 * n as i32,
        "the copies' \"enters\" triggers didn't all resolve"
    );
    new
}
