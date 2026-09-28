//! Shared helpers for the tests of rulings batch S13 (`r_s13_*.rs`): partner with,
//! persist, plot, populate, prepared, proliferate, protection. (The helpers of batches
//! S01–S11 are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `n` counters of `kind` on a permanent or player (as an effect would).
pub fn add(t: &mut TestGame, e: impl Into<Entity>, kind: &str, n: u32) {
    t.g.add_counters(e.into(), kind, n, None);
    t.g.flush_events();
}

/// A game that hasn't started yet (no opening hands), with the given decks.
pub fn pregame(
    config: mtg_engine::game::GameConfig,
    decks: Vec<Vec<std::sync::Arc<mtg_engine::card::CardDef>>>,
) -> TestGame {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    let n = decks.len();
    let script = Arc::new(Mutex::new(Script {
        queues: vec![VecDeque::new(); n],
        asked: vec![],
    }));
    let agents: Vec<Box<dyn mtg_engine::decision::Agent>> = (0..n)
        .map(|i| {
            Box::new(ScriptedAgent {
                player: PlayerId(i as u8),
                script: script.clone(),
            }) as Box<dyn mtg_engine::decision::Agent>
        })
        .collect();
    let mut g = mtg_engine::game::Game::new(config, decks, agents);
    g.logging = true;
    TestGame { g, script }
}

/// A two-player Commander game in progress (player 0's first main phase).
pub fn commander_game() -> TestGame {
    TestGame::with_config(
        2,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..Default::default()
        },
    )
}

/// Puts the real card `name` into `p`'s command zone as one of their commanders.
pub fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.command(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.players[p.idx()].commander_names.push(name.into());
    id
}
