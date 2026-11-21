//! Shared helpers for the tests of rulings batch P101 (`r_p101_*.rs`): the second batch of
//! "mana value matters" rulings — opening hands and Leylines, alternative costs and free
//! casts, Pacts, Moxen and spirit guides, control of spells, and effects that choose or
//! compare mana values. (The helpers of batch P100 and earlier batches are used too.)

#![allow(dead_code)]

pub use crate::r_p100_common::*;
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::*;

/// A game of `n` players that hasn't begun, each with a 40-card deck of `name`, `starting`
/// the starting player; `mulligans` lists players who take one mulligan. Each player
/// answers yes to every opening-hand action. Runs the procedure before the first turn
/// (CR 103) and returns the game and the players asked about opening-hand actions, in
/// order (one entry per prompt), with the index of the last mulligan decision before them.
pub fn opening_hands(
    n: usize,
    name: &str,
    starting: PlayerId,
    mulligans: &[PlayerId],
) -> (TestGame, Vec<PlayerId>) {
    let decks = (0..n).map(|_| vec![card(name); 40]).collect();
    let mut t = crate::r_s12_common::pregame(
        GameConfig {
            starting_player: Some(starting),
            ..Default::default()
        },
        decks,
    );
    for p in mulligans {
        t.answer(
            *p,
            DecisionKind::Mulligan,
            mtg_engine::decision::Answer::Bool(true),
        );
    }
    for i in 0..n {
        for _ in 0..8 {
            t.answer_yes(PlayerId(i as u8), true);
        }
    }
    t.g.pregame();
    let asked = t.asked();
    let last_mulligan = asked
        .iter()
        .rposition(|(_, d)| matches!(d, Decision::Mulligan { .. }));
    let prompts: Vec<(usize, PlayerId)> = asked
        .iter()
        .enumerate()
        .filter(|(_, (_, d))| {
            matches!(d, Decision::YesNo { prompt, .. }
                if prompt.starts_with("Begin the game with") || prompt.starts_with("Reveal "))
        })
        .map(|(i, (p, _))| (i, *p))
        .collect();
    if let Some(m) = last_mulligan {
        assert!(
            prompts.iter().all(|(i, _)| *i > m),
            "{name}: opening-hand actions come after every mulligan"
        );
    }
    (t, prompts.into_iter().map(|(_, p)| p).collect())
}

/// Collapses runs of the same player: the order in which players took their actions.
pub fn player_runs(ps: &[PlayerId]) -> Vec<PlayerId> {
    let mut v: Vec<PlayerId> = vec![];
    for p in ps {
        if v.last() != Some(p) {
            v.push(*p);
        }
    }
    v
}

/// The number of permanents named `name` that `p` controls.
pub fn controls_named(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.name == name)
        .count()
}
