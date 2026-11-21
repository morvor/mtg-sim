//! Shared helpers for the tests of rulings batch P042 (`r_p042_*.rs`), the second half of
//! the "damage prevention" group: prevention and redirection shields (CR 615, 614.9),
//! life totals that can't go below 1, and the order of prevention effects (CR 616.1).
//! The helpers of P041 (`r_p041_common`) are reused.

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p041_common::*;

/// The prevention events of this turn: (source, recipient, amount prevented).
pub fn prevented_events(t: &TestGame) -> Vec<(ObjectId, Entity, u32)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::DamagePrevented {
                source,
                target,
                amount,
                ..
            } => Some((*source, *target, *amount)),
            _ => None,
        })
        .collect()
}

/// Whether any combat damage event of this turn was dealt by `source` to `to`.
pub fn combat_damage(t: &TestGame, source: ObjectId, to: Entity) -> u32 {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source: s,
                target,
                amount,
                combat: true,
            } if *s == source && *target == to => Some(*amount),
            _ => None,
        })
        .sum()
}

/// The (combat flag of the) damage events dealt by `source` this turn.
pub fn damage_by(t: &TestGame, source: ObjectId) -> Vec<(Entity, u32, bool)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source: s,
                target,
                amount,
                combat,
            } if *s == source => Some((*target, *amount, *combat)),
            _ => None,
        })
        .collect()
}

/// The life gained by `p` this turn.
pub fn life_gained(t: &TestGame, p: PlayerId) -> u32 {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::LifeGained { player, amount } if *player == p => Some(*amount),
            _ => None,
        })
        .sum()
}

/// Sets `p`'s life total (no life gain or loss events).
pub fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    t.g.players[p.idx()].life = life;
}
