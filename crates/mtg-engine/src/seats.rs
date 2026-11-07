//! Seats around the table: "the player to your left/right", "the opponent to your left",
//! "choose left or right ... the next player in the chosen direction", and "each player
//! may attack only the nearest opponent in the chosen direction" ([`PlayerRef::Seat`],
//! [`SeatSide`], [`Restriction::AttackOnlyNearestOpponent`]).
//!
//! Players sit in turn order: the next player in turn order is the one seated to the
//! left (CR 101.4, 103.1: turn order proceeds clockwise). Seats ignore teams (the player
//! to your right may be a teammate, Hithlain Rope ruling; Inniaz in Two-Headed Giant),
//! except "the nearest opponent" in a direction, which skips teammates (Mystic Barrier
//! ruling). In a two-player game the other player sits on both sides. Seats are where
//! the players sit, not the current turn order (`multiplayer::attack::neighbors`).
//!
//! A direction chosen for an object ("choose left or right") is its chosen word; the
//! latest choice is "the last chosen direction". With none chosen, the direction is
//! undefined and so is everything that depends on it (Pramikon ruling: its restriction
//! has no effect).

use crate::ability::*;
use crate::game::Game;
use crate::types::*;

/// The words of "choose left or right".
pub const DIRECTIONS: [&str; 2] = ["left", "right"];

/// `side` resolved for `source`: `Some(true)` for left, `Some(false)` for right, `None`
/// if it's the chosen direction and none was chosen.
pub fn resolve_side(g: &Game, side: SeatSide, source: Option<ObjectId>) -> Option<bool> {
    match side {
        SeatSide::Left => Some(true),
        SeatSide::Right => Some(false),
        SeatSide::Chosen => {
            let o = g.try_obj(source?)?;
            match o.choices.text.as_deref() {
                Some("left") => Some(true),
                Some("right") => Some(false),
                _ => None,
            }
        }
    }
}

/// The players still in the game in seating order, starting with `p` and going around
/// the table to the left (`left`) or to the right; empty if `p` isn't in the game.
pub fn around(g: &Game, p: PlayerId, left: bool) -> Vec<PlayerId> {
    let seated = g.players_in_game();
    let Some(i) = seated.iter().position(|q| *q == p) else {
        return vec![];
    };
    let n = seated.len();
    (0..n)
        .map(|k| {
            if left {
                seated[(i + k) % n]
            } else {
                seated[(i + n - k) % n]
            }
        })
        .collect()
}

/// The player seated next to `p` on that side (`left`), if any other player is in the
/// game.
pub fn neighbor(g: &Game, p: PlayerId, left: bool) -> Option<PlayerId> {
    around(g, p, left).get(1).copied()
}

/// The opponent of `p` seated nearest to them on that side, skipping teammates.
pub fn nearest_opponent(g: &Game, p: PlayerId, left: bool) -> Option<PlayerId> {
    around(g, p, left)
        .into_iter()
        .skip(1)
        .find(|q| g.are_opponents(p, *q))
}

/// [`PlayerRef::Seat`] for one player.
pub fn seat_of(
    g: &Game,
    p: PlayerId,
    side: SeatSide,
    opponent: bool,
    source: Option<ObjectId>,
) -> Option<PlayerId> {
    let left = resolve_side(g, side, source)?;
    if opponent {
        nearest_opponent(g, p, left)
    } else {
        neighbor(g, p, left)
    }
}

/// Whether the attack-direction restrictions in effect let a creature controlled by
/// `attacker` be declared as attacking `defender` (the player, or the controller of the
/// planeswalker, or the protector of the battle; `battle`: it's a battle). Each such
/// restriction allows only the nearest opponent in its direction and that opponent's
/// planeswalkers; all of them apply (Pramikon, Mystic Barrier and Teyo rulings: if they
/// disagree on the direction, in a game of three or more players no one can attack).
pub fn attack_allowed(g: &Game, attacker: PlayerId, defender: PlayerId, battle: bool) -> bool {
    g.all_restrictions().iter().all(|(src, _, r, _)| match r {
        Restriction::AttackOnlyNearestOpponent { side } => match resolve_side(g, *side, *src) {
            Some(left) => !battle && nearest_opponent(g, attacker, left) == Some(defender),
            None => true,
        },
        _ => true,
    })
}
