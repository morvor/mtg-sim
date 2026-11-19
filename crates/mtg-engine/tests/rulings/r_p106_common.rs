//! Shared helpers for the tests of rulings batch P106 (`r_p106_*.rs`): mana rocks and
//! mana-producing abilities with a second effect ("mini refund"), what their mana does
//! when spent, and the other abilities of the same cards. (The helpers of earlier batches
//! are used too.)

#![allow(dead_code)]

use mtg_engine::ability::AbilityKind;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p107_common::{
    act, cast_from_hand, hand_names, is_creature, mana, obj, pool_total, put_counters,
    resolved,
};
pub use crate::r_s01_common::{stack_library, tokens};
pub use crate::r_s02_common::destroy;
pub use crate::r_s06_common::activate_containing;
pub use crate::r_s20_common::tap_for_mana;
pub use crate::r_s24_common::enter_together;
pub use crate::r_s27_common::can_activate_containing;

pub const BIRGI: &str = "Birgi, God of Storytelling // Harnfel, Horn of Bounty";

/// Whether the ability of `source` whose text contains `needle` is an activated mana
/// ability (CR 605.1a).
pub fn is_mana_ability(t: &mut TestGame, source: ObjectId, needle: &str) -> bool {
    t.g.recompute();
    let s = t.g.current(source);
    t.g.obj(s)
        .chars
        .abilities
        .iter()
        .find(|a| a.text.contains(needle))
        .map(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
        .unwrap_or_else(|| panic!("no ability containing {needle:?}"))
}

/// The number of events logged this turn so far (a mark for [`events_since`]).
pub fn mark(t: &TestGame) -> usize {
    t.g.turn_events.len()
}

/// The events logged this turn since `mark`.
pub fn events_since(t: &TestGame, mark: usize) -> Vec<Event> {
    t.g.turn_events[mark..].to_vec()
}

/// The total life `p` lost in the events since `mark`.
pub fn life_lost_since(t: &TestGame, mark: usize, p: PlayerId) -> u32 {
    events_since(t, mark)
        .iter()
        .filter_map(|e| match e {
            Event::LifeLost { player, amount } if *player == p => Some(*amount),
            _ => None,
        })
        .sum()
}

/// The total life `p` gained in the events since `mark`.
pub fn life_gained_since(t: &TestGame, mark: usize, p: PlayerId) -> u32 {
    events_since(t, mark)
        .iter()
        .filter_map(|e| match e {
            Event::LifeGained { player, amount } if *player == p => Some(*amount),
            _ => None,
        })
        .sum()
}

/// The permanents named `name` that `p` controls.
pub fn controlled(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .map(|o| o.id)
        .collect()
}
