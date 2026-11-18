//! Shared helpers for the tests of rulings batch P135 (`r_p135_*.rs`): fights whose
//! targets or sources go away, fight spells that need both targets, and land destruction
//! (targeted, mass, and "each player sacrifices"). (The helpers of earlier batches are
//! used too.)

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p113_common::{cast_modes, spoil, Spoil};
pub use crate::r_p133_common::{fizzles, kill};
pub use crate::r_p134_common::*;
pub use crate::r_s01_common::{attack_with, custom_card, stack_library, supported};
pub use crate::r_s02_common::{destroy, target_candidates};
pub use crate::r_s05_common::run_from;
pub use crate::r_s06_common::{attach_new, attached_to, give_control};
pub use crate::r_s07_common::{damage_on, resolved};
pub use crate::r_s25_common::lands_for_cost;

/// The total damage dealt to `to` this turn (from the turn's damage events).
pub fn dealt(t: &TestGame, to: impl Into<Entity>) -> u32 {
    let to = to.into();
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage { target, amount, .. } if *target == to => Some(*amount),
            _ => None,
        })
        .sum()
}

/// Tries to cast the real card `name` (lands for its mana cost are added) with one
/// target per slot (an empty slot answers "no target") and the given modes; the queued
/// answers are cleared afterwards. Returns the spell or the reason it's illegal.
pub fn try_cast(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    modes: &[usize],
    targets: &[&[Entity]],
) -> Result<ObjectId, mtg_engine::casting::Illegal> {
    supported(name);
    lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, e);
    }
    let mut b = t.cast(p, card);
    if !modes.is_empty() {
        b = b.modes(modes);
    }
    let r = b.try_go();
    t.clear_answers();
    r
}

/// The number of permanents named `name` on the battlefield.
pub fn count_named(t: &TestGame, name: &str) -> usize {
    t.named_on_battlefield(name).len()
}

/// The permanents `p` controls with the given card type.
pub fn controlled(t: &TestGame, p: PlayerId, ty: mtg_engine::types::CardType) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(ty))
        .count()
}
