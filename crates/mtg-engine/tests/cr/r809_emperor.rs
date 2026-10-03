//! CR 809: the Emperor variant.

use super::r800_common::*;
use mtg_engine::decision::Action;
use mtg_engine::game::{GameConfig, GameResult};
use mtg_engine::multiplayer::setup::SetupError;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Two teams of three: P0, P1 (emperor), P2 against P3, P4 (emperor), P5.
fn emperor6() -> TestGame {
    TestGame::with_config(6, GameConfig::emperor(vec![0, 0, 0, 1, 1, 1]))
}

#[test]
fn teams_of_three_players() {
    cr!("809.1");
    assert_eq!(GameConfig::emperor(vec![0, 0, 0, 1, 1, 1]).validate(6), Ok(()));
    assert_eq!(
        GameConfig::emperor(vec![0, 0, 0, 1, 1, 1, 2, 2, 2]).validate(9),
        Ok(())
    );
    assert!(GameConfig::emperor(vec![0, 0, 1, 1])
        .validate(4)
        .unwrap_err()
        .contains(&SetupError::TeamSizes));
}

#[test]
fn each_emperor_sits_in_the_middle_of_their_team() {
    cr!("809.2");
    let g = mtg_engine::multiplayer::setup::new_seated(
        GameConfig::emperor(vec![0, 1, 0, 1, 0, 1]),
        (0..6).map(|_| super::r100_common::fillers(5)).collect(),
        vec![],
    );
    // Each team sits together, in the order it chose; the middle seat is the emperor.
    assert_eq!(g.multiplayer.seats, vec![0, 2, 4, 1, 3, 5]);
    assert!(g.is_emperor(P1) && g.is_emperor(P4));
    assert!(!g.is_emperor(P0) && !g.is_emperor(P2) && !g.is_emperor(P3));
}

#[test]
fn the_emperor_variants_default_options() {
    cr!("809.3", "809.3b");
    let t = emperor6();
    assert!(t.g.config.deploy_creatures);
    assert_eq!(t.g.range_of_influence(P1), Some(2));
    assert_eq!(t.g.range_of_influence(P0), Some(1));
    // Deploy creatures: a general can give a creature to the emperor.
    let mut t = emperor6();
    let b = bear(&mut t, P0);
    t.activate(P0, b, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(b).controller, P1);
}

#[test]
fn a_player_can_attack_only_an_opponent_next_to_them() {
    cr!("809.3c");
    let mut t = emperor6();
    // P2 sits next to P3; P0 next to P5 (around the table).
    let g2 = bear(&mut t, P2);
    assert_eq!(targets_of(&attack_choices(&mut t, P2), g2), vec![Entity::Player(P3)]);
    let g0 = bear(&mut t, P0);
    assert_eq!(targets_of(&attack_choices(&mut t, P0), g0), vec![Entity::Player(P5)]);
    // The emperor's neighbors are teammates: though P3 and P5 are within the emperor's
    // range of influence, the emperor can't attack them.
    let e = bear(&mut t, P1);
    assert!(t.g.players_in_range(P1).contains(&P3));
    assert!(targets_of(&attack_choices(&mut t, P1), e).is_empty());
}

#[test]
fn a_random_emperor_goes_first() {
    cr!("809.4");
    let mut firsts = std::collections::BTreeSet::new();
    for seed in 0..16u64 {
        let mut t = super::r100_common::pregame(
            GameConfig {
                seed,
                skip_mulligans: true,
                ..GameConfig::emperor(vec![0, 0, 0, 1, 1, 1])
            },
            (0..6).map(|_| super::r100_common::fillers(20)).collect(),
        );
        t.g.start();
        firsts.insert(t.g.turn.starting_player);
    }
    assert_eq!(firsts.into_iter().collect::<Vec<_>>(), vec![P1, P4]);
    // Turn order goes to the players' left.
    let mut t = emperor6();
    t.set_step(P1, Step::Cleanup);
    to_turn_of(&mut t, P2);
    assert_eq!(t.g.turn.previous_active, Some(P1));
}

#[test]
fn a_team_wins_and_loses_with_its_emperor() {
    cr!("809.5", "809.5a", "809.5b");
    let mut t = emperor6();
    // A general losing doesn't end the team's game.
    concede(&mut t, P3);
    assert!(t.g.player(P4).in_game() && t.g.player(P5).in_game());
    // The emperor losing does: the team loses, and the other team — with its emperor —
    // wins.
    t.g.players[4].library.clear();
    t.g.draw_cards(P4, 1);
    t.settle();
    assert!(t.has_lost(P4) && t.has_lost(P5));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0, P1, P2])));
}

#[test]
fn a_draw_for_the_emperor_is_a_draw_for_the_team() {
    cr!("809.5c");
    // Divine Intervention (P2, a general with range of influence 1): the game is a draw
    // for P2 and the players within P2's range of influence, P1 and P3 (CR 801.15). P1 is
    // an emperor, so it's a draw for P1's whole team — P0 too, though P0 is two seats
    // from P2.
    let mut t = emperor6();
    let di = t.battlefield(P2, "Divine Intervention");
    t.g.objects[di.0 as usize]
        .counters
        .insert("intervention".into(), 1);
    t.set_step(P1, Step::End);
    to_step(&mut t, P2, Step::Upkeep);
    t.resolve_all();
    assert!(!t.g.players_in_range(P2).contains(&P0));
    for p in [P0, P1, P2, P3] {
        assert!(t.g.drew_game(p), "{p}");
    }
    // P3 is only a general: it's not a draw for the rest of P3's team, which keeps
    // playing — and, the only team left, wins.
    assert!(!t.g.drew_game(P4) && !t.g.drew_game(P5));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P4, P5])));
}

#[test]
fn larger_emperor_teams_adjust_their_ranges() {
    cr!("809.6", "809.6a");
    // Two teams of five: G G E G G | G G E G G.
    let teams = vec![0, 0, 0, 0, 0, 1, 1, 1, 1, 1];
    assert_eq!(GameConfig::emperor(teams.clone()).validate(10), Ok(()));
    assert!(GameConfig::emperor(vec![0, 0, 0, 0, 0, 1, 1, 1])
        .validate(8)
        .unwrap_err()
        .contains(&SetupError::TeamSizes));
    let t = TestGame::with_config(10, GameConfig::emperor(teams));
    assert!(t.g.is_emperor(PlayerId(2)) && t.g.is_emperor(PlayerId(7)));
    // Generals: the smallest range reaching one opposing general.
    assert_eq!(t.g.range_of_influence(P4), Some(1), "P5 sits next to P4");
    assert_eq!(t.g.range_of_influence(P3), Some(2));
    assert_eq!(t.g.range_of_influence(P0), Some(1), "P9 sits next to P0");
    // Emperors: the smallest reaching two opposing generals.
    assert_eq!(t.g.range_of_influence(PlayerId(2)), Some(3));
    // No emperor begins the game within another's range.
    assert!(!t.g.players_in_range(PlayerId(2)).contains(&PlayerId(7)));
}

#[test]
fn emperor_team_resources_arent_shared() {
    cr!("809.7");
    let mut t = emperor6();
    let card = t.hand(P0, "Grizzly Bears");
    let land = t.battlefield(P0, "Forest");
    t.g.turn.priority = Some(P1);
    let actions = t.g.legal_actions(P1);
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card)));
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::Activate { source, .. } if *source == land)));
    assert!(mtg_engine::facedown::can_look_at(&t.g, P1, card));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P5, card));
    assert_eq!(t.zone(card), Zone::Hand(P0));
}
