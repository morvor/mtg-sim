//! Shared helpers for the tests of rulings batch P084 (`r_p084_*.rs`): graveyard hate
//! (Scryfall Tagger function "hate-graveyard"). The helpers of earlier batches are used
//! too.

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

pub use crate::r_p003_common::{activate, mana, obj, pl};
pub use crate::r_p085_common::tokens_of;
pub use crate::r_s01_common::{supported, tokens};

/// Moves the card (followed to its current object) to exile, as a response that makes a
/// graveyard target illegal would, without settling.
pub fn exile_now(t: &mut TestGame, card: ObjectId) {
    let c = t.g.current(card);
    t.g.move_object(c, Zone::Exile, events::MoveCause::Effect, None);
    t.g.flush_events();
}

/// Moves the card to the top of its owner's library (a response that makes a graveyard
/// target illegal) and returns the new object.
pub fn move_to_library(t: &mut TestGame, card: ObjectId) -> ObjectId {
    let c = t.g.current(card);
    let owner = t.g.obj(c).owner;
    let new = t
        .g
        .move_object(c, Zone::Library(owner), events::MoveCause::Effect, None)
        .expect("moved");
    t.g.flush_events();
    new
}

/// P0 casts the real card `name` from hand with the mana of `cost` in their pool, the
/// given modes (empty: not modal) and targets (one entry per target slot); it stays on
/// the stack.
pub fn cast_modes(
    t: &mut TestGame,
    p: PlayerId,
    name: &str,
    cost: &str,
    modes: &[usize],
    targets: &[&[Entity]],
) -> ObjectId {
    supported(name);
    let c = t.hand(p, name);
    mana(t, p, cost);
    for ts in targets {
        t.answer_targets(p, ts);
    }
    let b = t.cast(p, c);
    let b = if modes.is_empty() { b } else { b.modes(modes) };
    let s = b
        .try_go()
        .unwrap_or_else(|e| panic!("casting {name}: {e:?}"));
    t.settle();
    s
}

/// How many times `p`'s library was shuffled since the turn event with index `from`.
pub fn shuffles_since(t: &TestGame, p: PlayerId, from: usize) -> usize {
    t.g.turn_events[from..]
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == p))
        .count()
}

/// The number of cards in exile owned by `p`.
pub fn exile_count(t: &TestGame, p: PlayerId) -> usize {
    t.g.exile.iter().filter(|&&id| t.g.obj(id).owner == p).count()
}

/// A snapshot of the outcomes a graveyard-hate effect can have: each player's life, P0's
/// hand size, P0's tokens, and the mana in P0's pool.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Outcome {
    pub life: Vec<i32>,
    pub hand: usize,
    pub tokens: usize,
    pub pool: usize,
}

pub fn outcome(t: &TestGame) -> Outcome {
    Outcome {
        life: (0..t.g.players.len())
            .map(|i| t.life(PlayerId(i as u8)))
            .collect(),
        hand: t.hand_size(P0),
        tokens: tokens(t, P0).len(),
        pool: t.g.players[0].mana_pool.total(),
    }
}
