//! Rule-modifying static abilities that each change one game rule:
//!
//! * [`cant_be_copied`]: "This spell can't be copied", "This ability can't be copied"
//!   (CR 113.6g, 707.10).
//! * [`cleanup_damage`]: "Damage isn't removed from [permanents] during cleanup steps"
//!   (an exception to CR 514.2).
//! * [`counters_remain`]: "Counters remain on ~ as it moves to any zone other than a
//!   player's hand or library" (an exception to CR 122.2 and 400.7).
//! * [`face_up`]: "[permanents] can't be turned face up" (CR 708.7).
//! * [`turns_taken`]: "your first, second, or third turn of the game".

pub mod cant_be_copied;
pub mod cleanup_damage;
pub mod counters_remain;
pub mod face_up;
pub mod turns_taken;

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
