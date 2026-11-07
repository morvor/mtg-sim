//! Shared helpers for the tests of rulings batch P066 (`r_p066_*.rs`): creatures that gain
//! menace, trample, vigilance, protection or -1/-1 counters, "Giant Growth" spells, and
//! cards that let you cast cards from exile.

#![allow(dead_code)]

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

pub fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The creatures blocking `attacker` (followed across zone changes) now.
pub fn blockers_of(t: &TestGame, attacker: ObjectId) -> Vec<ObjectId> {
    let a = t.g.current(attacker);
    t.g.combat
        .as_ref()
        .map(|c| c.blockers_of(a))
        .unwrap_or_default()
}

/// Whether `attacker` is a blocked creature (CR 509.1h).
pub fn is_blocked(t: &TestGame, attacker: ObjectId) -> bool {
    let a = t.g.current(attacker);
    t.g.combat
        .as_ref()
        .and_then(|c| c.attackers.iter().find(|x| x.id == a))
        .is_some_and(|x| x.blocked)
}

/// Whether the object (followed across zone changes) is tapped.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}
