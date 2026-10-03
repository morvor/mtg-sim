//! CR 804: the deploy creatures option.

use super::r800_common::*;
use mtg_engine::game::GameConfig;
use mtg_engine::multiplayer::deploy::DEPLOY_TEXT;
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn has_deploy(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| a.text == DEPLOY_TEXT)
}

#[test]
fn emperor_games_always_use_deploy_creatures_and_team_games_may() {
    cr!("804.1");
    // The Emperor variant always uses it.
    let emperor = GameConfig::emperor(vec![0, 0, 0, 1, 1, 1]);
    assert!(emperor.deploy_creatures);
    assert_eq!(emperor.validate(6), Ok(()));
    let without = GameConfig {
        deploy_creatures: false,
        ..GameConfig::emperor(vec![0, 0, 0, 1, 1, 1])
    };
    assert!(without
        .validate(6)
        .unwrap_err()
        .contains(&SetupError::DeployRequired));
    // Another team variant may use it.
    let tvt = GameConfig {
        deploy_creatures: true,
        ..GameConfig::team_vs_team(vec![0, 0, 1, 1])
    };
    assert_eq!(tvt.validate(4), Ok(()));
    let mut t = TestGame::with_config(4, tvt);
    let b = bear(&mut t, P0);
    assert!(has_deploy(&t, b));
    // Individual variants don't.
    let ffa = GameConfig {
        deploy_creatures: true,
        ..GameConfig::free_for_all()
    };
    assert!(ffa
        .validate(4)
        .unwrap_err()
        .contains(&SetupError::DeployNotAllowed));
}

#[test]
fn each_creature_can_give_itself_to_a_teammate_as_a_sorcery() {
    cr!("804.2");
    let mut t = with_teams(
        &[0, 0, 1, 1],
        GameConfig {
            deploy_creatures: true,
            ..GameConfig::team_vs_team(vec![])
        },
    );
    let b = bear(&mut t, P0);
    let land = t.battlefield(P0, "Forest");
    assert!(has_deploy(&t, b));
    assert!(!has_deploy(&t, land), "only creatures have it");
    // Only a teammate can be the target.
    t.activate(P0, b, 0, &[Entity::Player(P1)]).unwrap();
    let offered = last_target_candidates(&t, P0);
    assert_eq!(offered, vec![Entity::Player(P1)]);
    // It costs {T}.
    assert!(t.obj_now(b).tapped);
    t.resolve();
    assert_eq!(t.obj_now(b).controller, P1);
    // Activate only as a sorcery: not during combat, nor while the stack isn't empty.
    let c = bear(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.activate(P0, c, 0, &[Entity::Player(P1)]).is_err());
    t.set_step(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P2).go();
    assert!(t.activate(P0, c, 0, &[Entity::Player(P1)]).is_err());
    t.resolve();
    assert!(t.activate(P0, c, 0, &[Entity::Player(P1)]).is_ok());
    t.resolve();
    assert_eq!(t.obj_now(c).controller, P1);
    // Without teammates there's no legal target.
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            deploy_creatures: true,
            ..GameConfig::default()
        },
    );
    let b = bear(&mut t, P0);
    assert!(t.activate(P0, b, 0, &[Entity::Player(P1)]).is_err());
}
