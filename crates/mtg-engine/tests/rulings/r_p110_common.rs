//! Shared helpers for the tests of rulings batch P110 (`r_p110_*.rs`): "each player
//! sacrifices" effects, naturalize-like spells with a rider, power-5 ("ferocious") cards,
//! name-a-card prohibitions, and triggered or loyalty abilities that add mana. (The helpers
//! of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub use crate::r_p107_common::{act, cast_from_hand, mana, pool_total};
pub use crate::r_p108_common::{put_counters, resolved};
pub use crate::r_s01_common::{creatures, stack_library, supported, tokens, watch};
pub use crate::r_s02_common::{can_activate, can_cast, can_play_land, create_token, destroy};
pub use crate::r_s06_common::{activate_containing, damage, has_kw};
pub use crate::r_s19_common::add_lore;
pub use crate::r_s25_common::{abilities_from, cast_new, lands_for_cost, spell_copies};

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The players asked to choose permanents to sacrifice since decision `from`, in order.
pub fn sacrifice_choosers(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { prompt, .. } if prompt.contains("sacrifice") => Some(*p),
            _ => None,
        })
        .collect()
}

/// The number of creatures on the battlefield.
pub fn creatures_on_battlefield(g: &mtg_engine::game::Game) -> usize {
    g.permanents().filter(|o| o.is(CardType::Creature)).count()
}

/// The amount of mana of type `ty` in `p`'s mana pool.
pub fn pool_of(t: &TestGame, p: PlayerId, ty: ManaType) -> u32 {
    t.g.player(p).mana_pool.count(ty) as u32
}

/// Puts `n` Grizzly Bears onto the battlefield under `p`'s control.
pub fn bears(t: &mut TestGame, p: PlayerId, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.battlefield(p, "Grizzly Bears")).collect()
}

/// Whether the stack holds an ability (not a spell) whose source is `source`.
pub fn ability_on_stack(t: &TestGame, source: ObjectId) -> bool {
    !abilities_from(t, source).is_empty()
}
