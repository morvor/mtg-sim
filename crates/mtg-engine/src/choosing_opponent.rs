//! Which opponent makes a choice "an opponent" makes (register
//! `controller-picks-the-choosing-opponent`, decided by the user on Nov 19).
//!
//! CR 601.7a: "If there is more than one opponent who could make such a choice, the
//! spell's controller decides which of those opponents will make the choice." (602.3a: the
//! same for activated abilities.) The user extends this to every "an opponent [chooses,
//! separates, ...]", including choices made as a spell or ability resolves: the controller
//! of the spell or ability picks the opponent, unless the text, a ruling or the CR says
//! otherwise. Every place the engine picks the opponent who makes such a choice comes here:
//! the controller is asked among the candidates (those within their range of influence,
//! CR 801.5a, worked out by the caller), and by default the next of them in turn order
//! after the controller makes the choice.

use crate::game::Game;
use crate::types::{Entity, ObjectId, PlayerId};

/// `players` in turn order starting with the next player after `from` (`from` last).
pub fn in_turn_order_after(g: &Game, from: PlayerId, players: &[PlayerId]) -> Vec<PlayerId> {
    let n = g.players.len();
    let mut out: Vec<PlayerId> = (1..=n)
        .map(|k| g.seat_in_turn_order(from, k))
        .filter(|p| players.contains(p))
        .collect();
    // Any not seated (none in practice) keep their place at the end.
    let rest: Vec<PlayerId> = players
        .iter()
        .copied()
        .filter(|p| !out.contains(p))
        .collect();
    out.extend(rest);
    out
}

/// `controller` picks which of `candidates` (players, usually their opponents) makes a
/// choice for a spell or ability they control, offered in turn order from the next player
/// after them; by default that next one. `None` if there's no candidate. A lone candidate
/// is picked without asking unless `ask_alone` (an explicit "choose an opponent", which
/// the controller is asked even with one opponent).
pub fn pick(
    g: &mut Game,
    controller: PlayerId,
    source: Option<ObjectId>,
    candidates: &[PlayerId],
    prompt: &str,
    ask_alone: bool,
) -> Option<PlayerId> {
    let order = in_turn_order_after(g, controller, candidates);
    match order.as_slice() {
        [] => None,
        [one] if !ask_alone => Some(*one),
        _ => Some(
            g.ask_entities(
                controller,
                source,
                prompt,
                order.iter().map(|p| Entity::Player(*p)).collect(),
                1,
                1,
            )
            .first()
            .and_then(|e| e.player())
            .filter(|p| order.contains(p))
            .unwrap_or(order[0]),
        ),
    }
}

/// The prompt for picking the opponent who makes a choice.
pub const PROMPT: &str = "Choose the opponent who makes the choice";
