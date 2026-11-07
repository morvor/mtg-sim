//! Game-history queries over other stretches of the game than the current turn
//! ([`HistoryWindow`]): "this combat", "last turn", "during your last turn", "since your
//! last turn", "since your last upkeep". Like this turn's history
//! (`kw/value_results.rs`), every event counts that a triggered ability with the queried
//! trigger condition (and the same source and controller) would have triggered on,
//! whether or not anything triggered (CR 603.1b, 603.2).
//!
//! The events of the turns before this one are kept in [`Game::past_turns`] (the latest
//! [`KEEP`] turns), each with the player or team whose turn it was.

use crate::ability::{HistoryWindow, PlayerRef, Tally, TriggerCond};
use crate::eval::Ctx;
use crate::events::Event;
use crate::game::Game;
use crate::turn::Step;
use crate::types::{ObjectId, PlayerId};
use std::collections::BTreeSet;

/// How many past turns are kept: enough for "your last turn" in a game of several players
/// with a few extra turns.
pub const KEEP: usize = 16;

/// A turn that has ended: whose it was and what happened.
#[derive(Clone, Debug, Default)]
pub struct PastTurn {
    pub active: Vec<PlayerId>,
    pub events: Vec<Event>,
}

/// Records the turn that just ended (called as the next turn begins).
pub fn record_turn(g: &mut Game, active: Vec<PlayerId>, events: Vec<Event>) {
    g.past_turns.push(PastTurn { active, events });
    if g.past_turns.len() > KEEP {
        let extra = g.past_turns.len() - KEEP;
        g.past_turns.drain(..extra);
    }
}

/// The index in [`Game::past_turns`] of the latest turn of the player before this one.
fn players_last_turn(g: &Game, p: PlayerId) -> Option<usize> {
    g.past_turns.iter().rposition(|t| t.active.contains(&p))
}

/// The events of this turn so far, including those of the current action not yet
/// processed (CR 608.2h).
fn this_turn(g: &Game) -> impl Iterator<Item = &Event> {
    g.turn_events.iter().chain(g.events.iter())
}

/// The events of the window, in order.
pub fn window_events<'a>(g: &'a Game, w: &HistoryWindow, ctx: &Ctx) -> Vec<&'a Event> {
    let player = |r: &PlayerRef| g.eval_player(r, ctx);
    match w {
        HistoryWindow::ThisCombat => {
            let all: Vec<&Event> = this_turn(g).collect();
            let start = all
                .iter()
                .rposition(|e| {
                    matches!(
                        e,
                        Event::StepBegan {
                            step: Step::BeginningOfCombat,
                            ..
                        }
                    )
                })
                // Without a combat phase begun this turn: the whole turn so far.
                .map_or(0, |i| i + 1);
            all[start..].to_vec()
        }
        HistoryWindow::ThisGame => g.game_events.iter().chain(this_turn(g)).collect(),
        HistoryWindow::LastTurn => g
            .past_turns
            .last()
            .map(|t| t.events.iter().collect())
            .unwrap_or_default(),
        HistoryWindow::PlayersLastTurn(r) => player(r)
            .and_then(|p| players_last_turn(g, p))
            .map(|i| g.past_turns[i].events.iter().collect())
            .unwrap_or_default(),
        HistoryWindow::SinceLastTurn(r) => {
            // Without an earlier turn of theirs: everything kept.
            let from = player(r)
                .and_then(|p| players_last_turn(g, p))
                .map_or(0, |i| i + 1);
            g.past_turns[from..]
                .iter()
                .flat_map(|t| t.events.iter())
                .chain(this_turn(g))
                .collect()
        }
        HistoryWindow::PlayersLastCombat(r) => player(r)
            .map(|p| players_last_combat(g, p))
            .unwrap_or_default(),
        HistoryWindow::SinceLastUpkeep(r) => {
            let Some(p) = player(r) else {
                return vec![];
            };
            let Some(i) = players_last_turn(g, p) else {
                return g
                    .past_turns
                    .iter()
                    .flat_map(|t| t.events.iter())
                    .chain(this_turn(g))
                    .collect();
            };
            let last = &g.past_turns[i].events;
            let start = last
                .iter()
                .position(|e| {
                    matches!(
                        e,
                        Event::StepBegan {
                            step: Step::Upkeep,
                            ..
                        }
                    )
                })
                .unwrap_or(0);
            last[start..]
                .iter()
                .chain(g.past_turns[i + 1..].iter().flat_map(|t| t.events.iter()))
                .chain(this_turn(g))
                .collect()
        }
    }
}

/// The combat phases among `events` (one turn's), in order: each from its beginning of
/// combat step until the next step that isn't a combat step (CR 506.1).
fn combat_phases<'a>(events: &[&'a Event]) -> Vec<Vec<&'a Event>> {
    let mut phases: Vec<Vec<&Event>> = Vec::new();
    let mut open = false;
    for e in events {
        if let Event::StepBegan { step, .. } = e {
            if *step == Step::BeginningOfCombat {
                phases.push(Vec::new());
                open = true;
            } else if !step.is_combat() {
                open = false;
            }
        }
        if open {
            if let Some(p) = phases.last_mut() {
                p.push(*e);
            }
        }
    }
    phases
}

/// [`HistoryWindow::PlayersLastCombat`]: the events of the latest combat phase of `p`'s
/// turns before the current combat phase. During a combat phase of `p`'s turn, that's an
/// earlier combat phase of this turn if there's one (CR 500.8); otherwise the last combat
/// phase of the latest earlier turn of theirs that had one (a turn whose combat phase was
/// skipped had no combat, CR 500.11). Nothing with no such combat.
fn players_last_combat(g: &Game, p: PlayerId) -> Vec<&Event> {
    if g.is_active_player(p) {
        let now: Vec<&Event> = this_turn(g).collect();
        let mut phases = combat_phases(&now);
        if g.turn.step.is_combat() {
            phases.pop();
        }
        if let Some(last) = phases.pop() {
            return last;
        }
    }
    for t in g.past_turns.iter().rev().filter(|t| t.active.contains(&p)) {
        let events: Vec<&Event> = t.events.iter().collect();
        if let Some(last) = combat_phases(&events).pop() {
            return last;
        }
    }
    vec![]
}

/// [`crate::ability::Value::EventsDuring`].
pub fn events_during(
    g: &Game,
    w: &HistoryWindow,
    cond: &TriggerCond,
    tally: Tally,
    ctx: &Ctx,
) -> i64 {
    let mut n = 0i64;
    let mut players = BTreeSet::new();
    for ev in window_events(g, w, ctx) {
        for info in g.trigger_matches_ctx(cond, ctx, ev) {
            match tally {
                Tally::Events => n += 1,
                Tally::Amount => n += info.amount.max(0) as i64,
                Tally::Players => {
                    // Players who have since left the game included (CR 800.4i).
                    if let Some(p) = info.player {
                        players.insert(p);
                    }
                }
            }
        }
    }
    if tally == Tally::Players {
        n = players.len() as i64;
    }
    n
}

/// [`crate::ability::Sel::EventsDuring`]: for a spell cast, the spell; for damage dealt,
/// its source; otherwise the object the event was about. Each object once.
pub fn objects_during(g: &Game, w: &HistoryWindow, cond: &TriggerCond, ctx: &Ctx) -> Vec<ObjectId> {
    let mut out = Vec::new();
    for ev in window_events(g, w, ctx) {
        for info in g.trigger_matches_ctx(cond, ctx, ev) {
            let o = match cond {
                TriggerCond::DealsDamage { .. } => info.other.or(info.object),
                _ => info.spell.or(info.object),
            };
            if let Some(o) = o.filter(|o| !out.contains(o)) {
                out.push(o);
            }
        }
    }
    out
}
