//! Shared helpers for the tests of rulings batch P124 (`r_p124_*.rs`): cards that protect
//! all of a player's permanents or creatures (phasing, "your life total can't change",
//! indestructible and hexproof lords, mass exile-and-return, persist). (The helpers of
//! earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::ability::Modification;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub use crate::r_p121_common::{damage, exile_now};
pub use crate::r_p123_common::*;
pub use crate::r_p125_common::choose_creature_type;
pub use crate::r_p127_common::{bounce, sacrifice};
pub use crate::r_s25_common::cast_new;
pub use crate::r_s26_common::modify_until_eot;
pub use crate::r_p108_common::put_counters;
pub use crate::r_s02_common::create_token;
pub use crate::r_s30_common::set_life;

/// Whether the permanent (followed across zone changes) is phased out.
pub fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

/// Advances to P0's next upkeep (P0's permanents phase in during its untap step).
pub fn to_next_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

/// P0 casts the real card `name` (with lands for its cost) with one target per slot, and
/// everything on the stack resolves.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) {
    supported(name);
    cast_new(t, p, name, targets);
    t.resolve_all();
}

/// The permanent (followed across zone changes) loses all abilities until end of turn.
pub fn lose_abilities(t: &mut TestGame, id: ObjectId) {
    modify_until_eot(t, id, vec![Modification::RemoveAllAbilities]);
}

/// Whether the permanent (followed across zone changes) has indestructible now.
pub fn indestructible(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.obj(id).has_keyword(mtg_engine::keywords::KeywordKind::Indestructible)
}

/// Whether the permanent (followed across zone changes) has the keyword now.
pub fn has_kw(t: &mut TestGame, id: ObjectId, kw: mtg_engine::keywords::KeywordKind) -> bool {
    t.g.recompute();
    let id = t.g.current(id);
    t.g.obj(id).has_keyword(kw)
}
