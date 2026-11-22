//! Shared helpers for the tests of rulings batch P014 (`r_p014_*.rs`): "catch up"
//! effects that compare players (life totals, lands, creatures, permanents), colorless
//! and all-colors characteristic-defining abilities with the Eldrazi around them, and
//! power/toughness-defining abilities. (The helpers of batches S01 and P038 are used too.)

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::*;

pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

pub fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Sets `p`'s life total directly (test setup, no life gain or loss happens).
pub fn set_life(t: &mut TestGame, p: PlayerId, n: i32) {
    t.g.players[p.idx()].life = n;
}

/// A Two-Headed Giant game: P0 and P1 are one team, P2 and P3 the other.
pub fn thg() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

/// The events of this turn from index `from` of `turn_events`.
pub fn events_since(t: &TestGame, from: usize) -> Vec<Event> {
    t.g.turn_events[from..].to_vec()
}

/// The life gained and lost (as `(player, +n or -n)`) since `from`.
pub fn life_changes(t: &TestGame, from: usize) -> Vec<(PlayerId, i64)> {
    events_since(t, from)
        .into_iter()
        .filter_map(|e| match e {
            Event::LifeGained { player, amount } => Some((player, amount as i64)),
            Event::LifeLost { player, amount } => Some((player, -(amount as i64))),
            _ => None,
        })
        .collect()
}

/// Every player's life total.
pub fn lives(t: &TestGame) -> Vec<i32> {
    t.g.players.iter().map(|p| p.life).collect()
}

/// Whether `id` (followed to its current object) is face down.
pub fn face_down(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).face_down
}
