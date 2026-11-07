//! Shared helpers for rulings batch P064 (`r_p064_*.rs`): creatures and permanents that
//! gain flying (or haste) — activated and triggered grants, conditional statics, animated
//! lands and artifacts, counters moved on death. (The helpers of earlier batches are used
//! too.)

#![allow(dead_code)]

use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The object as an entity (a target or a choice).
pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Whether the object (followed across zone changes) has flying now.
pub fn flies(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).has_keyword(KeywordKind::Flying)
}

/// Whether the object (followed across zone changes) has the keyword now.
pub fn has(t: &mut TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.g.recompute();
    t.obj_now(id).has_keyword(kw)
}

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn pool(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Puts `n` counters of `kind` on the object and settles (state-based actions and
/// triggers).
pub fn counters(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    crate::r_s29_common::put_counters(t, id, kind, n);
}

/// Activates the ability of `source` whose text contains `needle` (paying from the pool)
/// and resolves the stack. Panics if the activation is illegal.
pub fn activate_and_resolve(t: &mut TestGame, p: PlayerId, source: ObjectId, needle: &str) {
    crate::r_s06_common::activate_containing(t, p, source, needle)
        .unwrap_or_else(|e| panic!("couldn't activate {needle:?}: {e:?}"));
    t.resolve_all();
}

/// Whether the object (followed across zone changes) has the subtype now.
pub fn is_a(t: &mut TestGame, id: ObjectId, subtype: &str) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.subtypes.iter().any(|s| s == subtype)
}
