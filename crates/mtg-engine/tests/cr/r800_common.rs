//! Shared helpers for the multiplayer tests (CR 800–811).

#![allow(dead_code)]

pub use super::r506_common::*;
use mtg_engine::decision::{Action, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

pub const P4: PlayerId = PlayerId(4);
pub const P5: PlayerId = PlayerId(5);
pub const P6: PlayerId = PlayerId(6);
pub const P7: PlayerId = PlayerId(7);
pub const P8: PlayerId = PlayerId(8);
pub const P9: PlayerId = PlayerId(9);

/// A Free-for-All game of `n` players in which every player has a range of influence of
/// `range` (CR 801, 806.2a).
pub fn ranged(n: usize, range: u32) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            range_of_influence: Some(range),
            ..GameConfig::free_for_all()
        },
    )
}

/// A test game with the given configuration and teams (seat → team).
pub fn with_teams(teams: &[u8], config: GameConfig) -> TestGame {
    TestGame::with_config(
        teams.len(),
        GameConfig {
            teams: Some(teams.to_vec()),
            ..config
        },
    )
}

/// The candidates of the last target choice `p` was asked to make.
pub fn last_target_candidates(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// The candidates of the last choice among entities `p` was asked to make.
pub fn last_entity_candidates(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *q == p => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

/// Casts a Lightning Bolt from `p`'s hand (with a Mountain to pay for it) and returns the
/// legal targets `p` was offered. The Bolt stays on the stack.
pub fn bolt_candidates(t: &mut TestGame, p: PlayerId) -> Vec<Entity> {
    t.lands(p, "Mountain", 1);
    let bolt = t.hand(p, "Lightning Bolt");
    t.cast(p, bolt).try_go().expect("Lightning Bolt can be cast");
    last_target_candidates(t, p)
}

/// What each of `p`'s creatures could attack, from the attack declaration `p` is asked
/// to make at the start of the declare attackers step of their turn.
pub fn attack_choices(t: &mut TestGame, p: PlayerId) -> Vec<(ObjectId, Vec<Entity>)> {
    t.set_step(p, Step::BeginningOfCombat);
    t.g.combat = None;
    mtg_engine::combat::begin_combat(&mut t.g);
    mtg_engine::combat::attack_options(&t.g)
}

/// The players (and planeswalkers/battles) `creature` could attack.
pub fn targets_of(options: &[(ObjectId, Vec<Entity>)], creature: ObjectId) -> Vec<Entity> {
    options
        .iter()
        .find(|(c, _)| *c == creature)
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

/// A player concedes (CR 104.3a), leaving a multiplayer game (CR 800.4).
pub fn concede(t: &mut TestGame, p: PlayerId) {
    t.g.take_action(p, Action::Concede);
    t.g.flush_events();
}

/// Advances until `active`'s turn has begun and its untap step is over (the upkeep has
/// begun, or the first step after it that occurs).
pub fn to_turn_of(t: &mut TestGame, active: PlayerId) {
    let turn = t.g.turn.number;
    let ok = t.g.run_until(20_000, |g| {
        g.turn.number != turn && g.turn.active == active && g.turn.stage == Stage::Priority
    });
    assert!(ok, "{active}'s turn didn't begin");
}

/// Advances until `step` of `active`'s turn, with priority.
pub fn to_step(t: &mut TestGame, active: PlayerId, step: Step) {
    let ok = t.g.run_until(20_000, |g| {
        g.turn.active == active && g.turn.step == step && g.turn.stage == Stage::Priority
    });
    assert!(ok, "did not reach {step:?} of {active}'s turn");
}

/// Whether the object is on the battlefield (following it through zone changes).
pub fn on_bf(t: &TestGame, id: ObjectId) -> bool {
    t.g.is_live(t.g.current(id)) && t.zone(id) == Zone::Battlefield
}

/// A creature card that can be put into play easily.
pub fn bear(t: &mut TestGame, p: PlayerId) -> ObjectId {
    t.battlefield(p, "Grizzly Bears")
}
