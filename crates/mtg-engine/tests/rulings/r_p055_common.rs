//! Shared helpers for the tests of rulings batch P055 (`r_p055_*.rs`): exhaust, spells
//! that exile themselves as they resolve, ending the turn, exponential token makers,
//! additional combat phases, extra drafts and extra turns.

#![allow(dead_code)]

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// Every player in the game passes priority in turn, starting with `first` (the top of
/// the stack resolves, or the step ends).
pub fn all_pass(t: &mut TestGame, first: PlayerId) {
    let n = t.g.players.len();
    t.g.turn.passes = 0;
    for i in 0..n {
        let p = PlayerId(((first.idx() + i) % n) as u8);
        t.g.pass_priority(p);
    }
}

/// Advances the game until `active`'s next turn begins, returning the steps that began
/// in the rest of the current turn and whether any player received priority in them.
pub fn run_to_turn_of(t: &mut TestGame, active: PlayerId) -> (Vec<Step>, bool) {
    let mut steps = Vec::new();
    let mut priority = false;
    let turn = t.g.turn.number;
    for _ in 0..2000 {
        if t.g.turn.number != turn && t.g.turn.active == active {
            return (steps, priority);
        }
        let before = t.g.turn.stage;
        t.g.advance();
        if t.g.turn.number == turn {
            if before == Stage::Begin {
                steps.push(t.g.turn.step);
            }
            if t.g.turn.stage == Stage::Priority && t.g.turn.priority.is_some() {
                priority = true;
            }
        }
        if t.g.is_over() {
            break;
        }
    }
    panic!("{active}'s turn never began");
}

/// The active players of the next `n` turns that begin (the current one excluded).
pub fn next_turns(t: &mut TestGame, n: usize) -> Vec<PlayerId> {
    let mut out = Vec::new();
    let mut turn = t.g.turn.number;
    let ok = t.g.run_until(100_000, |g| {
        if g.turn.number != turn {
            turn = g.turn.number;
            out.push(g.turn.active);
        }
        out.len() >= n
    });
    assert!(ok, "only {} turns began: {out:?}", out.len());
    out
}

/// The `ChooseEntities` candidates offered to `p` since decision `from`.
pub fn entity_choices(t: &TestGame, p: PlayerId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .collect()
}

/// Whether `p` was asked a priority decision since decision `from`.
pub fn priority_asked(t: &TestGame, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. }))
}
