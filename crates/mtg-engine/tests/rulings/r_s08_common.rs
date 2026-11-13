//! Shared helpers for the tests of rulings batch S08 (`r_s08_*.rs`): ferocious, fight,
//! flash, flashback, flurry, flying (and the omen cards of its sets), Food, forage,
//! foretell, formidable, freerunning, fuse. (The helpers of batches S01–S07 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Action, Decision};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether the object (followed across zone changes) is tapped now.
pub fn is_tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// The legal actions `p` has now (with priority).
pub fn actions_of(t: &mut TestGame, p: PlayerId) -> Vec<Action> {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p)
}

/// The methods `p` could cast `card` with now, as legal actions.
pub fn legal_cast_methods(t: &mut TestGame, p: PlayerId, card: ObjectId) -> Vec<CastMethod> {
    let card = t.g.current(card);
    actions_of(t, p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Cast { card: c, method } if c == card => Some(method),
            _ => None,
        })
        .collect()
}

/// Number of priority decisions asked of `p` since decision `from`.
pub fn priority_asks_of(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.asked()[from..]
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::Priority { .. }))
        .count()
}

/// The mana value of the object (followed across zone changes) now.
pub fn mana_value(t: &TestGame, id: ObjectId) -> i64 {
    t.g.mana_value_of(t.g.current(id)) as i64
}
