//! CR 701.34b: proliferating in a Two-Headed Giant game, where poison counters are shared
//! by the team (CR 810.10).
//!
//! * A player has the kinds of counters their team has (CR 810.10d): a player whose
//!   teammate has poison counters can be chosen, and is given a poison counter.
//! * If more than one player on a team is chosen, only one of them can be given an
//!   additional poison counter; the player who proliferates chooses which (CR 701.34b).
//!
//! Used by `keyword_actions_impl::proliferate`.

use super::*;
use crate::game::Variant;
use crate::types::counters;

fn shares_poison(g: &Game) -> bool {
    g.config.variant == Variant::TwoHeadedGiant
}

/// The team's poison counters (CR 810.10a).
fn team_poison(g: &Game, p: PlayerId) -> u32 {
    g.player(p).poison()
        + g.teammates(p)
            .into_iter()
            .map(|q| g.player(q).poison())
            .sum::<u32>()
}

/// The kinds of counters player `p` has, for proliferate: their own, plus poison if their
/// team shares poison counters and has any (CR 810.10d).
pub fn player_counter_kinds(g: &Game, p: PlayerId) -> Vec<CounterKind> {
    let mut kinds: Vec<CounterKind> = g
        .player(p)
        .counters
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(k, _)| k.clone())
        .collect();
    if shares_poison(g)
        && team_poison(g, p) > 0
        && !kinds.iter().any(|k| k.as_str() == counters::POISON)
    {
        kinds.push(CounterKind::from(counters::POISON));
    }
    kinds
}

/// Of the chosen players, those who don't get an additional poison counter: on each team
/// with more than one chosen player who'd get one, all but the one `proliferator` chooses
/// (CR 701.34b).
pub fn players_without_poison(
    g: &mut Game,
    proliferator: PlayerId,
    chosen: &[Entity],
    source: Option<ObjectId>,
) -> Vec<PlayerId> {
    if !shares_poison(g) {
        return vec![];
    }
    let poisoned: Vec<PlayerId> = chosen
        .iter()
        .filter_map(|e| match e {
            Entity::Player(p) => Some(*p),
            _ => None,
        })
        .filter(|p| {
            player_counter_kinds(g, *p)
                .iter()
                .any(|k| k.as_str() == counters::POISON)
        })
        .collect();
    let mut out = vec![];
    let mut done: Vec<PlayerId> = vec![];
    for p in &poisoned {
        if done.contains(p) {
            continue;
        }
        let team: Vec<PlayerId> = poisoned
            .iter()
            .copied()
            .filter(|q| q == p || g.teammates(*p).contains(q))
            .collect();
        done.extend(team.iter().copied());
        if team.len() < 2 {
            continue;
        }
        let cands: Vec<Entity> = team.iter().map(|q| Entity::Player(*q)).collect();
        let pick = g
            .ask_entities(
                proliferator,
                source,
                "Proliferate: choose the player on this team who gets a poison counter",
                cands,
                1,
                1,
            )
            .into_iter()
            .find_map(|e| match e {
                Entity::Player(q) if team.contains(&q) => Some(q),
                _ => None,
            })
            .unwrap_or(team[0]);
        out.extend(team.into_iter().filter(|q| *q != pick));
    }
    out
}
