//! Shared helpers for the tests of the rulings residue item `rulings-residue-enchant-equip`
//! (`r_residue_enchant_equip_*.rs`): the open rulings left in batches P207–P213 (Auras and
//! Equipment). The helpers of the S and P batches are used too.

#![allow(dead_code)]

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// An entity for an object.
pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The name of the object (followed across zone changes) now.
pub fn name(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

/// Casts Metamorphic Alteration (with lands for it) for `p` on `host`, choosing `chosen`,
/// and resolves everything. The Aura, followed to its new zone.
pub fn alter(t: &mut TestGame, p: PlayerId, host: ObjectId, chosen: ObjectId) -> ObjectId {
    t.lands(p, "Island", 2);
    let aura = t.hand(p, "Metamorphic Alteration");
    t.answer_choose(p, &[Entity::Object(chosen)]);
    t.g.turn.priority = Some(p);
    t.cast(p, aura).target(host).go();
    t.resolve_all();
    t.g.current(aura)
}

/// Puts the real Aura `name` onto the battlefield under `p`'s control attached to `to`
/// (an object or a player) by setting its attachment directly, as if it had been there
/// all along (no enters abilities, no legality check), and returns it.
pub fn aura_on(t: &mut TestGame, p: PlayerId, name: &str, to: impl Into<Entity>) -> ObjectId {
    let id = t.battlefield(p, name);
    t.g.obj_mut(id).attached_to = Some(to.into());
    t.g.recompute();
    id
}

/// Whether the object (followed across zone changes) is now in `zone`.
pub fn in_zone(t: &TestGame, id: ObjectId, zone: Zone) -> bool {
    t.zone(t.g.current(id)) == zone
}
