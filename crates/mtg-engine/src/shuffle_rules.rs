//! CR 701.24 Shuffle.
//!
//! * Shuffling specific objects into a library shuffles it even if none of them are where
//!   they're expected to be (CR 701.24c), and shuffling a set of objects shuffles it even
//!   if the set is empty (CR 701.24d). See `Effect::ShuffleInto` and
//!   `Effect::ShuffleIntoLibrary`.
//! * A library of zero or one cards is still shuffled: shuffle triggers trigger
//!   (CR 701.24e); each of several simultaneous shuffles triggers them (CR 701.24f).
//! * An object put into a certain position of a library at the same time that library is
//!   shuffled ends up in that position of the shuffled library (CR 701.24g).

use crate::ability::LibraryPosition;
use crate::game::Game;
use crate::object::Zone;
use crate::replacement::ReplEvent;
use crate::types::PlayerId;

/// CR 701.24g: of the (replaced) moves of a simultaneous zone change, those that put an
/// object into a position of a library that's shuffled by another of them are performed
/// after the shuffles, so the result is a shuffled library with the object in that
/// position. The order of the other moves is kept.
pub fn positions_after_shuffles(_g: &Game, finals: &mut [(usize, ReplEvent)]) {
    let shuffled: Vec<PlayerId> = finals
        .iter()
        .filter_map(|(_, e)| match e {
            ReplEvent::Move(m) if matches!(m.pos, LibraryPosition::Shuffled) => match m.to {
                Zone::Library(p) => Some(p),
                _ => None,
            },
            _ => None,
        })
        .collect();
    if shuffled.is_empty() {
        return;
    }
    // A stable sort: positioned moves into those libraries go last, in their order.
    finals.sort_by_key(|(_, e)| match e {
        ReplEvent::Move(m) if !matches!(m.pos, LibraryPosition::Shuffled) => {
            matches!(m.to, Zone::Library(p) if shuffled.contains(&p))
        }
        _ => false,
    });
}
