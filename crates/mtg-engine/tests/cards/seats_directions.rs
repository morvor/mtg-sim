//! Seats and directions (CR 101.4, 103.1, 508.1c, 508.4): the player to your left or
//! right, choosing left or right, and attacking only the nearest opponent in a direction.
//! Grenzo's Rebuttal, Hithlain Rope, Barroom Brawl, Pramikon, Mystic Barrier, Teyo.

use crate::basic_effects_common::assert_supported;
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const LEFT: usize = 0;
const RIGHT: usize = 1;

fn choosers_since(t: &TestGame, from: usize) -> Vec<PlayerId> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect()
}

#[test]
fn grenzos_rebuttal_choices_go_left_in_turn_order_and_all_are_destroyed() {
    cr!("101.4b", "608.2c");
    ruling!(
        "Grenzo's Rebuttal",
        "The choices are made in turn order starting with you. However, all chosen permanents are destroyed simultaneously."
    );
    ruling!("Grenzo's Rebuttal", "The player on your right can choose the Ogre");
    assert_supported("Grenzo's Rebuttal");
    let mut t = TestGame::new(4);
    t.set_step(P0, Step::PrecombatMain);
    for p in [P1, P2, P3] {
        t.battlefield(p, "Mind Stone");
        t.battlefield(p, "Grizzly Bears");
        t.battlefield(p, "Forest");
    }
    let spell = t.hand(P0, "Grenzo's Rebuttal");
    t.lands(P0, "Mountain", 6);
    let from = t.asked().len();
    t.cast(P0, spell).go();
    t.resolve_all();
    // An artifact, a creature and a land each, P0 first, then P1, P2, P3.
    assert_eq!(
        choosers_since(&t, from),
        vec![P0, P0, P0, P1, P1, P1, P2, P2, P2, P3, P3]
    );
    // P1 and P2 lost their three permanents; P0 controls no artifact, so P3 chose only
    // the Ogre (P0's only creature) and a land of P0's.
    for p in [P1, P2] {
        assert_eq!(
            t.g.permanents().filter(|o| o.controller == p).count(),
            0,
            "{p:?}"
        );
    }
    assert!(t.named_on_battlefield("Ogre Token").is_empty());
}

#[test]
fn hithlain_rope_in_a_two_player_game_goes_to_the_other_player() {
    cr!("101.4");
    ruling!("Hithlain Rope", "the other player is the player to your right");
    let mut t = TestGame::new(2);
    let rope = t.battlefield(P0, "Hithlain Rope");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, rope, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(rope).controller, P1);
}

#[test]
fn hithlain_rope_may_go_to_a_teammate() {
    cr!("101.4");
    ruling!(
        "Hithlain Rope",
        "In some formats, such as Two-Headed Giant, the player to your right might be a teammate"
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            teams: Some(vec![0, 1, 1, 0]),
            ..Default::default()
        },
    );
    let rope = t.battlefield(P0, "Hithlain Rope");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, rope, 1, &[]).unwrap();
    t.resolve_all();
    // P3, P0's teammate, sits to P0's right.
    assert_eq!(t.obj_now(rope).controller, P3);
}

#[test]
fn barroom_brawl_fights_a_creature_of_the_opponent_to_your_left() {
    cr!("115.1", "707.10");
    ruling!("Barroom Brawl", "the other player is the opponent to your left");
    assert_supported("Barroom Brawl");
    // Two players: the opponent sits on both sides.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Barroom Brawl");
    t.lands(P0, "Forest", 2);
    t.cast(P0, spell).target(wurm).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn barroom_brawl_in_multiplayer_targets_only_the_left_opponent_who_may_copy_it() {
    cr!("115.1", "707.10c");
    ruling!(
        "Barroom Brawl",
        "so as long as there are legal targets available and the copies keep successfully resolving, the brawl can continue"
    );
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    let p1_giant = t.battlefield(P1, "Hill Giant");
    let p2_bears = t.battlefield(P2, "Grizzly Bears");
    let spell = t.hand(P0, "Barroom Brawl");
    t.lands(P0, "Forest", 2);
    // P1 copies it: its Hill Giant fights P2's Grizzly Bears (P2 sits to P1's left).
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Object(p1_giant)]);
    t.answer_targets(P1, &[Entity::Object(p2_bears)]);
    let from = t.asked().len();
    t.cast(P0, spell).target(wurm).target(p1_bears).go();
    let candidates: Vec<Entity> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, text, .. }
                if text.contains("opponent to your left") =>
            {
                Some(candidates.clone())
            }
            _ => None,
        })
        .flatten()
        .collect();
    assert!(candidates.contains(&Entity::Object(p1_bears)));
    assert!(!candidates.contains(&Entity::Object(p2_bears)));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P2, "Grizzly Bears"));
    assert!(t.on_battlefield(p1_giant));
}

/// What `c` could be declared as attacking now (with the attack multiple players option,
/// every opponent is a defending player).
fn attack_targets(t: &mut TestGame, c: ObjectId) -> Vec<Entity> {
    t.g.recompute();
    mtg_engine::combat::attack_options(&t.g)
        .into_iter()
        .find(|(x, _)| *x == c)
        .map(|(_, v)| v)
        .unwrap_or_default()
}

fn four_players() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            attack_multiple_players: true,
            ..Default::default()
        },
    )
}

#[test]
fn pramikon_lets_players_attack_only_the_nearest_opponent_in_the_chosen_direction() {
    cr!("508.1c");
    assert_supported("Pramikon, Sky Rampart");
    for (dir, only) in [(LEFT, P1), (RIGHT, P3)] {
        let mut t = four_players();
        t.answer(P0, DecisionKind::Option, Answer::Index(dir));
        t.enter(P0, "Pramikon, Sky Rampart");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let pw = t.battlefield(only, "Jace Beleren");
        let other_pw = t.battlefield(P2, "Jace Beleren");
        t.advance_to(P0, Step::BeginningOfCombat);
        let targets = attack_targets(&mut t, bears);
        assert!(targets.contains(&Entity::Player(only)), "{dir}");
        assert!(targets.contains(&Entity::Object(pw)), "{dir}");
        assert!(!targets.contains(&Entity::Object(other_pw)));
        assert_eq!(
            targets
                .iter()
                .filter(|e| matches!(e, Entity::Player(_)))
                .count(),
            1
        );
    }
}

#[test]
fn two_pramikons_with_different_directions_stop_all_attacks_with_three_or_more_players() {
    cr!("508.1c");
    ruling!(
        "Pramikon, Sky Rampart",
        "disagree on which direction three or more players must attack"
    );
    let mut t = four_players();
    t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
    t.enter(P0, "Pramikon, Sky Rampart");
    t.answer(P1, DecisionKind::Option, Answer::Index(RIGHT));
    t.enter(P1, "Pramikon, Sky Rampart");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(attack_targets(&mut t, bears).is_empty());
}

#[test]
fn two_mystic_barriers_in_a_two_player_game_still_allow_attacking_the_opponent() {
    cr!("508.1c");
    ruling!(
        "Mystic Barrier",
        "the opponent seated nearest you in both directions"
    );
    assert_supported("Mystic Barrier");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
    t.enter(P0, "Mystic Barrier");
    t.answer(P0, DecisionKind::Option, Answer::Index(RIGHT));
    t.enter(P0, "Mystic Barrier");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(attack_targets(&mut t, bears).contains(&Entity::Player(P1)));
}

#[test]
fn mystic_barrier_uses_the_last_chosen_direction() {
    cr!("508.1c");
    let mut t = four_players();
    t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
    t.enter(P0, "Mystic Barrier");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    let targets = attack_targets(&mut t, bears);
    assert!(targets.contains(&Entity::Player(P1)) && !targets.contains(&Entity::Player(P3)));
    // At P0's next upkeep the direction is chosen again.
    t.answer(P0, DecisionKind::Option, Answer::Index(RIGHT));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    t.advance_to(P0, Step::BeginningOfCombat);
    let targets = attack_targets(&mut t, bears);
    assert!(targets.contains(&Entity::Player(P3)) && !targets.contains(&Entity::Player(P1)));
}

#[test]
fn mystic_barrier_doesnt_affect_creatures_entering_attacking() {
    cr!("508.4", "508.4c");
    ruling!(
        "Mystic Barrier",
        "doesn’t affect creatures that enter the battlefield attacking"
    );
    ruling!(
        "Teyo, Geometric Tactician",
        "last ability doesn't affect creatures that enter the battlefield attacking"
    );
    for teyo in [false, true] {
        let mut t = four_players();
        t.set_step(P0, Step::PrecombatMain);
        t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
        if teyo {
            let teyo = t.battlefield(P0, "Teyo, Geometric Tactician");
            t.g.objects[teyo.0 as usize]
                .counters
                .insert("loyalty".into(), 5);
            t.activate(P0, teyo, 1, &[]).unwrap();
        } else {
            t.enter(P0, "Mystic Barrier");
        }
        t.resolve_all();
        let hero = t.battlefield(P0, "Hero of Bladehold");
        t.advance_to(P0, Step::BeginningOfCombat);
        // Hero of Bladehold attacks P1; its Soldier tokens enter attacking P2.
        t.answer_choose(P0, &[Entity::Player(P2)]);
        t.answer_choose(P0, &[Entity::Player(P2)]);
        t.attack(&[(hero, Entity::Player(P1))], &[]);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Soldier Token").len(), 2);
        // The Soldiers dealt their combat damage to P2.
        assert!(t.life(P2) < 20, "teyo: {teyo}");
        assert_eq!(t.life(P1), 17);
    }
}

#[test]
fn teyo_restricts_attacks_until_your_next_turn() {
    cr!("508.1c", "611.2a");
    ruling!(
        "Teyo, Geometric Tactician",
        "disagree on which direction three or more players may attack"
    );
    assert_supported("Teyo, Geometric Tactician");
    let mut t = four_players();
    let teyo = t.battlefield(P0, "Teyo, Geometric Tactician");
    t.g.objects[teyo.0 as usize]
        .counters
        .insert("loyalty".into(), 5);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Option, Answer::Index(RIGHT));
    t.activate(P0, teyo, 1, &[]).unwrap();
    t.resolve_all();
    // P1's turn: P1's nearest opponent to the right is P0.
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::BeginningOfCombat);
    let targets = attack_targets(&mut t, p1_bears);
    assert!(targets.contains(&Entity::Player(P0)));
    assert!(!targets.contains(&Entity::Player(P2)) && !targets.contains(&Entity::Player(P3)));
    // A Pramikon choosing left disagrees: no one can attack.
    t.answer(P1, DecisionKind::Option, Answer::Index(LEFT));
    t.enter(P1, "Pramikon, Sky Rampart");
    assert!(attack_targets(&mut t, p1_bears).is_empty());
}

#[test]
fn teyos_restriction_ends_at_your_next_turn() {
    cr!("611.2a");
    let mut t = four_players();
    let teyo = t.battlefield(P0, "Teyo, Geometric Tactician");
    t.g.objects[teyo.0 as usize]
        .counters
        .insert("loyalty".into(), 5);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
    t.activate(P0, teyo, 1, &[]).unwrap();
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    let targets = attack_targets(&mut t, bears);
    assert!(!targets.contains(&Entity::Player(P2)));
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(attack_targets(&mut t, bears).contains(&Entity::Player(P2)));
}

#[test]
fn teyos_restriction_outlasts_teyo() {
    cr!("611.2a", "508.1c");
    // The direction stays the one chosen for the ability, even after Teyo is gone.
    let mut t = four_players();
    let teyo = t.battlefield(P0, "Teyo, Geometric Tactician");
    t.g.objects[teyo.0 as usize]
        .counters
        .insert("loyalty".into(), 5);
    t.set_step(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
    t.activate(P0, teyo, 1, &[]).unwrap();
    t.resolve_all();
    t.g.destroy(teyo, None);
    t.settle();
    assert!(!t.on_battlefield(teyo));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    let targets = attack_targets(&mut t, bears);
    assert!(targets.contains(&Entity::Player(P1)));
    assert!(!targets.contains(&Entity::Player(P2)) && !targets.contains(&Entity::Player(P3)));
}

#[test]
fn attack_direction_restrictions_dont_affect_targeting() {
    cr!("508.1c", "115.1");
    ruling!(
        "Teyo, Geometric Tactician",
        "It doesn't affect what players may be targeted by spells or abilities"
    );
    ruling!(
        "Mystic Barrier",
        "affects only what players and planeswalkers each player may attack with creatures they control"
    );
    for teyo in [false, true] {
        let mut t = four_players();
        t.set_step(P0, Step::PrecombatMain);
        t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
        if teyo {
            let teyo = t.battlefield(P0, "Teyo, Geometric Tactician");
            t.g.objects[teyo.0 as usize]
                .counters
                .insert("loyalty".into(), 5);
            t.activate(P0, teyo, 1, &[]).unwrap();
        } else {
            t.enter(P0, "Mystic Barrier");
        }
        t.resolve_all();
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.objects[bears.0 as usize].summoning_sick = false;
        // P2 can't be attacked, but can be targeted.
        assert!(!attack_targets(&mut t, bears).contains(&Entity::Player(P2)));
        let bolt = t.hand(P0, "Lightning Bolt");
        t.lands(P0, "Mountain", 1);
        t.cast(P0, bolt).target(P2).go();
        t.resolve_all();
        assert_eq!(t.life(P2), 17, "teyo: {teyo}");
    }
}

#[test]
fn pramikon_without_a_chosen_direction_restricts_nothing() {
    cr!("508.1c");
    ruling!(
        "Pramikon, Sky Rampart",
        "If no choice was made for Pramikon"
    );
    let mut t = four_players();
    // Put onto the battlefield directly: no direction was chosen as it entered.
    let pramikon = t.battlefield(P0, "Pramikon, Sky Rampart");
    assert!(t.g.obj(pramikon).choices.text.is_none());
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    let targets = attack_targets(&mut t, bears);
    for p in [P1, P2, P3] {
        assert!(targets.contains(&Entity::Player(p)), "{p:?}");
    }
}

#[test]
fn mystic_barrier_skips_teammates_to_find_the_nearest_opponent() {
    cr!("508.1c", "102.3");
    ruling!(
        "Mystic Barrier",
        "you may need to disregard teammates sitting between you and the opponent seated nearest you"
    );
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            attack_multiple_players: true,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    t.answer(P0, DecisionKind::Option, Answer::Index(LEFT));
    t.enter(P0, "Mystic Barrier");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    // P1 (to P0's left) is a teammate: the nearest opponent to the left is P2.
    let targets = attack_targets(&mut t, bears);
    assert!(targets.contains(&Entity::Player(P2)));
    assert!(!targets.contains(&Entity::Player(P3)));
}

#[test]
fn grenzos_rebuttal_choices_arent_targeted_and_one_permanent_may_fill_two_kinds() {
    cr!("101.4b", "702.16b");
    ruling!(
        "Grenzo's Rebuttal",
        "If a permanent has more than one of the listed types, it can be chosen for any of them"
    );
    ruling!(
        "Grenzo's Rebuttal",
        "None of the chosen permanents are targeted. A player can choose a creature with protection from red"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let knight = t.battlefield(P1, "Vulshok Refugee");
    let forest = t.battlefield(P1, "Forest");
    let spell = t.hand(P0, "Grenzo's Rebuttal");
    t.lands(P0, "Mountain", 6);
    // P0 chooses the Ornithopter as both the artifact and the creature.
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(thopter) && !t.on_battlefield(forest));
    assert!(t.on_battlefield(bears) && t.on_battlefield(knight));

    // A red spell's choice can be a creature with protection from red.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P1, "Mind Stone");
    let knight = t.battlefield(P1, "Vulshok Refugee");
    t.battlefield(P1, "Forest");
    let spell = t.hand(P0, "Grenzo's Rebuttal");
    t.lands(P0, "Mountain", 6);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t.on_battlefield(knight));
}

#[test]
fn barroom_brawl_with_both_targets_gone_makes_no_copy() {
    cr!("608.2b");
    ruling!(
        "Barroom Brawl",
        "If both targets are illegal as Barroom Brawl attempts to resolve, it does nothing. No copy will be created."
    );
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Barroom Brawl");
    t.lands(P0, "Forest", 2);
    t.answer_yes(P1, true);
    t.cast(P0, spell).target(wurm).target(bears).go();
    t.g.destroy(wurm, None);
    t.g.destroy(bears, None);
    t.settle();
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.asked()[from..].iter().all(|(p, _)| *p != P1));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn barroom_brawl_with_one_target_gone_deals_no_damage_but_may_be_copied() {
    cr!("608.2b", "707.10c");
    ruling!(
        "Barroom Brawl",
        "neither creature will deal damage. However, the opponent to your left may still choose to copy"
    );
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let p2_bears = t.battlefield(P2, "Grizzly Bears");
    let spell = t.hand(P0, "Barroom Brawl");
    t.lands(P0, "Forest", 2);
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Object(giant)]);
    t.answer_targets(P1, &[Entity::Object(p2_bears)]);
    t.cast(P0, spell).target(wurm).target(bears).go();
    t.g.destroy(wurm, None);
    t.settle();
    t.resolve_all();
    // The original dealt no damage; P1's copy had its Hill Giant fight P2's Grizzly Bears.
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert!(!t.on_battlefield(p2_bears));
}
