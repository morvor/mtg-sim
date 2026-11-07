//! Shared helpers for the tests of rulings batch P056 (`r_p056_*.rs`): extra untaps
//! (permanents that untap during each other player's untap step, "untap all creatures you
//! control" with additional combat phases and beginning phases), and permanents that make
//! a choice as they enter (a player, a card name). (The helpers of earlier batches are used
//! too.)

#![allow(dead_code)]

use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// The object as an entity (a target or a choice).
pub fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// Whether the object (followed across zone changes) is tapped now.
pub fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

/// Taps the permanent (through the primitive, as if it had been tapped for a cost).
pub fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.tap(id);
}

/// How many times the object became untapped during the current turn.
pub fn untaps_this_turn(t: &TestGame, id: ObjectId) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Untapped { obj } if *obj == id))
        .count()
}

/// Moves on from the end step of the player before `p` into `p`'s upkeep, through `p`'s
/// untap step (two-player games: the other player).
pub fn untap_step_of(t: &mut TestGame, p: PlayerId) {
    let n = t.g.players.len();
    let before = PlayerId(((p.0 as usize + n - 1) % n) as u8);
    t.set_step(before, Step::End);
    t.advance_to(p, Step::Upkeep);
}

/// Runs the game to the end of the current turn and returns the steps the turn had (from
/// its start).
pub fn steps_of_this_turn(t: &mut TestGame) -> Vec<Step> {
    let turn = t.g.turn.number;
    let mut log = vec![];
    let ok = t.g.run_until(20_000, |g| {
        if g.turn.number == turn {
            log = g.turn.step_log.clone();
            false
        } else {
            true
        }
    });
    assert!(ok, "the turn didn't end");
    log
}

/// Advances within the current turn to the next time `step` begins with the active player
/// having priority, and puts the triggered abilities on the stack.
pub fn next_step(t: &mut TestGame, step: Step) {
    let turn = t.g.turn.number;
    let ap = t.g.turn.active;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == step && g.turn.stage == Stage::Priority && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "did not reach {step:?}");
    t.settle();
}

/// Number of times `step` occurs in `steps`.
pub fn count(steps: &[Step], step: Step) -> usize {
    steps.iter().filter(|s| **s == step).count()
}

/// Advances within the current turn until the `n`th occurrence of `step` this turn has
/// begun and the active player has priority.
pub fn nth_step(t: &mut TestGame, step: Step, n: usize) {
    let turn = t.g.turn.number;
    let ap = t.g.turn.active;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == step
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap)
            && g.turn.step_log.iter().filter(|s| **s == step).count() >= n)
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "did not reach {step:?} #{n}");
}
