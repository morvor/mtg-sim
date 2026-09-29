//! Shared helpers for the tests of rulings batch S33 (`r_s33_*.rs`): rulings shared by
//! cards about libraries (the top card of a library, {X} in a library, searching and
//! shuffling) and about life (paying life, gaining life and its triggers, life totals
//! that can't change, exchanging and halving life totals, Two-Headed Giant).
//! (The helpers of batches S01–S29 are used too.)

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 and P1 (a team) against P2 and P3 in a Two-Headed Giant game.
pub fn two_headed_giant() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

/// The number of events since index `from` of this turn's events for which `f` holds.
pub fn events_since(t: &TestGame, from: usize, f: impl Fn(&Event) -> bool) -> usize {
    t.g.turn_events[from..].iter().filter(|e| f(e)).count()
}

/// Whether `p`'s library was shuffled since index `from` of this turn's events.
pub fn shuffled_since(t: &TestGame, from: usize, p: PlayerId) -> bool {
    events_since(
        t,
        from,
        |e| matches!(e, Event::Shuffled { player } if *player == p),
    ) > 0
}

/// The life-gain events of `p` since index `from` of this turn's events (the amounts).
pub fn life_gains_since(t: &TestGame, from: usize, p: PlayerId) -> Vec<u32> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::LifeGained { player, amount, .. } if *player == p => Some(*amount),
            _ => None,
        })
        .collect()
}

/// `p`'s library, top card first.
pub fn library_top_first(t: &TestGame, p: PlayerId) -> Vec<ObjectId> {
    t.g.player(p).library.iter().rev().copied().collect()
}

/// Sets `p`'s life total directly (no life is gained or lost).
pub fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    t.g.players[p.idx()].life = life;
}

/// Puts the real card `name` into `p`'s hand with lands for its mana cost, then casts it
/// with all of `targets` in its first target slot ("any number of target ...").
pub fn cast_one_slot(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    crate::r_s25_common::lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    t.cast(p, card).targets(targets).go()
}
