//! Shared helpers for the tests of rulings batch P107 (`r_p107_*.rs`): mana rocks with a
//! second ability ("mini refund"), Bobbleheads, minigames, modified creatures and mixed
//! subtypes. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p108_common::{put_counters, resolved, tokens_with};
pub use crate::r_s01_common::{creatures, stack_library, supported, tokens, with_subtype};
pub use crate::r_s02_common::destroy;
pub use crate::r_s06_common::{activate_containing, attach_new, has_kw};
pub use crate::r_s27_common::can_activate_containing;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Adds `n` mana of type `ty` to `p`'s mana pool.
pub fn mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

/// Total mana in `p`'s pool.
pub fn pool_total(t: &TestGame, p: PlayerId) -> u32 {
    t.g.players[p.idx()].mana_pool.total() as u32
}

/// Activates the ability of `source` containing `needle` with `targets` (one per slot),
/// paying from the mana pool.
pub fn act(
    t: &mut TestGame,
    p: PlayerId,
    source: ObjectId,
    needle: &str,
    targets: &[Entity],
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    activate_containing(t, p, source, needle)
}

/// Casts the real card `name` from `p`'s hand with mana from the pool.
pub fn cast_from_hand(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let c = t.hand(p, name);
    t.cast_with(p, c, targets).expect("cast failed")
}

/// Whether the object (followed across zone changes) is a creature now.
pub fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

/// Names of the cards in `p`'s hand.
pub fn hand_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .hand
        .iter()
        .map(|id| t.g.obj(*id).chars.name.to_string())
        .collect()
}
