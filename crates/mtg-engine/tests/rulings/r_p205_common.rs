//! Shared helpers for the tests of rulings batch P205 (`r_p205_*.rs`): daybound,
//! deathtouch, defender, delirium, delve, demonstrate, descend, detain, dethrone, devoid,
//! devour, discover, domain. (The helpers of the S batches are used too.)

#![allow(dead_code)]

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether the permanent (followed across zone changes) has defender now.
pub fn has_defender(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).has_keyword(KeywordKind::Defender)
}

/// Whether the object is among the attackers of the current combat.
pub fn is_attacking(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_attacking(t.g.current(id))
}

/// Puts real cards into `p`'s graveyard.
pub fn bury(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    names.iter().map(|n| t.graveyard(p, n)).collect()
}

/// Four card types (land, creature, artifact, instant) in `p`'s graveyard: delirium.
pub fn delirium_graveyard(t: &mut TestGame, p: PlayerId) -> Vec<ObjectId> {
    bury(t, p, &["Forest", "Grizzly Bears", "Mind Stone", "Lightning Bolt"])
}
