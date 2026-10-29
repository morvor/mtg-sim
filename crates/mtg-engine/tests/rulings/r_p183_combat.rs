//! Rulings batch P183 — typal combat abilities: creatures put onto the battlefield
//! attacking (CR 508.4, 508.3a), attack restrictions checked only as attackers are
//! declared (CR 508.1c, 506.4), "attacks and isn't blocked" (CR 509.1h), and a combat
//! damage trigger (CR 510.2, 603.2).

use crate::r_p009_common::{kill, obj, pl};
use crate::r_p189_common::{stack_triggers_from, triggers_of};
use crate::r_s01_common::{attack_with, asked_since, block_and_finish, supported, tokens};
use crate::r_s02_common::can_attack;
use crate::r_s12_common::attack_target;
use crate::r_s19_common::add_lore;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0's Kaalia attacks `at` with `in_hand` in P0's hand; P0 says yes to the trigger and
/// picks that card. Returns (Kaalia, the card). The trigger is on the stack.
fn kaalia_attacks(t: &mut TestGame, at: Entity, in_hand: &str) -> (ObjectId, ObjectId) {
    let k = t.battlefield(P0, "Kaalia of the Vast");
    let c = t.hand(P0, in_hand);
    attack_with(t, &[(k, at)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(c)]);
    (k, c)
}

#[test]
fn kaalias_creature_ignores_cant_attack_effects() {
    cr!("508.4", "508.1c");
    ruling!(
        "Kaalia of the Vast",
        "Any effects that say that the Angel, Demon, or Dragon creature can't attack (such as that of Propaganda or if the creature has defender) affect only the declaration of attackers. They won't stop the creature from entering the battlefield attacking."
    );
    supported("Kaalia of the Vast");
    supported("Dragon Egg");
    // Dragon Egg has defender; P1's Propaganda would make it cost {2} to attack.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Propaganda");
    t.lands(P0, "Wastes", 2);
    let (_, egg) = kaalia_attacks(&mut t, pl(P1), "Dragon Egg");
    t.resolve_all();
    let egg = t.g.current(egg);
    assert!(t.on_battlefield(egg));
    assert!(t.g.is_attacking(egg));
    assert!(t.obj_now(egg).tapped);
    assert_eq!(attack_target(&t, egg), Some(pl(P1)));
}

#[test]
fn kaalias_creature_attacks_the_player_kaalia_attacked() {
    cr!("508.4");
    ruling!(
        "Kaalia of the Vast",
        "If Kaalia attacks an opponent, the creature enters attacking that player even if Kaalia is somehow attacking a different player or planeswalker as its triggered ability resolves."
    );
    supported("Misleading Signpost");
    let mut t = TestGame::new(3);
    let (k, angel) = kaalia_attacks(&mut t, pl(P1), "Serra Angel");
    // In response, Misleading Signpost reselects Kaalia to attack P2.
    t.clear_answers();
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[obj(k)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[pl(P2)]);
    // Then Kaalia's trigger: put Serra Angel onto the battlefield.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(angel)]);
    let sp = t.hand(P0, "Misleading Signpost");
    t.cast(P0, sp).go();
    t.resolve();
    t.resolve();
    assert_eq!(attack_target(&t, k), Some(pl(P2)));
    t.resolve_all();
    let angel = t.g.current(angel);
    assert!(t.g.is_attacking(angel));
    assert_eq!(attack_target(&t, angel), Some(pl(P1)));
}

#[test]
fn kaalias_creature_isnt_attacking_if_that_opponent_left_the_game() {
    cr!("508.4a");
    ruling!(
        "Kaalia of the Vast",
        "If the opponent Kaalia attacked is no longer in the game when its ability resolves, you may put an Angel, Demon, or Dragon creature card onto the battlefield tapped, but it won't be attacking anyone and it won't be an attacking creature."
    );
    let mut t = TestGame::new(3);
    let (_, angel) = kaalia_attacks(&mut t, pl(P1), "Serra Angel");
    t.g.lose_game(P1);
    t.settle();
    t.resolve_all();
    let angel = t.g.current(angel);
    assert!(t.on_battlefield(angel));
    assert!(t.obj_now(angel).tapped);
    assert!(!t.g.is_attacking(angel));
}

#[test]
fn kaalia_doesnt_trigger_attacking_a_planeswalker_or_battle() {
    cr!("508.3a");
    ruling!(
        "Kaalia of the Vast",
        "Kaalia's ability doesn't trigger if it attacks a planeswalker or battle."
    );
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    let k = t.battlefield(P0, "Kaalia of the Vast");
    attack_with(&mut t, &[(k, obj(jace))]);
    assert_eq!(triggers_of(&t, k), 0);
    // A battle P0 owns, protected by P1.
    let mut t = TestGame::new(2);
    t.answer_choose(P0, &[pl(P1)]);
    let battle = t.enter(P0, "Invasion of Segovia");
    t.resolve_all();
    let k = t.battlefield(P0, "Kaalia of the Vast");
    attack_with(&mut t, &[(k, obj(battle))]);
    assert_eq!(attack_target(&t, k), Some(obj(battle)));
    assert_eq!(triggers_of(&t, k), 0);
}

#[test]
fn war_falcon_checks_for_a_knight_or_soldier_only_as_attackers_are_declared() {
    cr!("508.1c", "506.4");
    ruling!(
        "War Falcon",
        "If an effect makes War Falcon a Knight or a Soldier, it will be able to attack even if it's the only creature you control."
    );
    ruling!(
        "War Falcon",
        "To attack with War Falcon, you must control a Knight or a Soldier when you declare attackers. Once War Falcon is attacking, it won't be removed from combat if at any point you no longer control a Knight or Soldier."
    );
    supported("War Falcon");
    let mut t = TestGame::new(2);
    let falcon = t.battlefield(P0, "War Falcon");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, falcon));
    choose_creature_type(&mut t, P0, "Knight");
    t.enter(P0, "Arcane Adaptation");
    assert!(can_attack(&mut t, falcon));
    // With a Soldier that then leaves: still attacking, and the damage is dealt.
    let mut t = TestGame::new(2);
    let falcon = t.battlefield(P0, "War Falcon");
    let soldier = t.battlefield(P0, "Elite Vanguard");
    attack_with(&mut t, &[(falcon, pl(P1))]);
    assert!(t.g.is_attacking(falcon));
    kill(&mut t, soldier);
    assert!(t.g.is_attacking(falcon));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn stinkdrinker_bandit_bonus_applies_even_if_the_rogue_becomes_blocked() {
    cr!("509.1h");
    ruling!(
        "Stinkdrinker Bandit",
        "This ability triggers after blockers are declared during the declare blockers step. Each attacking Rogue you control gets +2/+1 if no creatures were assigned to block it. That creature will get the bonus even if an effect causes it to become blocked."
    );
    supported("Stinkdrinker Bandit");
    supported("Curtain of Light");
    let mut t = TestGame::new(2);
    let bandit = t.battlefield(P0, "Stinkdrinker Bandit");
    let rogue = t.battlefield(P0, "Bane Alley Blackguard");
    let blocked = t.battlefield(P0, "Bane Alley Blackguard");
    let wall = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(rogue, pl(P1)), (blocked, pl(P1))]);
    assert_eq!(triggers_of(&t, bandit), 0, "not on attacking");
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(wall, blocked)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    // One trigger, for the unblocked Rogue.
    assert_eq!(stack_triggers_from(&t, bandit).len(), 1);
    // In response, Curtain of Light makes it blocked.
    cast_new(&mut t, P0, "Curtain of Light", &[obj(rogue)]);
    t.resolve();
    assert!(t.g.combat.as_ref().unwrap().is_blocked(rogue));
    t.resolve_all();
    assert_eq!(t.pt(rogue), (1 + 2, 3 + 1));
    assert_eq!(t.pt(blocked), (1, 3));
}

#[test]
fn aphelias_trigger_resolves_after_combat_damage() {
    cr!("510.2", "603.3", "107.1a");
    ruling!(
        "Aphelia, Viper Whisperer",
        "The triggered ability created by Aphelia's last ability triggers and resolves after combat damage is dealt. For example, if Gorgons and/or Snakes you control deal a total of 3 combat damage to a player with 12 life, combat damage will reduce that player's life total to 9. Then the triggered ability will cause the player to lose 5 life, leaving the player at 4."
    );
    supported("Aphelia, Viper Whisperer");
    let mut t = TestGame::new(2);
    t.g.players[1].life = 12;
    let aphelia = t.battlefield(P0, "Aphelia, Viper Whisperer");
    let viper = t.battlefield(P0, "Ambush Viper");
    t.lands(P0, "Swamp", 5);
    t.activate(P0, aphelia, 0, &[]).unwrap();
    t.resolve_all();
    // (Aphelia's attack trigger: don't pay.)
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(aphelia, pl(P1)), (viper, pl(P1))]);
    t.resolve_all();
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 9);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 4);
}

/// Origin of the Hidden Ones on P0's battlefield with chapter III resolved, and one of its
/// Assassin tokens from chapter II. Returns the tokens.
fn origin_iii(t: &mut TestGame) -> Vec<ObjectId> {
    let saga = t.battlefield(P0, "Origin of the Hidden Ones");
    t.answer_targets(P0, &[pl(P1)]);
    add_lore(t, saga, 1);
    t.resolve_all();
    add_lore(t, saga, 1);
    t.resolve_all();
    let assassins = tokens(t, P0);
    assert_eq!(assassins.len(), 2);
    for a in &assassins {
        t.g.objects[a.0 as usize].entered_turn = 0;
        t.g.objects[a.0 as usize].summoning_sick = false;
    }
    add_lore(t, saga, 1);
    t.resolve_all();
    assassins
}

#[test]
fn origins_tokens_enter_attacking_without_being_declared() {
    cr!("508.4", "508.3a");
    ruling!(
        "Origin of the Hidden Ones",
        "Although the tokens created by the third chapter ability enter the battlefield attacking, they were never declared as attacking creatures. Abilities that trigger whenever a creature attacks won't trigger when they enter the battlefield attacking."
    );
    supported("Origin of the Hidden Ones");
    let mut t = TestGame::new(2);
    let a = origin_iii(&mut t);
    attack_with(&mut t, &[(a[0], pl(P1))]);
    t.resolve_all();
    // One new attacking token, which didn't make the Saga's ability trigger again.
    let all = tokens(&t, P0);
    assert_eq!(all.len(), 3);
    assert!(all.iter().filter(|x| t.g.is_attacking(**x)).count() == 2);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn origins_tokens_can_attack_whatever_you_choose() {
    cr!("508.4", "508.4a");
    ruling!(
        "Origin of the Hidden Ones",
        "You choose which player, planeswalker, or battle the Assassin tokens are attacking. Each Assassin token can enter attacking a different player, planeswalker, or battle, and they don't need to be the same ones that the Assassins you control are attacking."
    );
    let mut t = TestGame::new(3);
    let jace = t.battlefield(P2, "Jace Beleren");
    let a = origin_iii(&mut t);
    let from = t.asked().len();
    // Both Assassins attack P1; the two new tokens attack P2 and Jace.
    t.answer(
        P0,
        DecisionKind::Entities,
        mtg_engine::decision::Answer::Entities(vec![pl(P2)]),
    );
    t.answer(
        P0,
        DecisionKind::Entities,
        mtg_engine::decision::Answer::Entities(vec![obj(jace)]),
    );
    attack_with(&mut t, &[(a[0], pl(P1)), (a[1], pl(P1))]);
    t.resolve_all();
    let choices: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(choices.len(), 2);
    assert!(choices[0].contains(&pl(P2)) && choices[0].contains(&obj(jace)));
    let new: Vec<ObjectId> = tokens(&t, P0)
        .into_iter()
        .filter(|x| !a.contains(x))
        .collect();
    assert_eq!(new.len(), 2);
    let mut targets: Vec<Entity> = new.iter().filter_map(|x| attack_target(&t, *x)).collect();
    targets.sort();
    let mut want = vec![pl(P2), obj(jace)];
    want.sort();
    assert_eq!(targets, want);
}
