//! The worked examples of CR 810 (the Two-Headed Giant variant), played out as tests.
//! P0 and P1 are one team, P2 and P3 the other. The CR 810.8a example of drawing from an
//! empty library is cited by `teams_win_and_lose_together` in `r810_two_headed_giant.rs`.

use super::r600_examples_common::supported;
use super::r800_common::*;
use mtg_engine::events::Event;
use mtg_engine::game::{GameConfig, GameResult};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn thg() -> TestGame {
    TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]))
}

fn life_events(t: &TestGame, from: usize) -> Vec<Event> {
    t.g.turn_events[from..]
        .iter()
        .filter(|e| matches!(e, Event::LifeLost { .. } | Event::LifeGained { .. }))
        .cloned()
        .collect()
}

#[test]
fn transcendence_keeps_its_team_in_the_game_at_zero_life() {
    cr!("810.8a", "810.8c", "104.3b");
    cr_example!("810.8a", "Transcendence");
    supported("Transcendence");
    let mut t = thg();
    // The team is below 20 life ("when you have 20 or more life, you lose the game").
    t.g.lose_life(P1, 15);
    t.settle();
    t.battlefield(P0, "Transcendence");
    t.settle();
    // P1, who doesn't control Transcendence, loses the rest of the team's life.
    t.g.lose_life(P1, 15);
    t.resolve_all();
    assert_eq!(t.life(P0), 0);
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
    assert_eq!(t.g.result, None);
    // Without it, the team loses.
    let mut t = thg();
    t.g.lose_life(P1, 30);
    t.settle();
    assert!(t.has_lost(P0) && t.has_lost(P1));
}

#[test]
fn platinum_angel_protects_both_teammates_and_stops_both_opponents_winning() {
    cr!("810.8a");
    cr_example!("810.8a", "Platinum Angel");
    supported("Platinum Angel");
    supported("Felidar Sovereign");
    let mut t = thg();
    t.battlefield(P3, "Platinum Angel");
    // Neither P2 nor P3 loses with their team at 0 life.
    t.g.lose_life(P2, 30);
    t.settle();
    assert!(!t.has_lost(P2) && !t.has_lost(P3));
    // P1's Felidar Sovereign: P1's team has 40 life at P1's upkeep, but neither P0 nor P1
    // can win.
    t.battlefield(P1, "Felidar Sovereign");
    t.g.gain_life(P0, 10);
    assert_eq!(t.life(P1), 40);
    t.set_step(P3, Step::End);
    to_step(&mut t, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, None);
    assert!(t.g.players.iter().all(|p| p.in_game()));
    // Once the Angel is gone, the team at 0 life loses.
    let angel = t.named_on_battlefield("Platinum Angel")[0];
    t.g.destroy(angel, None);
    t.settle();
    assert!(t.has_lost(P2) && t.has_lost(P3));
}

#[test]
fn flame_rift_deals_each_team_eight_damage() {
    cr!("810.9");
    cr_example!("810.9", "Flame Rift");
    supported("Flame Rift");
    let mut t = thg();
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let rift = t.hand(P0, "Flame Rift");
    let from = t.g.turn_events.len();
    t.cast(P0, rift).go();
    t.resolve();
    // Each player was dealt 4 damage.
    let lost: Vec<(PlayerId, u32)> = life_events(&t, from)
        .into_iter()
        .filter_map(|e| match e {
            Event::LifeLost { player, amount } => Some((player, amount)),
            _ => None,
        })
        .collect();
    assert_eq!(lost.len(), 4);
    assert!(lost.iter().all(|(_, n)| *n == 4));
    assert_eq!((t.life(P0), t.life(P2)), (22, 22));
}

#[test]
fn beacon_of_immortality_doubles_the_teams_life_total() {
    cr!("810.9a");
    cr_example!("810.9a", "Beacon of Immortality");
    supported("Beacon of Immortality");
    let mut t = thg();
    t.g.lose_life(P1, 13);
    t.settle();
    assert_eq!(t.life(P1), 17);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Plains", 6);
    let beacon = t.hand(P0, "Beacon of Immortality");
    let from = t.g.turn_events.len();
    t.cast(P0, beacon).target(Entity::Player(P1)).go();
    t.resolve();
    let evs = life_events(&t, from);
    assert_eq!(evs.len(), 1);
    assert!(matches!(evs[0], Event::LifeGained { player, amount: 17 } if player == P1));
    assert_eq!((t.life(P0), t.life(P1)), (34, 34));
}

#[test]
fn test_of_endurance_wins_with_the_team_at_fifty_life() {
    cr!("810.9a", "810.8a");
    cr_example!("810.9a", "Test of Endurance");
    supported("Test of Endurance");
    let mut t = thg();
    t.battlefield(P0, "Test of Endurance");
    // The teammate gains the life.
    t.g.gain_life(P1, 20);
    assert_eq!(t.life(P0), 50);
    t.set_step(P3, Step::End);
    to_step(&mut t, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0, P1])));
    // At 49 life, nothing happens.
    let mut t = thg();
    t.battlefield(P0, "Test of Endurance");
    t.g.gain_life(P1, 19);
    t.set_step(P3, Step::End);
    to_step(&mut t, P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, None);
}

#[test]
fn lurking_evil_costs_half_the_team_life_rounded_up() {
    cr!("810.9a", "118.3b");
    cr_example!("810.9a", "Lurking Evil");
    supported("Lurking Evil");
    let mut t = thg();
    let evil = t.battlefield(P0, "Lurking Evil");
    t.g.lose_life(P1, 19);
    t.settle();
    assert_eq!(t.life(P0), 11);
    t.set_step(P0, Step::PrecombatMain);
    let from = t.g.turn_events.len();
    t.activate(P0, evil, 0, &[]).expect("activate");
    let evs = life_events(&t, from);
    assert_eq!(evs.len(), 1);
    assert!(matches!(evs[0], Event::LifeLost { player, amount: 6 } if player == P0));
    assert_eq!((t.life(P0), t.life(P1)), (5, 5));
    t.resolve_all();
    assert!(t.obj_now(evil).chars.is(CardType::Creature));
    assert_eq!(t.pt(evil), (4, 4));
    assert!(t.obj_now(evil).has_keyword(KeywordKind::Flying));
}

#[test]
fn a_life_total_set_to_ten_from_a_team_of_25_loses_15() {
    cr!("810.9c");
    cr_example!("810.9c", "has 25 life is targeted");
    supported("Magister Sphinx");
    // Magister Sphinx: "When this creature enters, target player's life total becomes 10."
    let mut t = thg();
    t.g.lose_life(P3, 5);
    t.settle();
    assert_eq!(t.life(P2), 25);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    let from = t.g.turn_events.len();
    t.enter(P0, "Magister Sphinx");
    t.resolve_all();
    let evs = life_events(&t, from);
    assert_eq!(evs.len(), 1);
    assert!(matches!(evs[0], Event::LifeLost { player, amount: 15 } if player == P2));
    assert_eq!((t.life(P2), t.life(P3)), (10, 10));
}

#[test]
fn repay_in_kind_brings_the_team_at_13_down_to_7() {
    cr!("810.9d");
    cr_example!("810.9d", "Repay in Kind");
    supported("Repay in Kind");
    let mut t = thg();
    t.g.lose_life(P1, 23);
    t.g.lose_life(P3, 17);
    t.settle();
    assert_eq!((t.life(P0), t.life(P2)), (7, 13));
    // Each team chooses one of its members to be affected.
    t.answer_choose(P0, &[Entity::Player(P1)]);
    t.answer_choose(P2, &[Entity::Player(P3)]);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 7);
    let repay = t.hand(P0, "Repay in Kind");
    let from = t.g.turn_events.len();
    t.cast(P0, repay).go();
    t.resolve();
    let evs = life_events(&t, from);
    assert_eq!(evs.len(), 1, "{evs:?}");
    assert!(matches!(evs[0], Event::LifeLost { player, amount: 6 } if player == P3));
    assert_eq!((t.life(P0), t.life(P2)), (7, 7));
}
