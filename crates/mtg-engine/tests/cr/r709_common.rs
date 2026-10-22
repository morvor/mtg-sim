//! Shared helpers for the CR 709–722 tests (special cards: split, flip, leveler,
//! double-faced, saga, adventurer, class, attraction, prototype, case, omen, station and
//! preparation cards).

#![allow(dead_code)]

pub use super::r200_common::{can_cast_face, mv, name_card};
pub use super::r600_common::{stat, CB};
pub use super::r703_common::{bear, oracle_card, supported};
use mtg_engine::ability::*;
use mtg_engine::eval::Ctx;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Performs an effect controlled by `p` (with `source` as its source and `targets` in its
/// first target slot), then lets triggered abilities trigger.
pub fn run_effect(
    t: &mut TestGame,
    p: PlayerId,
    source: Option<ObjectId>,
    targets: &[Entity],
    effect: Effect,
) {
    let mut ctx = Ctx::new(source, p);
    ctx.targets = vec![targets.to_vec()];
    t.g.exec(&effect, &mut ctx);
    t.g.recompute();
    t.g.flush_events();
}

/// The single permanent named `name` on the battlefield.
pub fn the(t: &TestGame, name: &str) -> ObjectId {
    let v = t.named_on_battlefield(name);
    assert_eq!(v.len(), 1, "expected exactly one {name} on the battlefield");
    v[0]
}

/// Casts one half (or the Adventure, Omen, ... part) of a card: `CastMethod::Half(i)`.
pub fn cast_half(t: &mut TestGame, p: PlayerId, card: ObjectId, half: u8) -> ObjectId {
    t.cast(p, card).method(CastMethod::Half(half)).go()
}

/// Whether the object is (still) in `zone`.
pub fn in_zone(t: &TestGame, id: ObjectId, zone: Zone) -> bool {
    let now = t.g.current(id);
    t.g.is_live(now) && t.obj(now).zone == zone
}
