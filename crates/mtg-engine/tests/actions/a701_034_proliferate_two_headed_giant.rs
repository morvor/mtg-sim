//! CR 701.34b: proliferating in a Two-Headed Giant game.

use crate::a701_028_071_common::*;
use mtg_engine::ability::{KeywordAction, Sel};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn two_headed() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

fn poison(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).poison()
}

fn proliferate(t: &mut TestGame) {
    run(
        t,
        P0,
        None,
        ka(KeywordAction::Proliferate, Sel::None, 1),
        &[],
    );
}

#[test]
fn only_one_player_on_a_team_gets_an_additional_poison_counter() {
    cr!("701.34b");
    let mut t = two_headed();
    t.g.players[2].counters.insert(counters::POISON.into(), 3);
    t.g.players[3].counters.insert(counters::POISON.into(), 1);
    t.g.players[3].counters.insert(counters::ENERGY.into(), 2);
    // P0 chooses both opponents, then which of them gets the poison counter.
    t.answer_choose(P0, &[Entity::Player(P2), Entity::Player(P3)]);
    t.answer_choose(P0, &[Entity::Player(P3)]);
    proliferate(&mut t);
    assert_eq!(poison(&t, P2), 3);
    assert_eq!(poison(&t, P3), 2);
    // Other kinds of counters aren't shared: P3 still gets an energy counter.
    assert_eq!(t.g.player(P3).counter(counters::ENERGY), 3);
    // The team got one poison counter in all.
    assert_eq!(poison(&t, P2) + poison(&t, P3), 5);
    // The proliferating player was asked to pick one of the two.
    let picks: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(picks.len(), 2);
    assert_eq!(picks[1], vec![Entity::Player(P2), Entity::Player(P3)]);
}

#[test]
fn a_player_has_their_teams_poison_counters() {
    cr!("701.34b");
    // P3 has no poison counters of their own, but their team has: they can be chosen and
    // given one.
    let mut t = two_headed();
    t.g.players[2].counters.insert(counters::POISON.into(), 4);
    t.answer_choose(P0, &[Entity::Player(P3)]);
    proliferate(&mut t);
    assert_eq!(poison(&t, P2), 4);
    assert_eq!(poison(&t, P3), 1);
    let cands: Vec<Entity> = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("asked");
    assert!(cands.contains(&Entity::Player(P2)) && cands.contains(&Entity::Player(P3)));
    assert!(!cands.contains(&Entity::Player(P0)));
    // Outside Two-Headed Giant, only a player with counters can be chosen.
    let mut t = TestGame::new(4);
    t.g.players[2].counters.insert(counters::POISON.into(), 4);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    proliferate(&mut t);
    let cands: Vec<Entity> = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .expect("asked");
    assert_eq!(cands, vec![Entity::Player(P2)]);
    assert_eq!(poison(&t, P2), 5);
    assert_eq!(poison(&t, P3), 0);
}
