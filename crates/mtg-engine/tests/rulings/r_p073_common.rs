//! Shared helpers for the tests of rulings batch P073 (`r_p073_*.rs`): cards that give
//! flying, haste, hexproof, indestructible or lifelink. (The helpers of earlier batches
//! are used too.)

#![allow(dead_code)]

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

pub const INDESTRUCTIBLE: KeywordKind = KeywordKind::Indestructible;
pub const HEXPROOF: KeywordKind = KeywordKind::Hexproof;
pub const HASTE: KeywordKind = KeywordKind::Haste;
pub const LIFELINK: KeywordKind = KeywordKind::Lifelink;
pub const FLYING: KeywordKind = KeywordKind::Flying;

/// Whether the object (followed across zone changes) has the keyword now.
pub fn kw(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(k)
}

/// `p` casts the real card `name` from hand (lands for its cost are added) with one
/// target per slot, and everything on the stack resolves.
pub fn cast_resolve(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let id = crate::r_p079_common::cast_card(t, p, name, targets).expect("cast");
    t.resolve_all();
    id
}

/// Destroys the permanent (as a resolving "destroy" effect would) and settles. Whether
/// it left the battlefield.
pub fn destroy_now(t: &mut TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.destroy(id, None);
    t.g.flush_events();
    t.settle();
    !t.on_battlefield(id)
}

/// Whether `p` may choose `target` for the spell `name` (cast from hand, with mana):
/// the spell's first target choice is inspected and casting is abandoned.
pub fn can_target_with(t: &mut TestGame, p: PlayerId, name: &str, target: Entity) -> bool {
    crate::r_s04_common::spell_targets(t, p, name).contains(&target)
}

/// Puts the real card `name` onto the battlefield as `p`'s commander.
pub fn bf_commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.dirty = true;
    t.settle();
    id
}
