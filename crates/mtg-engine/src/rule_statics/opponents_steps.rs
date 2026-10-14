//! Statics that act during other players' turns (an emblem of Teferi, Who Slows the
//! Sunset):
//!
//! * "Untap all permanents you control during each opponent's untap step": they untap
//!   along with the active player's permanents (CR 502.3), like "during each other
//!   player's untap step" (`Restriction::UntapDuringOthersUntapSteps`) but only in the
//!   untap steps of that player's opponents.
//! * "You draw a card during each opponent's draw step": the player draws a card as the
//!   active player draws for the turn (CR 504.1), not by a triggered ability.
//!
//! Both are `StaticEffect::Custom` static abilities with these names.

use crate::game::Game;
use crate::types::{ObjectId, PlayerId};

/// "Untap all permanents you control during each opponent's untap step."
pub const UNTAP_DURING_OPPONENTS_UNTAP_STEPS: &str =
    "untap all permanents you control during each opponent's untap step";
/// "You draw a card during each opponent's draw step."
pub const DRAW_DURING_OPPONENTS_DRAW_STEPS: &str =
    "you draw a card during each opponent's draw step";

/// The players with `name` (one entry per such ability) for whom the active players are
/// opponents.
fn holders(g: &Game, name: &str) -> Vec<PlayerId> {
    let actives = g.active_players();
    g.statics
        .customs
        .iter()
        .filter(|(_, c, n)| {
            n.as_str() == name
                && !actives.contains(c)
                && actives.iter().any(|a| g.are_opponents(*a, *c))
        })
        .map(|(_, c, _)| *c)
        .collect()
}

/// The permanents that untap in this untap step because of their controllers' "untap all
/// permanents you control during each opponent's untap step" (several such abilities
/// untap a permanent once).
pub fn extra_untaps(g: &Game) -> Vec<ObjectId> {
    let players = holders(g, UNTAP_DURING_OPPONENTS_UNTAP_STEPS);
    g.permanents()
        .filter(|o| o.tapped && players.contains(&o.controller))
        .map(|o| o.id)
        .collect()
}

/// The draws of this draw step's "you draw a card during each opponent's draw step"
/// abilities: each such ability draws its controller a card.
pub fn extra_draws(g: &mut Game) {
    for p in holders(g, DRAW_DURING_OPPONENTS_DRAW_STEPS) {
        g.draw_cards(p, 1);
    }
}
