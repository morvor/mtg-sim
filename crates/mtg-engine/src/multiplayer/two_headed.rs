//! Two-Headed Giant (CR 810): what teams share. The shared life total itself is in
//! [`crate::life_totals`] (CR 810.4, 810.9), the team's state-based actions in
//! [`crate::sba`] (CR 810.8c, 810.8d), shared team turns in [`crate::teams`]
//! (CR 805, 810.2).

use crate::game::{Game, Variant};
use crate::types::*;

/// `PlayerModification::Custom` name: "You can't get poison counters."
pub const CANT_GET_POISON: &str = "can't get poison counters";

/// Players win and lose the game only as a team: when a player loses the game (or
/// concedes, taking their team out of the game), the teammates still in the game who lose
/// with them (CR 810.8a, 810.8b).
pub fn teammates_losing_with(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    if g.config.variant != Variant::TwoHeadedGiant {
        return vec![];
    }
    g.team_members(p)
        .into_iter()
        .filter(|q| *q != p && g.player(*q).in_game())
        .collect()
}

/// Whether the teams of this game share their poison counters (CR 810.10).
pub fn shares_poison(g: &Game) -> bool {
    g.config.variant == Variant::TwoHeadedGiant
}

/// The number of counters of `kind` a player has, as effects see it: with shared poison
/// counters, a player has as many poison counters as their team (CR 810.10a).
pub fn player_counter(g: &Game, p: PlayerId, kind: &str) -> u32 {
    if kind == counters::POISON && shares_poison(g) {
        g.team_members(p)
            .into_iter()
            .map(|q| g.player(q).poison())
            .sum()
    } else {
        g.player(p).counter(kind)
    }
}

/// CR 810.10b: if an effect says a player loses poison counters, that player's team loses
/// that many poison counters: they come off the player first, then off their teammates.
/// Returns the number removed, or `None` if the team rule doesn't apply.
pub fn remove_team_poison(
    g: &mut Game,
    target: Entity,
    kind: &str,
    n: u32,
    by: Option<PlayerId>,
) -> Option<u32> {
    let Entity::Player(p) = target else {
        return None;
    };
    if kind != counters::POISON || !shares_poison(g) {
        return None;
    }
    let mut left = n;
    let mut removed = 0;
    let mut members = vec![p];
    members.extend(g.team_members(p).into_iter().filter(|q| *q != p));
    for q in members {
        if left == 0 {
            break;
        }
        let have = g.player(q).poison();
        let k = have.min(left);
        if k == 0 {
            continue;
        }
        let c = g.players[q.idx()]
            .counters
            .get_mut(counters::POISON)
            .expect("counted above");
        *c -= k;
        if *c == 0 {
            g.players[q.idx()].counters.remove(counters::POISON);
        }
        left -= k;
        removed += k;
        g.dirty = true;
        g.emit(crate::events::Event::CountersRemoved {
            target: Entity::Player(q),
            kind: counters::POISON.into(),
            n: k,
            by,
        });
    }
    Some(removed)
}

/// Whether a player can't get counters of `kind`: an effect says they can't get poison
/// counters — or, with shared poison counters, says so of a player on their team
/// (CR 810.10c).
pub fn cant_get_counters(g: &Game, target: Entity, kind: &str) -> bool {
    let Entity::Player(p) = target else {
        return false;
    };
    if kind != counters::POISON {
        return false;
    }
    let players = if shares_poison(g) {
        g.team_members(p)
    } else {
        vec![p]
    };
    players.into_iter().any(|q| {
        g.player(q).has_mod(
            |m| matches!(m, crate::ability::PlayerModification::Custom(n) if n == CANT_GET_POISON),
        )
    })
}

/// The players an effect that sets life totals affects: with shared life totals, if an
/// effect would set the life total of each player on a team to a number, that team
/// chooses one of its members (its primary player deciding), and on that team only that
/// player is affected (CR 810.9d).
pub fn life_setters(g: &mut Game, players: Vec<PlayerId>) -> Vec<PlayerId> {
    if !crate::life_totals::shares_team_life(g) {
        return players;
    }
    let mut out: Vec<PlayerId> = Vec::new();
    let mut done: Vec<u8> = Vec::new();
    for p in players.clone() {
        let team = g.player(p).team;
        if done.contains(&team) {
            continue;
        }
        done.push(team);
        let members: Vec<PlayerId> = players
            .iter()
            .copied()
            .filter(|q| g.player(*q).team == team)
            .collect();
        let chosen = if members.len() > 1 {
            let chooser = g.primary_player(p);
            g.ask_entities(
                chooser,
                None,
                "Choose the member of your team whose life total is set",
                members.iter().map(|q| Entity::Player(*q)).collect(),
                1,
                1,
            )
            .first()
            .and_then(|e| e.player())
            .filter(|q| members.contains(q))
            .unwrap_or(members[0])
        } else {
            p
        };
        out.push(chosen);
    }
    out
}

/// `Effect::Custom` name: "Redistribute any number of players' life totals."
pub const REDISTRIBUTE_LIFE: &str = "redistribute life totals";

/// Runs a named custom effect from this module, if it is one. Returns true if handled.
pub fn custom_effect(g: &mut Game, name: &str, ctx: &mut crate::eval::Ctx) -> bool {
    if name != REDISTRIBUTE_LIFE {
        return false;
    }
    redistribute_life(g, ctx);
    true
}

/// "Redistribute any number of players' life totals": the controller chooses players,
/// then which of their life totals each of them gets; each gains or loses the life needed
/// (CR 119.5). With shared team life totals, no more than one member of each team can be
/// affected this way (CR 810.9f).
fn redistribute_life(g: &mut Game, ctx: &crate::eval::Ctx) {
    let me = ctx.controller;
    let cands = g.eval_players(&crate::ability::PlayerRef::EachPlayer, ctx);
    let chosen: Vec<PlayerId> = g
        .ask_entities(
            me,
            ctx.source,
            "Choose players whose life totals to redistribute",
            cands.iter().map(|p| Entity::Player(*p)).collect(),
            0,
            cands.len() as u32,
        )
        .into_iter()
        .filter_map(|e| e.player())
        .filter(|p| cands.contains(p))
        .collect();
    let mut players: Vec<PlayerId> = Vec::new();
    for p in chosen {
        let teammate_taken = crate::life_totals::shares_team_life(g)
            && players
                .iter()
                .any(|q| g.player(*q).team == g.player(p).team);
        if !players.contains(&p) && !teammate_taken {
            players.push(p);
        }
    }
    let mut totals: Vec<i32> = players.iter().map(|p| g.player(*p).life).collect();
    let mut assigned: Vec<(PlayerId, i32)> = Vec::new();
    for p in &players {
        let options: Vec<String> = totals.iter().map(|t| t.to_string()).collect();
        let i = g
            .ask_option(me, ctx.source, &format!("Life total for {p}"), options)
            .min(totals.len() - 1);
        assigned.push((*p, totals.remove(i)));
    }
    for (p, to) in assigned {
        let cur = g.player(p).life;
        if to > cur {
            g.gain_life(p, (to - cur) as u32);
        } else if to < cur {
            g.lose_life(p, (cur - to) as u32);
        }
    }
}
