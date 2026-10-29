//! CR 803: the attack left and attack right options.

use super::r800_common::*;
use mtg_engine::game::{AttackSide, GameConfig};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn sided(n: usize, side: AttackSide) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            attack_side: Some(side),
            attack_multiple_players: false,
            ..GameConfig::free_for_all()
        },
    )
}

#[test]
fn some_games_use_the_attack_left_or_attack_right_option() {
    cr!("803.1");
    // Without the option, P0 may attack any opponent; with either option, only the one on
    // that side: P1 to the left, P4 to the right.
    let mut free = TestGame::with_config(5, GameConfig::free_for_all());
    let a = bear(&mut free, P0);
    assert_eq!(targets_of(&attack_choices(&mut free, P0), a).len(), 4);
    for (side, only) in [(AttackSide::Left, P1), (AttackSide::Right, P4)] {
        let mut t = sided(5, side);
        let a = bear(&mut t, P0);
        assert_eq!(
            targets_of(&attack_choices(&mut t, P0), a),
            vec![Entity::Player(only)],
            "{side:?}"
        );
    }
}

#[test]
fn with_attack_left_only_the_opponent_immediately_to_the_left() {
    cr!("803.1a");
    let mut t = sided(5, AttackSide::Left);
    let a = bear(&mut t, P2);
    // The player to P2's left is the next player in turn order, P3.
    let jace = t.battlefield(P3, "Jace Beleren");
    t.battlefield(P1, "Jace Beleren");
    let targets = targets_of(&attack_choices(&mut t, P2), a);
    assert_eq!(targets.len(), 2);
    assert!(targets.contains(&Entity::Player(P3)));
    assert!(targets.contains(&Entity::Object(jace)));
    // P4's left-hand neighbor is P0, around the table.
    let mut t = sided(5, AttackSide::Left);
    let a = bear(&mut t, P4);
    assert_eq!(targets_of(&attack_choices(&mut t, P4), a), vec![Entity::Player(P0)]);
    // If the nearest opponent to the left is more than one seat away (a teammate sits
    // there), the player can't attack.
    let mut t = with_teams(
        &[0, 0, 1, 1, 2],
        GameConfig {
            attack_side: Some(AttackSide::Left),
            attack_multiple_players: false,
            ..GameConfig::team_vs_team(vec![])
        },
    );
    let a = bear(&mut t, P0);
    assert!(targets_of(&attack_choices(&mut t, P0), a).is_empty());
    let b = bear(&mut t, P1);
    assert_eq!(targets_of(&attack_choices(&mut t, P1), b), vec![Entity::Player(P2)]);
}

#[test]
fn with_attack_right_only_the_opponent_immediately_to_the_right() {
    cr!("803.1b");
    let mut t = sided(5, AttackSide::Right);
    let a = bear(&mut t, P2);
    let targets = targets_of(&attack_choices(&mut t, P2), a);
    assert_eq!(targets, vec![Entity::Player(P1)]);
    let a0 = bear(&mut t, P0);
    assert_eq!(targets_of(&attack_choices(&mut t, P0), a0), vec![Entity::Player(P4)]);
    // A teammate to the right: no attack.
    let mut t = with_teams(
        &[0, 0, 1, 1, 2],
        GameConfig {
            attack_side: Some(AttackSide::Right),
            attack_multiple_players: false,
            ..GameConfig::team_vs_team(vec![])
        },
    );
    let b = bear(&mut t, P1);
    assert!(targets_of(&attack_choices(&mut t, P1), b).is_empty());
}
