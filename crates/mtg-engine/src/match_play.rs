//! Matches (CR 100.6a): a series of games between the same players. A two-player match
//! is played until one player has won two games; a multiplayer match is a single game.
//! Between games, players may modify their decks with their sideboards (CR 100.4), and the
//! loser of the previous game chooses who takes the first turn (CR 103.1).

use crate::card::CardDef;
use crate::decision::Agent;
use crate::game::{Game, GameConfig, GameResult};
use crate::types::*;
use std::sync::Arc;

/// A match in progress.
pub struct Match {
    pub config: GameConfig,
    pub decks: Vec<Vec<Arc<CardDef>>>,
    pub sideboards: Vec<Vec<Arc<CardDef>>>,
    /// Games a player must win to win the match.
    pub games_to_win: u32,
    /// Results of the games played so far.
    pub results: Vec<GameResult>,
    /// Games won by each player.
    pub wins: Vec<u32>,
    /// Who chose the starting player in each game played.
    pub choosers: Vec<Option<PlayerId>>,
    /// The starting player of each game played.
    pub starting_players: Vec<PlayerId>,
}

impl Match {
    /// A new match: best of three for two players, a single game otherwise (CR 100.6a).
    pub fn new(
        config: GameConfig,
        decks: Vec<Vec<Arc<CardDef>>>,
        sideboards: Vec<Vec<Arc<CardDef>>>,
    ) -> Match {
        let n = decks.len();
        Match {
            config,
            decks,
            sideboards,
            games_to_win: if n == 2 { 2 } else { 1 },
            results: vec![],
            wins: vec![0; n],
            choosers: vec![],
            starting_players: vec![],
        }
    }

    /// The match winner, if any.
    pub fn winner(&self) -> Option<PlayerId> {
        self.wins
            .iter()
            .position(|w| *w >= self.games_to_win)
            .map(|i| PlayerId(i as u8))
    }

    /// Whether the match is over: someone has won enough games, or a multiplayer match's
    /// single game has been played.
    pub fn is_over(&self) -> bool {
        self.winner().is_some() || (self.decks.len() > 2 && !self.results.is_empty())
    }

    /// Between games, a player swaps a card in their deck for one in their sideboard
    /// (CR 100.4). Returns false if either card isn't there.
    pub fn sideboard(&mut self, p: PlayerId, out: &str, into: &str) -> bool {
        let deck = &mut self.decks[p.idx()];
        let side = &mut self.sideboards[p.idx()];
        let (Some(i), Some(j)) = (
            deck.iter().position(|c| c.name == out),
            side.iter().position(|c| c.name == into),
        ) else {
            return false;
        };
        let a = deck.remove(i);
        let b = side.remove(j);
        deck.push(b);
        side.push(a);
        true
    }

    /// Who chooses the starting player of the next game (CR 103.1): nobody in particular
    /// for the first game (the players determine it); afterwards the loser of the
    /// previous game, or after a draw the player who chose in that game.
    pub fn next_chooser(&self) -> Option<PlayerId> {
        let last = self.results.last()?;
        match last {
            GameResult::Win(ws) => (0..self.decks.len())
                .map(|i| PlayerId(i as u8))
                .find(|p| !ws.contains(p)),
            GameResult::Draw => self.choosers.last().copied().flatten(),
        }
    }

    /// Creates the next game of the match (not yet started).
    pub fn next_game(&self, agents: Vec<Box<dyn Agent>>, seed: u64) -> Game {
        let config = GameConfig {
            first_turn_chooser: self.next_chooser(),
            seed,
            ..self.config.clone()
        };
        let mut g = Game::new(config, self.decks.clone(), agents);
        for (i, side) in self.sideboards.iter().enumerate() {
            g.add_to_sideboard(PlayerId(i as u8), side.clone());
        }
        g
    }

    /// Records a finished game.
    pub fn record(&mut self, g: &Game) {
        let Some(result) = g.result.clone() else {
            return;
        };
        if let GameResult::Win(ws) = &result {
            for w in ws {
                self.wins[w.idx()] += 1;
            }
        }
        self.choosers.push(g.start.chooser);
        self.starting_players.push(g.turn.starting_player);
        self.results.push(result);
    }

    /// Plays the next game to its end with the given agents and records it.
    pub fn play_game(&mut self, agents: Vec<Box<dyn Agent>>, seed: u64) -> Game {
        let mut g = self.next_game(agents, seed);
        g.run();
        self.record(&g);
        g
    }
}
