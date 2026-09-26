//! Setting up a multiplayer game (CR 800.2, 800.5, 806–811): each variant's default
//! options, checking that a game's options and teams fit its variant, seating the
//! players, and who goes first in the team variants that say so.
//!
//! Seats are player indices: `PlayerId(i)` sits in seat `i`, and turn order proceeds from
//! each seat to the next one, which is to that player's left (CR 101.4).

use crate::agents::RandomAgent;
use crate::card::CardDef;
use crate::decision::Agent;
use crate::game::{Game, GameConfig, Variant};
use crate::types::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A way in which a game's configuration doesn't fit its multiplayer variant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SetupError {
    /// Multiplayer variants and options are for games that begin with more than two
    /// players (CR 800.1).
    NotMultiplayer,
    /// Players in a Free-for-All or Grand Melee game compete as individuals (CR 806.1,
    /// 807.1).
    TeamsInIndividualVariant,
    /// The variant is played between teams (CR 808.1, 809.1, 810.1, 811.1).
    NoTeams,
    /// Teams of the wrong size or of different sizes (CR 809.1, 809.6, 810.1, 810.11,
    /// 811.1).
    TeamSizes,
    /// Exactly one of the attack left, attack right and attack multiple players options
    /// must be used (CR 806.2b, 811.2b).
    AttackOptions,
    /// The attack multiple players option isn't used in Grand Melee (CR 807.2c).
    AttackMultipleNotAllowed,
    /// The deploy creatures option isn't used in Free-for-All or Grand Melee games
    /// (CR 806.2c, 807.2c).
    DeployNotAllowed,
    /// The Emperor variant always uses the deploy creatures option (CR 804.1, 809.3b).
    DeployRequired,
    /// In Free-for-All, every player has the same range of influence (CR 806.2a).
    UnequalRanges,
    /// Each team sits together (CR 808.2, 809.2, 810.3); the shared team turns option can
    /// be used only if each team's members sit in adjacent seats (CR 805.1).
    TeamsNotTogether,
    /// No one sits next to a teammate, and each team is equally spaced out (CR 811.3).
    TeammatesSeatedTogether,
    /// The shared team turns option is only for games between teams (CR 805.1).
    SharedTeamTurnsWithoutTeams,
}

impl GameConfig {
    /// A Free-for-All game (CR 806): players compete as individuals, with the attack
    /// multiple players option, no limited range of influence and no deploy creatures
    /// option (CR 806.2a–c).
    pub fn free_for_all() -> GameConfig {
        GameConfig {
            variant: Variant::FreeForAll,
            attack_multiple_players: true,
            ..Default::default()
        }
    }

    /// A Grand Melee game (CR 807): each player has a range of influence of 1, and the
    /// attack left option is used (CR 807.2a–c).
    pub fn grand_melee() -> GameConfig {
        GameConfig {
            variant: Variant::GrandMelee,
            range_of_influence: Some(1),
            attack_side: Some(crate::game::AttackSide::Left),
            attack_multiple_players: false,
            ..Default::default()
        }
    }

    /// A Team vs. Team game (CR 808) with the attack multiple players option (CR 808.3a).
    /// `teams` gives each seat's team.
    pub fn team_vs_team(teams: Vec<u8>) -> GameConfig {
        GameConfig {
            variant: Variant::TeamVsTeam,
            teams: Some(teams),
            attack_multiple_players: true,
            ..Default::default()
        }
    }

    /// An Emperor game (CR 809): ranges of influence of 2 for emperors and 1 for generals,
    /// the deploy creatures option, and attacks only on neighbors (CR 809.3a–c).
    pub fn emperor(teams: Vec<u8>) -> GameConfig {
        GameConfig {
            variant: Variant::Emperor,
            teams: Some(teams),
            deploy_creatures: true,
            attack_multiple_players: true,
            ..Default::default()
        }
    }

    /// A Two-Headed Giant game (CR 810): shared team turns (CR 810.2) and a shared life
    /// total (CR 810.4).
    pub fn two_headed_giant(teams: Vec<u8>) -> GameConfig {
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(teams),
            ..Default::default()
        }
    }

    /// An Alternating Teams game (CR 811): the recommended range of influence of 2 and the
    /// attack multiple players option (CR 811.2a–c).
    pub fn alternating_teams(teams: Vec<u8>) -> GameConfig {
        GameConfig {
            variant: Variant::AlternatingTeams,
            teams: Some(teams),
            range_of_influence: Some(2),
            attack_multiple_players: true,
            ..Default::default()
        }
    }

    /// Checks that the options and seating fit the variant for a game of `players`
    /// players, as seated (seat `i` is player `i`). A game uses only one variant, and any
    /// number of options (CR 800.2).
    pub fn validate(&self, players: usize) -> Result<(), Vec<SetupError>> {
        let mut errors = Vec::new();
        let teams: Vec<u8> = match &self.teams {
            Some(t) => (0..players)
                .map(|i| t.get(i).copied().unwrap_or(i as u8))
                .collect(),
            None => (0..players).map(|i| i as u8).collect(),
        };
        let team_sizes = team_sizes(&teams);
        let has_teams = team_sizes.iter().any(|n| *n > 1);
        let multiplayer_variant = !matches!(
            self.variant,
            Variant::Standard
                | Variant::Commander
                | Variant::Planechase
                | Variant::Archenemy
                | Variant::Vanguard
        );
        if multiplayer_variant && players <= 2 {
            errors.push(SetupError::NotMultiplayer);
        }
        let attack_options = [
            self.attack_multiple_players,
            self.attack_side == Some(crate::game::AttackSide::Left),
            self.attack_side == Some(crate::game::AttackSide::Right),
        ]
        .iter()
        .filter(|b| **b)
        .count();
        match self.variant {
            Variant::FreeForAll | Variant::GrandMelee => {
                if has_teams {
                    errors.push(SetupError::TeamsInIndividualVariant);
                }
                if self.deploy_creatures {
                    errors.push(SetupError::DeployNotAllowed);
                }
                if self.variant == Variant::FreeForAll {
                    if attack_options != 1 {
                        errors.push(SetupError::AttackOptions);
                    }
                    if !self.player_ranges.is_empty() {
                        errors.push(SetupError::UnequalRanges);
                    }
                } else if self.attack_multiple_players {
                    errors.push(SetupError::AttackMultipleNotAllowed);
                }
            }
            Variant::TeamVsTeam => {
                if team_sizes.len() < 2 {
                    errors.push(SetupError::NoTeams);
                }
                if !teams_together(&teams) {
                    errors.push(SetupError::TeamsNotTogether);
                }
            }
            Variant::Emperor => {
                if team_sizes.len() < 2 {
                    errors.push(SetupError::NoTeams);
                }
                // Teams of three (CR 809.1), or any number of equally sized teams
                // (CR 809.6).
                if team_sizes.iter().any(|n| *n != team_sizes[0]) || team_sizes[0] < 3 {
                    errors.push(SetupError::TeamSizes);
                }
                if !teams_together(&teams) {
                    errors.push(SetupError::TeamsNotTogether);
                }
                if !self.deploy_creatures {
                    errors.push(SetupError::DeployRequired);
                }
            }
            Variant::TwoHeadedGiant => {
                // Two teams of two players (CR 810.1), or equally sized larger teams
                // (CR 810.11).
                if team_sizes.len() != 2 || team_sizes[0] != team_sizes[1] || team_sizes[0] < 2 {
                    errors.push(SetupError::TeamSizes);
                }
                if !teams_together(&teams) {
                    errors.push(SetupError::TeamsNotTogether);
                }
            }
            Variant::AlternatingTeams => {
                if team_sizes.len() < 2 {
                    errors.push(SetupError::NoTeams);
                } else if team_sizes.iter().any(|n| *n != team_sizes[0]) {
                    errors.push(SetupError::TeamSizes);
                }
                if attack_options != 1 {
                    errors.push(SetupError::AttackOptions);
                }
                if !alternating(&teams) {
                    errors.push(SetupError::TeammatesSeatedTogether);
                }
            }
            _ => {}
        }
        if self.shared_team_turns {
            if !has_teams {
                errors.push(SetupError::SharedTeamTurnsWithoutTeams);
            } else if !teams_together(&teams) && !errors.contains(&SetupError::TeamsNotTogether) {
                errors.push(SetupError::TeamsNotTogether);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// The number of players on each team, in order of first appearance.
fn team_sizes(teams: &[u8]) -> Vec<usize> {
    let mut order: Vec<u8> = Vec::new();
    for t in teams {
        if !order.contains(t) {
            order.push(*t);
        }
    }
    order
        .iter()
        .map(|t| teams.iter().filter(|x| *x == t).count())
        .collect()
}

/// Whether each team's members sit in adjacent seats (around the table).
pub fn teams_together(teams: &[u8]) -> bool {
    let n = teams.len();
    if n == 0 {
        return true;
    }
    // Around the table, each team is one run of seats: the number of places where the
    // team changes between neighbors equals the number of teams (or is 0 for one team).
    let changes = (0..n).filter(|i| teams[*i] != teams[(i + 1) % n]).count();
    let distinct = team_sizes(teams).len();
    if distinct == 1 {
        changes == 0
    } else {
        changes == distinct
    }
}

/// Whether no one sits next to a teammate and the teams are equally spaced out
/// (CR 811.3): the seats repeat one fixed order of teams.
pub fn alternating(teams: &[u8]) -> bool {
    let k = team_sizes(teams).len();
    let n = teams.len();
    k > 1 && n % k == 0 && (0..n).all(|i| teams[i] == teams[i % k])
}

/// Seats the participants of a game for its variant (CR 800.5): returns, for each seat in
/// turn order, the index of the participant who sits there. `teams` gives each
/// participant's team.
///
/// * Free-for-All and Grand Melee players are seated at random (CR 806.3, 807.3).
/// * In Team vs. Team, Emperor and Two-Headed Giant games each team sits together, in the
///   order its players chose (their order in `teams`) (CR 808.2, 809.2, 810.3).
/// * In Alternating Teams games no one sits next to a teammate and each team is equally
///   spaced out (CR 811.3).
/// * Otherwise the players stay where they are, a mutually agreeable method (CR 800.5).
pub fn seating(config: &GameConfig, teams: &[u8], rng: &mut ChaCha8Rng) -> Vec<usize> {
    let n = teams.len();
    let mut order: Vec<u8> = Vec::new();
    for t in teams {
        if !order.contains(t) {
            order.push(*t);
        }
    }
    let members = |t: u8| -> Vec<usize> { (0..n).filter(|i| teams[*i] == t).collect() };
    match config.variant {
        Variant::FreeForAll | Variant::GrandMelee => {
            let mut v: Vec<usize> = (0..n).collect();
            v.shuffle(rng);
            v
        }
        Variant::TeamVsTeam | Variant::Emperor | Variant::TwoHeadedGiant => {
            order.iter().flat_map(|t| members(*t)).collect()
        }
        Variant::AlternatingTeams => {
            let groups: Vec<Vec<usize>> = order.iter().map(|t| members(*t)).collect();
            let size = groups.iter().map(|g| g.len()).max().unwrap_or(0);
            (0..size)
                .flat_map(|i| groups.iter().filter_map(move |g| g.get(i).copied()))
                .collect()
        }
        _ => (0..n).collect(),
    }
}

/// Creates a game whose participants are first seated as the variant prescribes
/// ([`seating`]): participant `i` brings `decks[i]`, `agents[i]` and team
/// `config.teams[i]`; the player in seat `s` is the participant `seats[s]`, recorded in
/// `g.multiplayer.seats`.
pub fn new_seated(
    config: GameConfig,
    decks: Vec<Vec<Arc<CardDef>>>,
    agents: Vec<Box<dyn Agent>>,
) -> Game {
    let n = decks.len();
    let teams: Vec<u8> = match &config.teams {
        Some(t) => (0..n)
            .map(|i| t.get(i).copied().unwrap_or(i as u8))
            .collect(),
        None => (0..n).map(|i| i as u8).collect(),
    };
    let mut rng = ChaCha8Rng::seed_from_u64(config.seed ^ 0x5ea7_5ea7);
    let seats = seating(&config, &teams, &mut rng);
    let mut decks: Vec<Option<Vec<Arc<CardDef>>>> = decks.into_iter().map(Some).collect();
    let mut agents: Vec<Option<Box<dyn Agent>>> = agents.into_iter().map(Some).collect();
    let seated_decks: Vec<Vec<Arc<CardDef>>> = seats
        .iter()
        .map(|i| decks[*i].take().unwrap_or_default())
        .collect();
    let seated_agents: Vec<Box<dyn Agent>> = seats
        .iter()
        .map(|i| {
            agents
                .get_mut(*i)
                .and_then(|a| a.take())
                .unwrap_or_else(|| Box::new(RandomAgent::new(*i as u64)))
        })
        .collect();
    let config = GameConfig {
        teams: config
            .teams
            .as_ref()
            .map(|_| seats.iter().map(|i| teams[*i]).collect()),
        ..config
    };
    let mut g = Game::new(config, seated_decks, seated_agents);
    g.multiplayer.seats = seats;
    g
}

/// Each team's players in seat order around the table, starting with the player whose
/// right-hand neighbor isn't a teammate (for a team seated together, the team's rightmost
/// seat from its own perspective).
pub fn team_in_seat_order(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    let n = g.players.len();
    let team = g.player(p).team;
    let members: Vec<PlayerId> = g
        .players
        .iter()
        .filter(|q| q.team == team)
        .map(|q| q.id)
        .collect();
    let Some(first) = members
        .iter()
        .copied()
        .find(|q| g.players[(q.idx() + n - 1) % n].team != team)
    else {
        return members;
    };
    let mut out = Vec::new();
    for i in 0..n {
        let q = PlayerId(((first.idx() + i) % n) as u8);
        if g.player(q).team == team {
            out.push(q);
        }
    }
    out
}

/// The primary player of `p`'s team (CR 805.2): the player seated in the team's
/// rightmost seat from the team's perspective — the team member whose right-hand
/// neighbor (the previous seat in turn order) isn't on the team. Among players still in
/// the game.
pub fn primary_player(g: &Game, p: PlayerId) -> PlayerId {
    let order = team_in_seat_order(g, p);
    order
        .iter()
        .copied()
        .find(|q| g.player(*q).in_game())
        .or_else(|| order.first().copied())
        .unwrap_or(p)
}

/// Who goes first in the variants that say how (CR 808.4, 809.4), if the game didn't set a
/// starting player:
///
/// * Team vs. Team: a random team; its center seat, or with an even number of players
///   the player to the left of its midpoint.
/// * Emperor: a random emperor.
pub fn variant_starting_player(g: &mut Game) -> Option<PlayerId> {
    match g.config.variant {
        Variant::TeamVsTeam | Variant::Emperor => {}
        _ => return None,
    }
    let reps: Vec<PlayerId> = {
        let mut seen: Vec<u8> = Vec::new();
        let mut out = Vec::new();
        for pl in &g.players {
            if !seen.contains(&pl.team) {
                seen.push(pl.team);
                out.push(pl.id);
            }
        }
        out
    };
    if reps.is_empty() {
        return None;
    }
    let k = g.random_range(0, reps.len() as u32 - 1) as usize;
    let team = team_in_seat_order(g, reps[k]);
    if g.config.variant == Variant::Emperor {
        return g.emperor_of(reps[k]);
    }
    // CR 808.4: odd — the center seat; even — the player to the left of the midpoint,
    // the first seat of the team's second half (turn order goes to the left).
    team.get(team.len() / 2).copied()
}

/// Whether `viewer` may look at the cards in teammate `owner`'s hand. In the Team vs.
/// Team, Emperor and Two-Headed Giant variants teammates may review each other's hands at
/// any time (CR 808.5, 809.7, 810.5); in Alternating Teams only teammates sitting next to
/// each other may (CR 811.5). Resources aren't shared otherwise: teammates can't
/// manipulate each other's cards or permanents.
pub fn may_review_hand(g: &Game, viewer: PlayerId, owner: PlayerId) -> bool {
    if viewer == owner || g.are_opponents(viewer, owner) {
        return false;
    }
    match g.config.variant {
        Variant::TeamVsTeam | Variant::Emperor | Variant::TwoHeadedGiant => true,
        Variant::AlternatingTeams => {
            let (l, r) = super::attack::neighbors(g, viewer);
            l == Some(owner) || r == Some(owner)
        }
        _ => false,
    }
}
