//! Rulings batch P046 — life totals: "a player's life total becomes N" is a gain or loss
//! of life (CR 119.5), starting and current life totals in Two-Headed Giant (CR 810.9),
//! and "if a creature died this turn" when its controller lost the game at the same time
//! (CR 704.3).

use crate::r_p046_common::*;
use crate::r_s01_common::*;
use mtg_engine::game::GameConfig;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Torgaar enters under P0's control targeting `target`.
fn torgaar_targets(t: &mut TestGame, target: PlayerId) {
    t.answer_targets(P0, &[Entity::Player(target)]);
    t.enter(P0, "Torgaar, Famine Incarnate");
    t.resolve_all();
}

#[test]
fn torgaar_makes_the_player_gain_or_lose_life() {
    cr!("119.5", "603.2");
    ruling!(
        "Torgaar, Famine Incarnate",
        "if your life total is 4 when Torgaar's ability resolves targeting you, it will cause you to gain 6 life"
    );
    supported("Torgaar, Famine Incarnate");
    // At 4 life: P0 gains 6, and Ajani's Pridemate ("Whenever you gain life") triggers.
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    t.g.players[0].life = 4;
    torgaar_targets(&mut t, P0);
    assert_eq!(t.life(P0), 10);
    assert_eq!(t.counters(pridemate, "+1/+1"), 1);
    // At 25: P1 loses 15, and P0's Exquisite Blood ("Whenever an opponent loses life, you
    // gain that much life") gains P0 15.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exquisite Blood");
    t.g.players[1].life = 25;
    torgaar_targets(&mut t, P1);
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.life(P0), 35);
}

/// A Two-Headed Giant game (P0 and P1 against P2 and P3) with each team's life total.
fn two_headed_giant(team0: i32, team1: i32) -> TestGame {
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    for (p, life) in [(0, team0), (1, team0), (2, team1), (3, team1)] {
        t.g.players[p].life = life;
    }
    t
}

#[test]
fn in_two_headed_giant_torgaar_uses_the_team_total_but_only_the_target_loses_life() {
    cr!("810.9a", "810.9c", "119.1a");
    ruling!(
        "Torgaar, Famine Incarnate",
        "In a Two-Headed Giant game, Torgaar's ability causes the team's life total to become half the team's starting life total (normally 15, half of 30), but only the target player actually gains or loses life."
    );
    // P2's team is at 22: it becomes 15, and only P2 loses (7) life — P0's Exquisite Blood
    // triggers once.
    let mut t = two_headed_giant(30, 22);
    t.battlefield(P0, "Exquisite Blood");
    torgaar_targets(&mut t, P2);
    assert_eq!((t.life(P2), t.life(P3)), (15, 15));
    assert_eq!((t.life(P0), t.life(P1)), (37, 37));
    assert_eq!(t.g.history.life_lost.get(&P3).copied().unwrap_or(0), 0);
    assert_eq!(t.g.history.life_lost.get(&P2).copied().unwrap_or(0), 7);
}

#[test]
fn shadow_of_mortality_uses_the_team_life_totals_in_two_headed_giant() {
    cr!("810.9a", "119.1a", "601.2f");
    ruling!(
        "Shadow of Mortality",
        "In a Two-Headed Giant game, your starting life total is the life total your team started at, and your current life total is your team's current life total."
    );
    supported("Shadow of Mortality");
    // At 20 life of a team's 30: {10} less.
    let mut t = two_headed_giant(20, 30);
    let s = t.hand(P0, "Shadow of Mortality");
    assert_eq!(cost_to_cast(&mut t, P0, s, &[]), Some(5));
    // At 25: {5} less.
    let mut t = two_headed_giant(25, 30);
    let s = t.hand(P0, "Shadow of Mortality");
    assert_eq!(cost_to_cast(&mut t, P0, s, &[]), Some(10));
    // In a two-player game at 20 of 20: no reduction.
    let mut t = TestGame::new(2);
    let s = t.hand(P0, "Shadow of Mortality");
    assert_eq!(cost_to_cast(&mut t, P0, s, &[]), Some(15));
}

/// In a three-player game, P1 at 2 life controls a Grizzly Bears; P0's Earthquake for 2
/// makes P1 lose the game as the Bears die (one check of state-based actions). Then P0's
/// `name` ("costs {3} less to cast if a creature died this turn") costs {3} less.
fn creature_died_with_its_controller(name: &str) {
    supported(name);
    let mut t = TestGame::new(3);
    t.g.players[1].life = 2;
    t.battlefield(P1, "Grizzly Bears");
    let quake = t.hand(P0, "Earthquake");
    t.g.players[0].mana_pool.add_type(ManaType::R, 3);
    t.cast(P0, quake).x(2).go();
    t.resolve_all();
    assert!(t.has_lost(P1), "{name}");
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty(), "{name}");
    let s = t.hand(P0, name);
    let full = printed_mana_value(name) as usize;
    assert_eq!(cost_to_cast(&mut t, P0, s, &[]), Some(full - 3), "{name}");
}

#[test]
fn a_creature_dying_as_its_controller_loses_counts_as_died() {
    cr!("704.3", "704.5a", "704.5g");
    ruling!(
        "Bone Picker",
        "In a multiplayer game, a player may lose the game at the same time that their creatures die. If so, Bone Picker's cost reduction applies."
    );
    creature_died_with_its_controller("Bone Picker");
    ruling!(
        "Dreaded Bat-Cloud",
        "In a multiplayer game, a player may lose the game at the same time that their creatures die. If so, Dreaded Bat-Cloud's cost reduction applies."
    );
    creature_died_with_its_controller("Dreaded Bat-Cloud");
}
