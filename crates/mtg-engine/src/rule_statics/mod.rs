//! Rule-modifying static abilities that each change one game rule:
//!
//! * [`cleanup_damage`]: "Damage isn't removed from [permanents] during cleanup steps"
//!   (an exception to CR 514.2).
//! * [`counters_remain`]: "Counters remain on ~ as it moves to any zone other than a
//!   player's hand or library" (an exception to CR 122.2 and 400.7).

pub mod cleanup_damage;
pub mod counters_remain;

use crate::ability::*;
use crate::game::Game;
use crate::types::{ObjectId, PlayerId};

/// The active static abilities (CR 604) with effects `pick` selects, with their sources
/// and controllers.
pub(crate) fn active_others<'a, T>(
    g: &'a Game,
    pick: impl Fn(&'a StaticEffect) -> Option<T> + 'a,
) -> impl Iterator<Item = (ObjectId, PlayerId, T)> + 'a {
    g.statics
        .other
        .iter()
        .filter_map(move |(s, c, e)| pick(e).map(|t| (*s, *c, t)))
}
