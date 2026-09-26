//! Shared helpers for the CR 500–505 tests (turn structure, beginning phase, main phase).

#![allow(dead_code)]

use crate::r114_common::{spy, Probe};
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::turn::{Phase, Stage, Step};
use mtg_engine::types::*;

/// Runs the game from the current point through the whole of `active`'s next turn
/// (starting it if it hasn't started) and returns the steps that turn had, in order.
pub fn next_turn_steps(t: &mut TestGame, active: PlayerId) -> Vec<Step> {
    let mut log: Vec<Step> = vec![];
    let mut started = false;
    let ok = t.g.run_until(20_000, |g| {
        if g.turn.active == active && g.turn.stage != Stage::PreGame {
            started = true;
            log = g.turn.step_log.clone();
            false
        } else {
            started
        }
    });
    assert!(ok, "{active}'s turn didn't end");
    log
}

/// The phases of a list of steps, one entry per phase: a step starts a new phase when it
/// belongs to another phase than the step before it, or it's the first step of a phase.
pub fn phases(steps: &[Step]) -> Vec<Phase> {
    let mut out: Vec<Phase> = vec![];
    let mut prev: Option<Step> = None;
    for s in steps {
        let new = match prev {
            None => true,
            Some(p) => {
                p.phase() != s.phase()
                    || s.is_main()
                    || matches!(s, Step::Untap | Step::BeginningOfCombat)
                    || *s < p
            }
        };
        if new {
            out.push(s.phase());
        }
        prev = Some(*s);
    }
    out
}

/// Advances until the given step of the current turn has begun and the active player
/// is about to receive priority (before triggers are put on the stack).
pub fn to_step(t: &mut TestGame, step: Step) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == step && g.turn.stage == Stage::Priority) || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "did not reach {step:?}");
}

/// Runs the game until `n` more turns have begun; returns whose turns they were.
pub fn next_turns(t: &mut TestGame, n: usize) -> Vec<PlayerId> {
    let mut out: Vec<PlayerId> = vec![];
    let mut last = t.g.turn.number;
    let ok = t.g.run_until(50_000, |g| {
        if g.turn.number != last {
            last = g.turn.number;
            out.push(g.turn.active);
        }
        out.len() >= n
    });
    assert!(ok, "only {} turns began: {out:?}", out.len());
    out
}

/// Runs until `pred` holds.
pub fn run_to(t: &mut TestGame, what: &str, pred: impl FnMut(&Game) -> bool) {
    assert!(t.g.run_until(20_000, pred), "never reached: {what}");
}

/// Records, for player `p`, the step in which each priority decision was asked (as
/// `"Turn:Step"` strings).
pub fn record_priority(t: &mut TestGame, p: PlayerId) -> Probe {
    spy(t, p, |g, _, d| {
        matches!(d, Decision::Priority { .. })
            .then(|| format!("{}:{:?}", g.turn.number, g.turn.step))
    })
}
