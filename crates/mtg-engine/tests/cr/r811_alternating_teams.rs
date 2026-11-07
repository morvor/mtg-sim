//! CR 811: the Alternating Teams variant.

use super::r800_common::*;
use mtg_engine::decision::Action;
use mtg_engine::game::{AttackSide, GameConfig};
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Three teams of two, alternating: 0 1 2 0 1 2.
fn alt6() -> TestGame {
    TestGame::with_config(6, GameConfig::alternating_teams(vec![0, 1, 2, 0, 1, 2]))
}

#[test]
fn two_or_more_teams_of_equal_size() {
    cr!("811.1");
    assert_eq!(
        GameConfig::alternating_teams(vec![0, 1, 0, 1]).validate(4),
        Ok(())
    );
    assert_eq!(
        GameConfig::alternating_teams(vec![0, 1, 2, 0, 1, 2]).validate(6),
        Ok(())
    );
    assert!(GameConfig::alternating_teams(vec![0, 1, 0, 1, 0])
        .validate(5)
        .unwrap_err()
        .contains(&SetupError::TeamSizes));
    assert!(GameConfig::alternating_teams(vec![0, 0, 0])
        .validate(3)
        .unwrap_err()
        .contains(&SetupError::NoTeams));
}

#[test]
fn alternating_teams_options_are_set_before_play() {
    cr!("811.2", "811.2a", "811.2c");
    let c = GameConfig::alternating_teams(vec![0, 1, 2, 0, 1, 2]);
    // The recommended range of influence of 2; no deploying.
    assert_eq!(c.range_of_influence, Some(2));
    assert!(!c.deploy_creatures);
    let t = TestGame::with_config(6, c);
    assert_eq!(t.g.players_in_range(P0), vec![P0, P1, P2, P4, P5]);
}

#[test]
fn exactly_one_attack_option_is_used_in_alternating_teams() {
    cr!("811.2b");
    let both = GameConfig {
        attack_side: Some(AttackSide::Left),
        ..GameConfig::alternating_teams(vec![0, 1, 0, 1])
    };
    assert!(both
        .validate(4)
        .unwrap_err()
        .contains(&SetupError::AttackOptions));
    let left = GameConfig {
        attack_side: Some(AttackSide::Left),
        attack_multiple_players: false,
        ..GameConfig::alternating_teams(vec![0, 1, 0, 1])
    };
    assert_eq!(left.validate(4), Ok(()));
}

#[test]
fn no_one_sits_next_to_a_teammate() {
    cr!("811.3");
    let g = mtg_engine::multiplayer::setup::new_seated(
        GameConfig::alternating_teams(vec![0, 0, 1, 1, 2, 2]),
        (0..6).map(|_| super::r100_common::fillers(5)).collect(),
        vec![],
    );
    // The teams take turns around the table: 0 1 2 0 1 2.
    assert_eq!(g.multiplayer.seats, vec![0, 2, 4, 1, 3, 5]);
    assert_eq!(g.config.teams, Some(vec![0, 1, 2, 0, 1, 2]));
    assert!(GameConfig::alternating_teams(vec![0, 0, 1, 1])
        .validate(4)
        .unwrap_err()
        .contains(&SetupError::TeammatesSeatedTogether));
}

#[test]
fn only_opponents_seated_next_to_you_can_be_attacked() {
    cr!("811.4");
    let mut t = alt6();
    let a = bear(&mut t, P0);
    // P2 and P4 are within P0's range of influence, but not seated next to P0.
    let targets = targets_of(&attack_choices(&mut t, P0), a);
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&Entity::Player(P1)) && targets.contains(&Entity::Player(P5)));
}

#[test]
fn teammates_review_hands_only_when_seated_next_to_each_other() {
    cr!("811.5");
    let mut t = TestGame::with_config(4, GameConfig::alternating_teams(vec![0, 1, 0, 1]));
    let card = t.hand(P0, "Grizzly Bears");
    // P2 is P0's teammate but isn't seated next to P0.
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P2, card));
    // Resources aren't shared: P2 can't cast P0's card.
    t.g.turn.priority = Some(P2);
    assert!(!t
        .g
        .legal_actions(P2)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card)));
    // Once P1 leaves, P0 and P2 sit next to each other.
    concede(&mut t, P1);
    assert!(mtg_engine::facedown::can_look_at(&t.g, P2, card));
}
