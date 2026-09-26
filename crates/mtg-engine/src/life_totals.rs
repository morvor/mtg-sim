//! Life totals (CR 119): starting life totals in each variant (CR 119.1a–e), the shared
//! life total of a Two-Headed Giant team (CR 810.4, 810.9), and exchanging life totals
//! (CR 119.7, 119.8).

use crate::game::{Game, Variant};
use crate::types::*;

/// The normal starting life total (CR 119.1).
pub const DEFAULT_STARTING_LIFE: i32 = 20;

/// Whether `p` is an archenemy (CR 904.2a): the only player on their team. In a
/// Supervillain Rumble game (no teams) each player is an archenemy (CR 904.12b).
pub fn is_archenemy(g: &Game, p: PlayerId) -> bool {
    if g.config.variant != Variant::Archenemy {
        return false;
    }
    let team = g.player(p).team;
    g.players.iter().filter(|q| q.team == team).count() == 1
}

/// The number of players on `p`'s team.
fn team_size(g: &Game, p: PlayerId) -> usize {
    let team = g.player(p).team;
    g.players.iter().filter(|q| q.team == team).count()
}

/// The life modifier of the vanguard card(s) `p` owns in the command zone (CR 902.4).
fn vanguard_life_modifier(g: &Game, p: PlayerId) -> i32 {
    g.command
        .iter()
        .map(|id| g.obj(*id))
        .filter(|o| o.owner == p && o.chars.is(CardType::Vanguard))
        .filter_map(|o| o.chars.life_modifier)
        .sum()
}

/// A player's starting life total (CR 119.1). The configured starting life total is
/// used, unless it's the default and the game is a variant with its own starting life
/// totals (CR 119.1a–e).
pub fn starting_life(g: &Game, p: PlayerId) -> i32 {
    let base = g.config.starting_life;
    if base != DEFAULT_STARTING_LIFE {
        return base;
    }
    match g.config.variant {
        // CR 119.1a, 810.4, 810.11: each team's starting life total is 30, plus 15 for
        // each player a team has beyond the second.
        Variant::TwoHeadedGiant => 30 + 15 * team_size(g, p).saturating_sub(2) as i32,
        // CR 119.1b, 902.4: 20 plus or minus the life modifier of the vanguard card.
        Variant::Vanguard => base + vanguard_life_modifier(g, p),
        // CR 119.1c, 903.7; Brawl (CR 119.1d, 903.12): 25 in a two-player game, 30 in a
        // multiplayer game.
        Variant::Commander => {
            if g.config.brawl {
                if g.players.len() == 2 {
                    25
                } else {
                    30
                }
            } else {
                40
            }
        }
        // CR 119.1e, 904.5: the archenemy's starting life total is 40; each other
        // player's is 20.
        Variant::Archenemy if is_archenemy(g, p) => 40,
        _ => base,
    }
}

/// Sets each player's life total to their starting life total (CR 119.1, 103.4).
pub fn set_starting_life_totals(g: &mut Game) {
    for p in g.player_ids() {
        let life = starting_life(g, p);
        g.players[p.idx()].life = life;
    }
}

/// Whether players on a team share a life total (CR 810.4).
pub fn shares_team_life(g: &Game) -> bool {
    g.config.variant == Variant::TwoHeadedGiant
}

/// CR 810.9: damage, life loss and life gain happen to each player individually, and the
/// result is applied to the team's shared life total. After `p`'s life total changed,
/// the rest of the team's life total (the same shared total) follows.
pub fn share_team_life(g: &mut Game, p: PlayerId) {
    if !shares_team_life(g) {
        return;
    }
    let team = g.player(p).team;
    let life = g.player(p).life;
    for q in g.players.iter_mut() {
        if q.team == team {
            q.life = life;
        }
    }
}

/// Exchanges two players' life totals (e.g. "two target players exchange life totals").
/// Each player gains or loses the amount of life necessary to end up with the other's
/// former total (CR 119.5); a player who can't gain life can't make an exchange that
/// would raise their life total, and a player who can't lose life can't make one that
/// would lower it — in either case the exchange doesn't happen (CR 119.7, 119.8).
/// Teammates in Two-Headed Giant can't exchange life totals (CR 810.9e).
pub fn exchange_life_totals(g: &mut Game, a: PlayerId, b: PlayerId) -> bool {
    if a == b || !g.player(a).in_game() || !g.player(b).in_game() {
        return false;
    }
    if shares_team_life(g) && g.player(a).team == g.player(b).team {
        return false;
    }
    let (la, lb) = (g.player(a).life, g.player(b).life);
    let blocked = |g: &Game, p: PlayerId, from: i32, to: i32| {
        (to > from && g.cant_gain_life(p)) || (to < from && g.cant_lose_life(p))
    };
    if blocked(g, a, la, lb) || blocked(g, b, lb, la) {
        return false;
    }
    for (p, from, to) in [(a, la, lb), (b, lb, la)] {
        if to > from {
            g.gain_life(p, (to - from) as u32);
        } else if to < from {
            g.lose_life(p, (from - to) as u32);
        }
    }
    true
}
