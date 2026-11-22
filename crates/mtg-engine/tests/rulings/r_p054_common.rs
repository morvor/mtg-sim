//! Shared helpers for the tests of rulings batch P054 (`r_p054_*.rs`): pump spells and
//! abilities ("enlarge"), "is dealt damage" triggers (enrage), "enters in company" modal
//! creatures, eponymous abilities, Aura counts, "attacks alone" (exalted) and exhaust.

#![allow(dead_code)]

use crate::r_s01_common::supported;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Fills `p`'s mana pool with 12 mana of each type.
pub fn pool(t: &mut TestGame, p: PlayerId) {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        t.g.players[p.idx()].mana_pool.add_type(ty, 12);
    }
}

/// `p` casts the real (fully supported) card `name` from hand, paying from a full mana
/// pool, with one target answer per entry of `targets`; the spell stays on the stack.
pub fn cast_how(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    method: CastMethod,
    targets: &[&[Entity]],
    x: Option<i64>,
    modes: Option<&[usize]>,
) -> ObjectId {
    supported(name);
    pool(t, p);
    let c = t.hand(p, name);
    for ts in targets {
        t.answer_targets(p, ts);
    }
    let mut b = t.cast(p, c).method(method);
    if let Some(x) = x {
        b = b.x(x);
    }
    if let Some(m) = modes {
        b = b.modes(m);
    }
    b.go()
}

/// `p` casts `name` normally with a single target.
pub fn cast_at(t: &mut TestGame, p: PlayerId, name: &str, target: ObjectId) -> ObjectId {
    cast_how(t, p, name, CastMethod::Normal, &[&[o(target)]], None, None)
}

/// Moves `id` (its current object) to `zone` as an effect would, then settles.
pub fn move_to(t: &mut TestGame, id: ObjectId, zone: Zone) {
    let id = t.g.current(id);
    t.g.move_object(id, zone, mtg_engine::events::MoveCause::Effect, None);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
}

/// Whether the spell or ability `id` is still on the stack.
pub fn on_stack(t: &TestGame, id: ObjectId) -> bool {
    t.g.stack.contains(&id)
}

/// The real card `name` enters under `p`'s control through a real zone change; its
/// triggers are put on the stack.
pub fn enter(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.enter(p, name);
    t.g.recompute();
    t.g.flush_events();
    t.settle();
    id
}
