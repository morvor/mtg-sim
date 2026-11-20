//! Shared helpers for the tests of rulings batch P078 (`r_p078_*.rs`): graveyard seals,
//! group hug and group slug, hand size, cost reductions and attackers. (The helpers of
//! batches S01, S02, S05, S20 and S25 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// A Two-Headed Giant game: P0 and P1 are one team, P2 and P3 the other.
pub fn thg() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

/// Adds `n` mana of each listed type to `p`'s mana pool.
pub fn add_mana(t: &mut TestGame, p: PlayerId, types: &[ManaType], n: u32) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add_type(*ty, n);
    }
}

/// Puts `n` Grizzly Bears into `p`'s hand.
pub fn fill_hand(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        t.hand(p, "Grizzly Bears");
    }
}

/// Advances from wherever the game is to `p`'s next upkeep (its triggers not yet on the
/// stack).
pub fn to_upkeep(t: &mut TestGame, p: PlayerId) {
    let n = t.g.players.len() as u8;
    let before = PlayerId((p.0 + n - 1) % n);
    t.set_step(before, Step::End);
    t.advance_to(p, Step::Upkeep);
}

/// The players asked a decision matching `pred` since decision `from`, in order.
pub fn deciders_since(t: &TestGame, from: usize, pred: fn(&Decision) -> bool) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| pred(d))
        .map(|(p, _)| *p)
        .collect()
}

/// The values a [`crate::r_s01_common::watch`] recorded.
pub fn seen_values<T: Clone>(seen: &crate::r_s01_common::Seen<T>) -> Vec<T> {
    seen.lock().unwrap().clone()
}
