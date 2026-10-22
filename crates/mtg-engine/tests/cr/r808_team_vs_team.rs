//! CR 808: the Team vs. Team variant.

use super::r800_common::*;
use mtg_engine::decision::Action;
use mtg_engine::game::{GameConfig, GameResult};
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn two_or_more_teams_of_any_size() {
    cr!("808.1");
    // Teams of different sizes, and three teams.
    assert_eq!(GameConfig::team_vs_team(vec![0, 0, 0, 1, 1]).validate(5), Ok(()));
    assert_eq!(
        GameConfig::team_vs_team(vec![0, 0, 1, 1, 2, 2]).validate(6),
        Ok(())
    );
    assert!(GameConfig::team_vs_team(vec![0, 0, 0])
        .validate(3)
        .unwrap_err()
        .contains(&SetupError::NoTeams));
    // A team wins together once the other teams are gone.
    let mut t = TestGame::with_config(5, GameConfig::team_vs_team(vec![0, 0, 0, 1, 1]));
    concede(&mut t, P3);
    concede(&mut t, P4);
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0, P1, P2])));
}

#[test]
fn each_team_sits_together_in_the_order_it_chooses() {
    cr!("808.2");
    let g = mtg_engine::multiplayer::setup::new_seated(
        GameConfig::team_vs_team(vec![1, 0, 1, 0, 1]),
        (0..5).map(|_| super::r100_common::fillers(5)).collect(),
        vec![],
    );
    // Team 1 (participants 0, 2, 4, in that order) sits together, then team 0.
    assert_eq!(g.multiplayer.seats, vec![0, 2, 4, 1, 3]);
    assert_eq!(g.config.teams, Some(vec![1, 1, 1, 0, 0]));
    assert!(GameConfig::team_vs_team(vec![0, 1, 0, 1])
        .validate(4)
        .unwrap_err()
        .contains(&SetupError::TeamsNotTogether));
}

#[test]
fn team_vs_team_options_are_set_before_play() {
    cr!("808.3", "808.3a", "808.3b");
    let c = GameConfig::team_vs_team(vec![0, 0, 1, 1]);
    // The attack multiple players option is used; deploying and limited ranges usually
    // aren't.
    assert!(c.attack_multiple_players);
    assert!(!c.deploy_creatures);
    assert_eq!(c.range_of_influence, None);
    let mut t = TestGame::with_config(4, c);
    let a = bear(&mut t, P0);
    let b = bear(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    assert_eq!(t.g.combat.as_ref().unwrap().defending_players, vec![P2, P3]);
    // P0 attacks both opponents at once.
    declare(&mut t, &[(a, Entity::Player(P2)), (b, Entity::Player(P3))]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!((t.life(P2), t.life(P3)), (18, 18));
    // They may be used if the players want.
    let with = GameConfig {
        deploy_creatures: true,
        range_of_influence: Some(2),
        ..GameConfig::team_vs_team(vec![0, 0, 1, 1])
    };
    assert_eq!(with.validate(4), Ok(()));
}

#[test]
fn a_random_team_goes_first_from_its_center_seat() {
    cr!("808.4");
    // Team 0 has three players (seats 0-2): its center seat, P1, goes first. Team 1 has
    // two (seats 3-4): the player to the left of its midpoint, P4, goes first.
    let mut firsts = std::collections::BTreeSet::new();
    for seed in 0..16u64 {
        let mut t = super::r100_common::pregame(
            GameConfig {
                seed,
                skip_mulligans: true,
                ..GameConfig::team_vs_team(vec![0, 0, 0, 1, 1])
            },
            (0..5).map(|_| super::r100_common::fillers(20)).collect(),
        );
        t.g.start();
        firsts.insert(t.g.turn.starting_player);
    }
    assert_eq!(firsts.into_iter().collect::<Vec<_>>(), vec![P1, P4]);
    // Turn order goes to the players' left.
    let mut t = TestGame::with_config(5, GameConfig::team_vs_team(vec![0, 0, 0, 1, 1]));
    t.set_step(P4, Step::Cleanup);
    to_turn_of(&mut t, P0);
    to_turn_of(&mut t, P1);
    assert_eq!(t.g.turn.previous_active, Some(P0));
}

#[test]
fn team_resources_arent_shared_but_hands_can_be_reviewed() {
    cr!("808.5");
    let mut t = TestGame::with_config(4, GameConfig::team_vs_team(vec![0, 0, 1, 1]));
    let bolt = t.hand(P0, "Lightning Bolt");
    let land = t.battlefield(P0, "Mountain");
    t.g.turn.priority = Some(P1);
    let actions = t.g.legal_actions(P1);
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == bolt)));
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == land)));
    assert!(mtg_engine::facedown::can_look_at(&t.g, P1, bolt));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P2, bolt));
}
