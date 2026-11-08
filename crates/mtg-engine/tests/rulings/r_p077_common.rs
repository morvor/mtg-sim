//! Shared helpers for the tests of rulings batch P077 (`r_p077_*.rs`): cards that give
//! vigilance, and cards that use cards in a graveyard (exiling them as a cost or an
//! effect). (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Whether the object (followed across zone changes) has vigilance now.
pub fn vig(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).has_keyword(KeywordKind::Vigilance)
}

/// A Mutavault that becomes a 2/2 creature ("{1}: Until end of turn, this land becomes a
/// 2/2 creature with all creature types") once its ability resolves: a noncreature
/// permanent that becomes a creature later in the turn.
pub fn animate_mutavault(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let vault = t.battlefield(p, "Mutavault");
    t.lands(p, "Wastes", 1);
    crate::r_s06_common::activate_containing(t, p, vault, "becomes")
        .expect("Mutavault's ability");
    t.resolve_all();
    assert!(crate::r_s04_common::is_creature(t, vault));
    vault
}

/// The number of cards in `p`'s graveyard and in exile.
pub fn yard_and_exile(t: &TestGame, p: PlayerId) -> (usize, usize) {
    (t.graveyard_size(p), t.g.exile.len())
}
