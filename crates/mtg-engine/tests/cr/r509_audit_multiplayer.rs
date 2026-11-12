//! CR 802.3–802.5, 805.10, 800.4e audit: combat with several defending players. Each
//! defending player declares blockers in APNAP order and only for creatures attacking
//! them (802.4–802.4a); each declaration's legality ignores the other players' attackers
//! and blockers (802.4b), so "no more than one creature can block" and "can't block alone"
//! are checked per defending player; blocking creatures assign their damage in APNAP
//! order (802.5); combat damage to a player who left isn't assigned (800.4e); the
//! defending team declares one combined block (805.10d). Also "attacking alone" as a
//! current state (506.5) and "attacks a player alone" (506.6).

use crate::r506_audit_common::*;
use crate::r506_common::{block, declare, go_to};
use mtg_engine::decision::Decision;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn three_players() -> TestGame {
    TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

fn blockers_asked(t: &TestGame) -> Vec<PlayerId> {
    t.asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::DeclareBlockers { .. }))
        .map(|(p, _)| p)
        .collect()
}

#[test]
fn defending_players_declare_blockers_in_apnap_order_each_for_attackers_of_their_own() {
    cr!("802.4", "802.4a");
    let mut t = three_players();
    // P1's turn: the defending players are P2 and P0, in that (APNAP) order.
    let a2 = t.battlefield(P1, "Grizzly Bears");
    let a0 = t.battlefield(P1, "Hill Giant");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    let b0 = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(a2, Entity::Player(P2)), (a0, Entity::Player(P0))],
    );
    // P2 tries to block the Giant attacking P0: illegal, so P2 doesn't block. P0 blocks
    // the Giant.
    block(&mut t, P2, &[(b2, a0)]);
    block(&mut t, P0, &[(b0, a0)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(blockers_asked(&t), vec![P2, P0]);
    assert!(blocking(&t, b2).is_empty());
    assert_eq!(blocking(&t, b0), vec![a0]);
    assert!(!blocked(&t, a2));
}

#[test]
fn a_blocker_limit_counts_only_that_defending_players_blockers() {
    cr!("802.4b", "509.1b");
    // Silent Arbiter: "No more than one creature can block each combat." With creatures
    // attacking two players, each defending player's declaration is checked ignoring the
    // other's blockers, so each may block with one creature.
    supported("Silent Arbiter");
    let mut t = three_players();
    t.battlefield(P2, "Silent Arbiter");
    let x = t.battlefield(P0, "Grizzly Bears");
    let y = t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Hill Giant");
    let b2 = t.battlefield(P2, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    // The Arbiter also lets only one creature attack: the second is made attacking by an
    // effect (CR 508.4c).
    declare(&mut t, &[(x, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(mtg_engine::combat::make_attacking(
        &mut t.g,
        y,
        Entity::Player(P2)
    ));
    block(&mut t, P1, &[(b1, x)]);
    block(&mut t, P2, &[(b2, y)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(blocking(&t, b1), vec![x]);
    assert_eq!(blocking(&t, b2), vec![y]);
}

#[test]
fn mogg_flunkies_cant_block_alone_even_if_another_defending_player_blocks() {
    cr!("802.4b", "509.1b");
    let mut t = three_players();
    let x = t.battlefield(P0, "Grizzly Bears");
    let y = t.battlefield(P0, "Grizzly Bears");
    let flunkies = t.battlefield(P1, "Mogg Flunkies");
    let giant = t.battlefield(P2, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(x, Entity::Player(P1)), (y, Entity::Player(P2))]);
    block(&mut t, P1, &[(flunkies, x)]);
    block(&mut t, P2, &[(giant, y)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(blocking(&t, flunkies).is_empty());
    assert_eq!(blocking(&t, giant), vec![y]);
}

#[test]
fn lure_attacking_one_player_forces_only_that_players_creatures_to_block() {
    cr!("802.4a", "509.1c");
    supported("Lure");
    let mut t = three_players();
    let lured = t.battlefield(P0, "Grizzly Bears");
    let lure = t.battlefield(P0, "Lure");
    assert!(t.g.attach(lure, Entity::Object(lured)));
    let b1 = t.battlefield(P1, "Hill Giant");
    let b2 = t.battlefield(P2, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(lured, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(blocking(&t, b1).is_empty());
    assert_eq!(blocking(&t, b2), vec![lured]);
}

#[test]
fn blocking_creatures_assign_combat_damage_in_apnap_order() {
    cr!("802.5", "510.1");
    supported("Night Market Guard");
    let mut t = three_players();
    // P1's turn: P2 then P0 assign their blockers' damage.
    let atts: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    let g0 = t.battlefield(P0, "Night Market Guard");
    let g2 = t.battlefield(P2, "Night Market Guard");
    t.set_step(P1, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[
            (atts[0], Entity::Player(P0)),
            (atts[1], Entity::Player(P0)),
            (atts[2], Entity::Player(P2)),
            (atts[3], Entity::Player(P2)),
        ],
    );
    block(&mut t, P2, &[(g2, atts[2]), (g2, atts[3])]);
    block(&mut t, P0, &[(g0, atts[0]), (g0, atts[1])]);
    go_to(&mut t, Step::DeclareBlockers);
    t.script.lock().unwrap().asked.clear();
    go_to(&mut t, Step::CombatDamage);
    let order: Vec<PlayerId> = damage_decisions(&t).into_iter().map(|d| d.0).collect();
    assert_eq!(order, vec![P2, P0]);
}

#[test]
fn combat_damage_isnt_assigned_to_a_player_who_left_the_game() {
    cr!("800.4e", "510.1b");
    let mut t = three_players();
    let x = t.battlefield(P0, "Grizzly Bears");
    let y = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(x, Entity::Player(P1)), (y, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareBlockers);
    t.g.lose_game(P2);
    assert!(!t.g.player(P2).in_game());
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 20);
    assert!(t.g.result.is_none());
}

#[test]
fn a_creature_that_becomes_the_only_attacker_is_attacking_alone() {
    cr!("506.5", "506.4");
    supported("Dream Prowler");
    // Dream Prowler: "can't be blocked as long as it's attacking alone".
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, "Dream Prowler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(prowler, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.can_block(wall, prowler));
    // The Bears leave combat: the Prowler is now attacking alone.
    remove_from_combat(&mut t, bears);
    block(&mut t, P1, &[(wall, prowler)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(!blocked(&t, prowler));
}

#[test]
fn a_creature_attacking_alone_stops_once_another_enters_attacking() {
    cr!("506.5", "508.4");
    let mut t = TestGame::new(2);
    let prowler = t.battlefield(P0, "Dream Prowler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(prowler, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!t.g.can_block(wall, prowler));
    // An effect makes the Bears attacking too: the Prowler no longer attacks alone.
    assert!(mtg_engine::combat::make_attacking(
        &mut t.g,
        bears,
        Entity::Player(P1)
    ));
    t.g.recompute();
    block(&mut t, P1, &[(wall, prowler)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(blocked(&t, prowler));
}

#[test]
fn a_creature_attacking_the_player_while_another_attacks_a_planeswalker_attacks_a_player_alone() {
    cr!("506.6");
    supported("Jiang Yanggu, Alone");
    let mut t = TestGame::new(2);
    // Jiang Yanggu: "Whenever a creature you control attacks a player alone, discard a
    // card, then draw a card. Then put a +1/+1 counter on that creature for each card
    // you've discarded this turn." One card in hand to discard.
    t.battlefield(P0, "Jiang Yanggu, Alone");
    t.hand(P0, "Island");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(bears, Entity::Player(P1)), (giant, Entity::Object(jace))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.counters(bears, mtg_engine::types::counters::PLUS1), 1);
    assert_eq!(t.counters(giant, mtg_engine::types::counters::PLUS1), 0);
}

#[test]
fn two_creatures_attacking_different_players_each_attack_a_player_alone() {
    cr!("506.6");
    let mut t = three_players();
    t.battlefield(P0, "Jiang Yanggu, Alone");
    t.hand(P0, "Island");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let third = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[
            (bears, Entity::Player(P1)),
            (giant, Entity::Player(P2)),
            (third, Entity::Player(P2)),
        ],
    );
    go_to(&mut t, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.counters(bears, mtg_engine::types::counters::PLUS1), 1);
    assert_eq!(t.counters(giant, mtg_engine::types::counters::PLUS1), 0);
    assert_eq!(t.counters(third, mtg_engine::types::counters::PLUS1), 0);
}

fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

#[test]
fn mogg_flunkies_attacks_and_blocks_with_a_teammates_creature() {
    cr!("805.10b", "805.10d", "508.1c", "509.1b");
    ruling!(
        "Mogg Flunkies",
        "In a Two-Headed Giant game, Mogg Flunkies can attack or block with a creature controlled by your teammate"
    );
    let mut t = two_headed_giant();
    let flunkies = t.battlefield(P0, "Mogg Flunkies");
    let mate = t.battlefield(P1, "Grizzly Bears");
    let their_flunkies = t.battlefield(P2, "Mogg Flunkies");
    let their_mate = t.battlefield(P3, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[(flunkies, Entity::Player(P2)), (mate, Entity::Player(P3))],
    );
    go_to(&mut t, Step::DeclareAttackers);
    assert!(t.g.is_attacking(flunkies));
    assert!(t.g.is_attacking(mate));
    // The defending team's combined block: P2's Flunkies with P3's Giant.
    let decider = t.g.primary_player(P2);
    block(
        &mut t,
        decider,
        &[(their_flunkies, flunkies), (their_mate, mate)],
    );
    go_to(&mut t, Step::DeclareBlockers);
    assert_eq!(blocking(&t, their_flunkies), vec![flunkies]);
    assert_eq!(blocking(&t, their_mate), vec![mate]);
}
