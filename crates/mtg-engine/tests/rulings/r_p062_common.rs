//! Shared helpers for the tests of rulings batch P062 (`r_p062_*.rs`): free sacrifice
//! outlets, and tapping effects with "doesn't untap during its controller's next untap
//! step" and stun counters (CR 502.3, 122.1d). (The helpers of earlier batches are used
//! too.)

#![allow(dead_code)]

use crate::r_s04_common::next_upkeep;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The object as an entity (a target or a choice).
pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Whether the object (followed across zone changes) is tapped now.
pub fn is_tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// Taps the permanent directly (as if it had attacked or been tapped for mana earlier).
pub fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].tapped = true;
}

/// A real creature card put onto the battlefield tapped under `p`'s control.
pub fn tapped_creature(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    tap(t, id);
    id
}

/// Runs through `p`'s next untap step (stopping in that upkeep).
pub fn through_untap_step(t: &mut TestGame, p: PlayerId) {
    next_upkeep(t, p);
}

/// The permanent stays tapped through `p`'s next untap step, then untaps as normal in the
/// one after it.
pub fn misses_one_untap(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    through_untap_step(t, p);
    assert!(
        is_tapped(t, id),
        "it untapped during its controller's next untap step"
    );
    through_untap_step(t, p);
    assert!(!is_tapped(t, id), "it didn't untap the step after that");
}

/// The permanent untaps as normal during `p`'s next untap step.
pub fn untaps_next(t: &mut TestGame, id: ObjectId, p: PlayerId) {
    through_untap_step(t, p);
    assert!(!is_tapped(t, id), "it didn't untap during the next untap step");
}
