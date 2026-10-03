//! CR 806: the Free-for-All variant.

use super::r800_common::*;
use mtg_engine::game::{AttackSide, GameConfig, GameResult};
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn errors(c: GameConfig, n: usize) -> Vec<SetupError> {
    c.validate(n).err().unwrap_or_default()
}

#[test]
fn players_compete_as_individuals() {
    cr!("806.1");
    let mut t = TestGame::with_config(4, GameConfig::free_for_all());
    // Every other player is an opponent.
    assert_eq!(t.g.opponents(P0), vec![P1, P2, P3]);
    // The last player standing wins alone.
    for p in [P1, P2] {
        concede(&mut t, p);
    }
    assert_eq!(t.g.result, None);
    concede(&mut t, P3);
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
    // Teams aren't part of a Free-for-All game.
    let teamed = GameConfig {
        teams: Some(vec![0, 0, 1, 1]),
        ..GameConfig::free_for_all()
    };
    assert!(errors(teamed, 4).contains(&SetupError::TeamsInIndividualVariant));
}

#[test]
fn the_options_are_determined_before_play_begins() {
    cr!("806.2");
    // The default options: attack multiple players, unlimited range, no deploying.
    let c = GameConfig::free_for_all();
    assert!(c.attack_multiple_players);
    assert_eq!(c.attack_side, None);
    assert_eq!(c.range_of_influence, None);
    assert!(!c.deploy_creatures);
    assert_eq!(c.validate(5), Ok(()));
    // They're part of the game's configuration, which the game then plays by.
    let mut t = TestGame::with_config(5, c);
    let a = bear(&mut t, P0);
    assert_eq!(targets_of(&attack_choices(&mut t, P0), a).len(), 4);
    assert_eq!(t.g.config.variant, mtg_engine::game::Variant::FreeForAll);
}

#[test]
fn a_limited_range_of_influence_is_the_same_for_everyone() {
    cr!("806.2a");
    let ranged_config = GameConfig {
        range_of_influence: Some(2),
        ..GameConfig::free_for_all()
    };
    assert_eq!(ranged_config.clone().validate(7), Ok(()));
    let t = TestGame::with_config(7, ranged_config);
    for i in 0..7u8 {
        assert_eq!(t.g.range_of_influence(PlayerId(i)), Some(2));
    }
    let uneven = GameConfig {
        range_of_influence: Some(2),
        player_ranges: vec![(P0, 3)],
        ..GameConfig::free_for_all()
    };
    assert!(errors(uneven, 7).contains(&SetupError::UnequalRanges));
}

#[test]
fn exactly_one_attack_option_is_used() {
    cr!("806.2b");
    let none = GameConfig {
        attack_multiple_players: false,
        ..GameConfig::free_for_all()
    };
    assert!(errors(none, 4).contains(&SetupError::AttackOptions));
    let two = GameConfig {
        attack_side: Some(AttackSide::Left),
        ..GameConfig::free_for_all()
    };
    assert!(errors(two, 4).contains(&SetupError::AttackOptions));
    let left = GameConfig {
        attack_side: Some(AttackSide::Right),
        attack_multiple_players: false,
        ..GameConfig::free_for_all()
    };
    assert_eq!(left.validate(4), Ok(()));
}

#[test]
fn the_deploy_creatures_option_isnt_used() {
    cr!("806.2c");
    let deploy = GameConfig {
        deploy_creatures: true,
        ..GameConfig::free_for_all()
    };
    assert!(errors(deploy, 4).contains(&SetupError::DeployNotAllowed));
    let mut t = TestGame::with_config(4, GameConfig::free_for_all());
    let b = bear(&mut t, P0);
    assert!(t
        .obj_now(b)
        .chars
        .abilities
        .iter()
        .all(|a| a.text != mtg_engine::multiplayer::deploy::DEPLOY_TEXT));
}

#[test]
fn players_are_seated_at_random() {
    cr!("806.3");
    let seatings: Vec<Vec<usize>> = (0..12u64)
        .map(|seed| {
            let g = mtg_engine::multiplayer::setup::new_seated(
                GameConfig {
                    seed,
                    ..GameConfig::free_for_all()
                },
                (0..5).map(|_| super::r100_common::fillers(5)).collect(),
                vec![],
            );
            let mut sorted = g.multiplayer.seats.clone();
            sorted.sort();
            assert_eq!(sorted, vec![0, 1, 2, 3, 4], "everyone gets a seat");
            g.multiplayer.seats
        })
        .collect();
    let distinct: std::collections::BTreeSet<Vec<usize>> = seatings.into_iter().collect();
    assert!(distinct.len() > 3, "seatings vary: {distinct:?}");
}
