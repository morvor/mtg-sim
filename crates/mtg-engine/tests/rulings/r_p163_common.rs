//! Shared helpers for the tests of rulings batch P163 (`r_p163_*.rs`): swap removal,
//! sweepers (attack-triggered, graveyard, one-sided), "1/1 synergy" and activated-ability
//! synergy. (The helpers of earlier batches are used too.)

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

pub use crate::r_p108_common::{obj, put_counters, resolved, two_headed_giant};
pub use crate::r_p146_common::{counter, destroy_together, n_asked, stacked_triggers};
pub use crate::r_p160_common::{activate_resolve, cast_resolve, plus_counters, yes};
pub use crate::r_s01_common::{
    attack_with, block_and_finish, creatures, stack_library, supported, tokens,
};
pub use crate::r_s02_common::{can_activate, can_cast, destroy};
pub use crate::r_s05_common::{enter, move_to, run_from};
pub use crate::r_s25_common::{cast_new, lands_for_cost};

/// The damage dealt this turn by `source` (followed by id): (recipient, amount, combat).
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

/// The damage marked on the object now (followed across zone changes).
pub fn marked(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

/// The life-gain events of `p` this turn.
pub fn life_gains(t: &TestGame, p: PlayerId) -> Vec<u32> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::LifeGained { player, amount } if *player == p => Some(*amount),
            _ => None,
        })
        .collect()
}

/// Puts `n` loyalty counters on the planeswalker `pw` (on top of its starting loyalty).
pub fn add_loyalty(t: &mut TestGame, pw: ObjectId, n: u32) {
    put_counters(t, pw, counters::LOYALTY, n);
}

/// The number of times `p`'s library was shuffled this turn.
pub fn shuffles(t: &TestGame, p: PlayerId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
        .count()
}

/// Moves every card in `p`'s hand to the bottom of their library (so tests control
/// exactly what's in hand).
pub fn empty_hand(t: &mut TestGame, p: PlayerId) {
    use mtg_engine::object::Zone;
    for c in t.g.player(p).hand.clone() {
        t.g.players[p.idx()].hand.retain(|x| *x != c);
        t.g.players[p.idx()].library.insert(0, c);
        t.g.objects[c.0 as usize].zone = Zone::Library(p);
    }
}

/// The names of the cards in `p`'s graveyard, bottom first.
pub fn graveyard_names(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.g.player(p)
        .graveyard
        .iter()
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}
