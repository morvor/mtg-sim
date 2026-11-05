//! Shared helpers for rulings batch P012 (`r_p012_*.rs`): "cards in graveyard matter" —
//! counts of graveyard cards, abilities that trigger on casting spells, spell copies,
//! casting without paying the mana cost and cost changes. (The helpers of batches S01–S30
//! are used too.)

#![allow(dead_code)]

use crate::r_s01_common::supported;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// `p` casts the real card `name` from hand (lands for its mana cost added) with the given
/// targets (one per target slot); the triggers it causes are put on the stack. Returns the
/// spell.
pub fn cast_card(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    let def = mtg_engine::card::card(name);
    if !def.front().chars.card_types.contains(CardType::Instant) && t.g.turn.active != p {
        t.set_step(p, mtg_engine::turn::Step::PrecombatMain);
    }
    lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    let spell = t
        .cast_with(p, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"));
    t.settle();
    spell
}

/// Whether the topmost stack object is a triggered ability.
pub fn trigger_on_top(t: &TestGame) -> bool {
    t.g.stack.last().is_some_and(|id| {
        matches!(
            t.g.obj(*id).stack.as_deref().map(|s| &s.kind),
            Some(StackKind::Triggered { .. })
        )
    })
}

/// Puts `n` copies of the real card `name` into `p`'s graveyard.
pub fn fill_graveyard(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, name)).collect()
}

/// Moves the card `id` from its zone to `p`'s library (removing it from the graveyard).
pub fn to_library(t: &mut TestGame, id: ObjectId) {
    let owner = t.obj_now(id).owner;
    crate::r_p127_common::move_to(t, id, mtg_engine::object::Zone::Library(owner));
    t.g.recompute();
}
