//! Shared helpers for the tests of rulings batch P136 (`r_p136_*.rs`): removal.

#![allow(dead_code)]

use crate::r_s04_common::add_mana;
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Gives `p` four mana of each color and four colorless in their pool.
pub fn plenty(t: &mut TestGame, p: PlayerId) {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        add_mana(t, p, ty, 4);
    }
}

/// `p` casts the real card `name` from hand paying from the mana pool (see [`plenty`]),
/// with one answer per target slot (each a list of targets).
pub fn try_cast(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    slots: &[&[Entity]],
) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    plenty(t, p);
    let c = t.hand(p, name);
    for s in slots {
        t.answer_targets(p, s);
    }
    t.g.turn.priority = Some(p);
    let r = t.g.cast_spell(p, c, CastMethod::Normal);
    t.g.flush_events();
    r
}

/// Like [`try_cast`], panicking if casting fails.
pub fn cast(t: &mut TestGame, p: PlayerId, name: &str, slots: &[&[Entity]]) -> ObjectId {
    try_cast(t, p, name, slots).unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// Returns the permanent to its owner's hand (making a spell's target illegal).
pub fn bounce(t: &mut TestGame, id: ObjectId) -> ObjectId {
    let id = t.g.current(id);
    let owner = t.g.obj(id).owner;
    let new = t
        .g
        .move_object(id, Zone::Hand(owner), MoveCause::Effect, None)
        .expect("bounced");
    t.g.flush_events();
    new
}

/// Empties `p`'s library (the cards cease to exist).
pub fn empty_library(t: &mut TestGame, p: PlayerId) {
    let lib = std::mem::take(&mut t.g.players[p.idx()].library);
    for id in lib {
        t.g.objects[id.0 as usize].zone = Zone::Nowhere;
    }
}
