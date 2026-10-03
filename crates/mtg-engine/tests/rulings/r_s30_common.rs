//! Shared helpers for the tests of rulings batch S30 (`r_s30_*.rs`): rulings shared by
//! cards that mention damage and drawing — the monarch and the initiative, combat and
//! noncombat damage, prevention and redirection, lifelink events, replacement order,
//! divided damage, indestructible, and draw triggers and replacements. (The helpers of
//! batches S01–S29 are used too.)

#![allow(dead_code)]

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Moves to `p`'s beginning of combat step (with priority).
pub fn to_combat(t: &mut TestGame, p: PlayerId) {
    t.set_step(p, Step::BeginningOfCombat);
}

/// The damage events this turn: (source, recipient, amount, combat).
pub fn damage_events(t: &TestGame) -> Vec<(ObjectId, Entity, u32, bool)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source,
                target,
                amount,
                combat,
            } => Some((*source, *target, *amount, *combat)),
            _ => None,
        })
        .collect()
}

/// Queues `p`'s answer to the next replacement-order choice (CR 616.1): the option whose
/// text contains `needle`.
pub fn pick_replacement(g: &mtg_engine::game::Game, d: &Decision, needle: &str) -> Option<Answer> {
    let _ = g;
    match d {
        Decision::ChooseReplacement { options } => options
            .iter()
            .position(|o| o.contains(needle))
            .map(Answer::Index),
        _ => None,
    }
}

/// Sets a player's life total directly.
pub fn set_life(t: &mut TestGame, p: PlayerId, life: i32) {
    t.g.players[p.idx()].life = life;
}
