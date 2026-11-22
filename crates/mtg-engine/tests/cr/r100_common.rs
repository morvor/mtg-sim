//! Shared helpers for the CR 100–104 tests (general rules, golden rules, players,
//! starting and ending the game).

#![allow(dead_code)]

use mtg_engine::card::{card, CardDef};
use mtg_engine::game::{Game, GameConfig};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// A game that hasn't started yet (CR 103): each player's deck is put into their library
/// in order; call `t.g.start()` to begin. Decisions are scripted as with `TestGame`.
pub fn pregame(config: GameConfig, decks: Vec<Vec<Arc<CardDef>>>) -> TestGame {
    let n = decks.len();
    let script = Arc::new(Mutex::new(Script {
        queues: vec![VecDeque::new(); n],
        asked: vec![],
    }));
    let agents: Vec<Box<dyn Agent>> = (0..n)
        .map(|i| {
            Box::new(ScriptedAgent {
                player: PlayerId(i as u8),
                script: script.clone(),
            }) as Box<dyn Agent>
        })
        .collect();
    let mut g = Game::new(config, decks, agents);
    g.logging = true;
    TestGame { g, script }
}

/// `n` copies of a real card.
pub fn copies(name: &str, n: usize) -> Vec<Arc<CardDef>> {
    (0..n).map(|_| card(name)).collect()
}

/// A deck of `n` filler cards.
pub fn fillers(n: usize) -> Vec<Arc<CardDef>> {
    (0..n).map(|_| filler_card()).collect()
}

/// A test game with `n` players whose teams are given by `teams` (player index → team).
pub fn team_game(teams: &[u8], config: GameConfig) -> TestGame {
    TestGame::with_config(
        teams.len(),
        GameConfig {
            teams: Some(teams.to_vec()),
            ..config
        },
    )
}

/// Passes priority until the beginning of `step` of `active`'s next turn (triggers of that
/// step are on the stack).
pub fn go_to(t: &mut TestGame, active: PlayerId, step: Step) {
    t.advance_to(active, step);
}

/// Names of the cards in a zone list.
pub fn names(t: &TestGame, ids: &[ObjectId]) -> Vec<String> {
    ids.iter()
        .map(|i| t.g.obj(*i).chars.name.to_string())
        .collect()
}
