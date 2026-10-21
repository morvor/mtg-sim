//! The worked examples of CR 809 (the Emperor variant), played out as tests.

use super::r800_common::*;
use mtg_engine::game::GameConfig;
use mtg_engine::multiplayer::setup;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn at_the_start_neither_emperor_can_attack_though_generals_are_in_spell_range() {
    cr!("809.3c", "809.3a", "801.4");
    cr_example!("809.3c", "neither emperor can attack any opponents");
    // Two teams of three: P1 and P4 are the emperors.
    let mut t = TestGame::with_config(6, GameConfig::emperor(vec![0, 0, 0, 1, 1, 1]));
    for (emperor, generals) in [(P1, [P3, P5]), (P4, [P0, P2])] {
        assert!(t.g.is_emperor(emperor));
        let e = bear(&mut t, emperor);
        assert!(targets_of(&attack_choices(&mut t, emperor), e).is_empty());
        t.g.combat = None;
        // Both opposing generals are within the emperor's range for spells.
        t.set_step(emperor, mtg_engine::turn::Step::PrecombatMain);
        let c = bolt_candidates(&mut t, emperor);
        for g in generals {
            assert!(c.contains(&Entity::Player(g)), "{emperor}: {g}");
        }
    }
}

#[test]
fn teams_of_four_seat_and_range_like_the_example() {
    cr!("809.6", "809.6a", "809.2");
    cr_example!("809.6a", "two teams of four players each");
    // Team A: general 1, emperor, general 2, general 3; then team B the same way.
    let teams = vec![0, 0, 0, 0, 1, 1, 1, 1];
    let config = GameConfig::emperor(teams.clone());
    assert_eq!(config.validate(8), Ok(()));
    let g = setup::new_seated(
        config,
        (0..8).map(|_| super::r100_common::fillers(5)).collect(),
        vec![],
    );
    assert_eq!(g.multiplayer.seats, (0..8).collect::<Vec<_>>());
    assert_eq!(setup::emperor_seats(&teams), vec![1, 5]);
    let t = TestGame::with_config(8, GameConfig::emperor(teams));
    for (seat, range) in [(0, 1), (1, 3), (2, 2), (3, 1), (4, 1), (5, 3), (6, 2), (7, 1)] {
        assert_eq!(
            t.g.range_of_influence(PlayerId(seat)),
            Some(range),
            "seat {seat}"
        );
    }
    assert!(t.g.is_emperor(P1) && t.g.is_emperor(P5));
    // No emperor begins the game within the other's range.
    assert!(!t.g.players_in_range(P1).contains(&P5));
    assert!(!t.g.players_in_range(P5).contains(&P1));
}
