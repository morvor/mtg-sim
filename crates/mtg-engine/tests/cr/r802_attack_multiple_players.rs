//! CR 802: the attack multiple players option.

use super::r800_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::GameConfig;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// A four-player Free-for-All game (which uses the attack multiple players option).
fn ffa4() -> TestGame {
    TestGame::with_config(4, GameConfig::free_for_all())
}

#[test]
fn the_active_player_may_attack_several_players_or_just_one() {
    cr!("802.1", "802.3");
    let mut t = ffa4();
    let a = bear(&mut t, P0);
    let b = bear(&mut t, P0);
    // Each attacking creature attacks the player chosen for it.
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P3))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attack_target(&t, a), Some(Entity::Player(P1)));
    assert_eq!(attack_target(&t, b), Some(Entity::Player(P3)));
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (18, 20, 18));
    // A player may also attack only one player.
    let mut t = ffa4();
    let a = bear(&mut t, P0);
    let b = bear(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a, Entity::Player(P2)), (b, Entity::Player(P2))]);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (20, 16, 20));
}

#[test]
fn all_opponents_are_defending_players() {
    cr!("802.2");
    let mut t = ffa4();
    let a = bear(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    let c = t.g.combat.as_ref().unwrap();
    assert_eq!(c.defending_players, vec![P1, P2, P3]);
    // Each opponent, their planeswalkers and battles can be attacked.
    let jace = t.battlefield(P2, "Jace Beleren");
    let targets = targets_of(&mtg_engine::combat::attack_options(&t.g), a);
    for e in [
        Entity::Player(P1),
        Entity::Player(P2),
        Entity::Player(P3),
        Entity::Object(jace),
    ] {
        assert!(targets.contains(&e), "{e:?}");
    }
    // Without the option, the active player chooses one opponent as the defending player.
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            attack_multiple_players: false,
            ..GameConfig::default()
        },
    );
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.set_step(P0, Step::BeginningOfCombat);
    assert_eq!(t.g.combat.as_ref().unwrap().defending_players, vec![P2]);
}

#[test]
fn group_restrictions_apply_to_all_attackers_and_player_ones_to_that_player() {
    cr!("802.3a");
    // Propaganda (P1): "Creatures can't attack you unless their controller pays {2} for
    // each creature they control that's attacking you." Attacking P2 costs nothing.
    let mut t = ffa4();
    t.battlefield(P1, "Propaganda");
    let a = bear(&mut t, P0);
    let b = bear(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a, Entity::Player(P2)), (b, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attacking(&t).len(), 2, "no cost to attack P2");
    // Attacking P1 without the mana: the attack on P1 can't be paid for.
    let mut t = ffa4();
    t.battlefield(P1, "Propaganda");
    let a = bear(&mut t, P0);
    let b = bear(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(attacking(&t).iter().all(|(_, e)| *e != Some(Entity::Player(P1))));
    // Silent Arbiter: "No more than one creature can attack each combat." It doesn't
    // concern a particular player, so it counts the whole group of attackers.
    let mut t = ffa4();
    t.battlefield(P3, "Silent Arbiter");
    let a = bear(&mut t, P0);
    let b = bear(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(attacking(&t).len() < 2, "the declaration as a whole is illegal");
    let mut t = ffa4();
    t.battlefield(P3, "Silent Arbiter");
    let a = bear(&mut t, P0);
    bear(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert_eq!(attacking(&t), vec![(a, Some(Entity::Player(P1)))]);
}

#[test]
fn creatures_in_a_band_cant_attack_different_players() {
    cr!("802.3b");
    let mut t = ffa4();
    let hero = t.battlefield(P0, "Benalish Hero");
    let other = bear(&mut t, P0);
    // P0 wants the Bear in the Hero's band, but they attack different players.
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(hero, Entity::Player(P1)), (other, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareAttackers);
    let c = t.g.combat.as_ref().unwrap();
    assert!(c.attacker(hero).unwrap().band.is_none());
    assert!(c.attacker(other).unwrap().band.is_none());
    // Attacking the same player, they can band.
    let mut t = ffa4();
    let hero = t.battlefield(P0, "Benalish Hero");
    let other = bear(&mut t, P0);
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(hero, Entity::Player(P1)), (other, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    let c = t.g.combat.as_ref().unwrap();
    assert!(c.attacker(hero).unwrap().band.is_some());
    assert_eq!(c.attacker(hero).unwrap().band, c.attacker(other).unwrap().band);
}

#[test]
fn each_defending_player_declares_blockers_in_apnap_order() {
    cr!("802.4", "802.4a");
    let mut t = ffa4();
    let a1 = bear(&mut t, P0);
    let a2 = bear(&mut t, P0);
    let b1 = bear(&mut t, P1);
    let b2 = bear(&mut t, P2);
    t.battlefield(P3, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a1, Entity::Player(P1)), (a2, Entity::Player(P2))]);
    block(&mut t, P1, &[(b1, a1)]);
    block(&mut t, P2, &[(b2, a2)]);
    go_to(&mut t, Step::DeclareBlockers);
    // P1 declares all of their blocks, then P2; P3 isn't being attacked.
    let asked: Vec<(PlayerId, Vec<(ObjectId, Vec<ObjectId>)>)> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::DeclareBlockers { options } => Some((p, options)),
            _ => None,
        })
        .collect();
    assert_eq!(
        asked.iter().map(|(p, _)| *p).collect::<Vec<_>>(),
        vec![P1, P2]
    );
    // Each defending player's creatures can block only creatures attacking that player.
    for (p, options) in &asked {
        let attacker = if *p == P1 { a1 } else { a2 };
        for (_, can_block) in options {
            assert_eq!(can_block, &vec![attacker]);
        }
    }
    assert!(is_blocked(&t, a1) && is_blocked(&t, a2));
}

#[test]
fn block_legality_ignores_other_players_attackers_and_blockers() {
    cr!("802.4b");
    // Silent Arbiter: "No more than one creature can block each combat." Each defending
    // player's block is checked on its own: P1 and P2 may each block with one creature.
    let mut t = ffa4();
    t.battlefield(P3, "Silent Arbiter");
    let a1 = bear(&mut t, P0);
    t.g.objects[a1.0 as usize].tapped = false;
    let b1 = bear(&mut t, P1);
    let b2 = bear(&mut t, P2);
    // P0 attacks with only one creature (Silent Arbiter), so give P0 a second combat's
    // worth: attack P1 with one creature and have a second attacker put onto the
    // battlefield attacking P2.
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a1, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    let a2 = enter_with(&mut t, P0, vanilla("Late Raider", 2, 2), Some(Entity::Player(P2)), None);
    block(&mut t, P1, &[(b1, a1)]);
    block(&mut t, P2, &[(b2, a2)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(is_blocked(&t, a1), "P1's block is legal on its own");
    assert!(is_blocked(&t, a2), "and so is P2's");
    // The restriction does apply within one player's block: P1 can't block with two.
    let mut t = ffa4();
    t.battlefield(P3, "Silent Arbiter");
    let a1 = bear(&mut t, P0);
    let b1 = bear(&mut t, P1);
    let b1b = bear(&mut t, P1);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(a1, Entity::Player(P1))]);
    block(&mut t, P1, &[(b1, a1), (b1b, a1)]);
    go_to(&mut t, Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().unwrap().blockers.len() < 2);
}

#[test]
fn combat_damage_is_assigned_in_apnap_order() {
    cr!("802.5");
    let mut t = ffa4();
    let extra = || {
        custom_card(
            "Wide Guard",
            "Creature — Soldier",
            Some((2, 4)),
            "This creature can block an additional creature each combat.",
        )
    };
    // P0's first attacker is blocked by two of P1's creatures, so P0 assigns its damage;
    // P1's and P2's guards each block two attackers and assign theirs.
    let attackers: Vec<ObjectId> = (0..5).map(|_| bear(&mut t, P0)).collect();
    let p1a = bear(&mut t, P1);
    let p1b = bear(&mut t, P1);
    let g1 = bf(&mut t, P1, extra());
    let g2 = bf(&mut t, P2, extra());
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        &[
            (attackers[0], Entity::Player(P1)),
            (attackers[1], Entity::Player(P1)),
            (attackers[2], Entity::Player(P1)),
            (attackers[3], Entity::Player(P2)),
            (attackers[4], Entity::Player(P2)),
        ],
    );
    block(
        &mut t,
        P1,
        &[
            (p1a, attackers[0]),
            (p1b, attackers[0]),
            (g1, attackers[1]),
            (g1, attackers[2]),
        ],
    );
    block(&mut t, P2, &[(g2, attackers[3]), (g2, attackers[4])]);
    go_to(&mut t, Step::DeclareBlockers);
    t.script.lock().unwrap().asked.clear();
    go_to(&mut t, Step::EndOfCombat);
    let order: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, Decision::AssignCombatDamage { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(order, vec![P0, P1, P2]);
}
