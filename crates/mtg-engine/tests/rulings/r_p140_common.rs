//! Shared helpers for the tests of rulings batch P140 (`r_p140_*.rs`): putting permanents
//! and spells into libraries ("tuck"), removing counters, removing things from the stack
//! and ending the turn, abilities that remove flying, hexproof and indestructible, renew,
//! and repeatable artifact tokens (token copies, Thopter makers, artifact-cast triggers).

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p016_common::lots;
pub use crate::r_p108_common::{obj, put_counters};
pub use crate::r_p117_common::modify;
pub use crate::r_p119_common::counter_spell;
pub use crate::r_p177_common::{concede, ffa, set_loyalty};
pub use crate::r_s01_common::{asked_since, stack_library, supported, triggers_on_stack};
pub use crate::r_s02_common::{can_activate, can_cast, destroy, target_candidates};
pub use crate::r_s05_common::move_to;
pub use crate::r_s06_common::{activate_containing, give_control};
pub use crate::r_s17_common::{become_copy, token_copy};
pub use crate::r_s25_common::cast_new;

/// The players whose libraries were shuffled this turn, in order.
pub fn shuffles(t: &TestGame) -> Vec<PlayerId> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Shuffled { player } => Some(*player),
            _ => None,
        })
        .collect()
}

/// Whether anything was countered this turn.
pub fn anything_countered(t: &TestGame) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::Countered { .. }))
}

/// The position of the card (followed across zone changes) in `p`'s library, counted from
/// the top (0 is the top card).
pub fn from_top(t: &TestGame, p: PlayerId, id: ObjectId) -> Option<usize> {
    let id = t.g.current(id);
    t.g.player(p).library.iter().rev().position(|c| *c == id)
}

/// Empties `p`'s library.
pub fn empty_library(t: &mut TestGame, p: PlayerId) {
    t.g.players[p.idx()].library.clear();
}

/// Leaves `n` filler cards in `p`'s library.
pub fn library_of(t: &mut TestGame, p: PlayerId, n: usize) {
    let lib = &mut t.g.players[p.idx()].library;
    let keep = lib.len().min(n);
    lib.drain(..lib.len() - keep);
}

/// The zone of the card (followed across zone changes).
pub fn zone_of(t: &TestGame, id: ObjectId) -> Zone {
    t.zone(t.g.current(id))
}

/// The permanents named `name` that `p` controls.
pub fn controlled_named(t: &TestGame, p: PlayerId, name: &str) -> Vec<ObjectId> {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj(*id).controller == p)
        .collect()
}

/// The tokens `p` controls.
pub fn tokens_of(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    crate::r_s01_common::tokens(t, p)
}

/// The tokens `p` controls with creature subtype `sub` (e.g. "Thopter").
pub fn tokens_typed(t: &TestGame, p: PlayerId, sub: &str) -> Vec<ObjectId> {
    tokens_of(t, p)
        .into_iter()
        .filter(|id| t.obj(*id).chars.has_subtype(sub))
        .collect()
}

/// `p` casts the real card `name` from hand with plenty of mana and the given targets
/// (one per slot); returns the spell.
pub fn cast_lots(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    supported(name);
    lots(t, p);
    let card = t.hand(p, name);
    t.cast_with(p, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// `p` activates the `index`th activated ability of `src` with plenty of mana.
pub fn activate_lots(
    t: &mut TestGame,
    p: PlayerId,
    src: ObjectId,
    index: usize,
    targets: &[Entity],
) -> Option<ObjectId> {
    lots(t, p);
    let src = t.g.current(src);
    t.activate(p, src, index, targets)
        .unwrap_or_else(|e| panic!("activation failed: {e:?}"))
}
