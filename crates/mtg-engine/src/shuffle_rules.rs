//! CR 701.24 Shuffle.
//!
//! * Shuffling specific objects into a library shuffles it even if none of them are where
//!   they're expected to be (CR 701.24c), and shuffling a set of objects shuffles it even
//!   if the set is empty (CR 701.24d): [`shuffle_into`], for `Effect::ShuffleInto` and
//!   `Effect::ShuffleIntoLibrary`.
//! * A library of zero or one cards is still shuffled: shuffle triggers trigger
//!   (CR 701.24e); each of several simultaneous shuffles triggers them (CR 701.24f).
//! * An object put into a certain position of a library at the same time that library is
//!   shuffled ends up in that position of the shuffled library (CR 701.24g).

use crate::ability::LibraryPosition;
use crate::eval::Ctx;
use crate::events::MoveCause;
use crate::game::Game;
use crate::object::Zone;
use crate::replacement::{EtbInfo, MoveEv, ReplEvent};
use crate::types::{ObjectId, PlayerId};

/// Shuffles objects into their owners' libraries (CR 701.24): they're put into those
/// libraries at the same time, then each of those libraries and each of `libraries` is
/// shuffled once — even if some or none of the objects could be put there, or there are
/// none (CR 701.24c, 701.24d). Returns the objects now in a library.
pub fn shuffle_into(
    g: &mut Game,
    objs: &[ObjectId],
    mut libraries: Vec<PlayerId>,
    ctx: &Ctx,
) -> Vec<ObjectId> {
    for o in objs {
        let owner = g.obj(*o).owner;
        if !libraries.contains(&owner) {
            libraries.push(owner);
        }
    }
    let moves: Vec<MoveEv> = objs
        .iter()
        .filter(|o| g.is_live(**o))
        .map(|o| MoveEv {
            obj: *o,
            to: Zone::Library(g.obj(*o).owner),
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(ctx.controller),
            etb: EtbInfo::default(),
            source: ctx.source,
        })
        .collect();
    let moved: Vec<ObjectId> = g.move_objects(moves).into_iter().flatten().collect();
    for p in libraries {
        g.shuffle_library(p);
    }
    moved
        .into_iter()
        .filter(|o| matches!(g.obj(*o).zone, Zone::Library(_)))
        .collect()
}

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
