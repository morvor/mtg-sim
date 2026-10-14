//! Rulings batch P215 — first strike (CR 702.7, 510.4) and firebending: what happens
//! between the first-strike and regular combat damage steps, and experience counters.

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 attacks P1 with `attackers`; P1 blocks with `blocks`; the game advances to the
/// first-strike combat damage step and resolves the triggers there.
fn to_first_strike_damage(
    t: &mut TestGame,
    attackers: &[ObjectId],
    blocks: &[(ObjectId, ObjectId)],
) {
    let a: Vec<_> = attackers
        .iter()
        .map(|id| (*id, Entity::Player(P1)))
        .collect();
    attack_with(t, &a);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    t.advance_to(P0, Step::FirstStrikeDamage);
    t.resolve_all();
}

fn finish_combat(t: &mut TestGame) {
    t.advance_to(P0, Step::EndOfCombat);
}

#[test]
fn mirri_gets_its_counter_between_first_strike_and_regular_damage() {
    cr!("510.4", "702.7b", "603.3");
    ruling!(
        "Mirri the Cursed",
        "If Mirri the Cursed is blocking or blocked by a creature that doesn't have first strike or double strike, Mirri will deal combat damage to that creature, then get a +1/+1 counter, then (if the other creature survived) Mirri will be dealt combat damage."
    );
    supported("Mirri the Cursed");
    let mut t = TestGame::new(2);
    let mirri = t.battlefield(P0, "Mirri the Cursed");
    // A 2/4 Giant Spider (reach) survives Mirri's 3 damage.
    let turtle = t.battlefield(P1, "Giant Spider");
    to_first_strike_damage(&mut t, &[mirri], &[(turtle, mirri)]);
    assert_eq!(damage_on(&t, turtle), 3);
    assert_eq!(t.counters(mirri, counters::PLUS1), 1);
    assert_eq!(damage_on(&t, mirri), 0);
    finish_combat(&mut t);
    assert_eq!(damage_on(&t, mirri), 2);
    assert_eq!(t.pt(mirri), (4, 3));
}

#[test]
fn phyrexian_crusaders_minus_counters_shrink_the_blocker_before_it_deals_damage() {
    cr!("510.4", "702.90c", "702.7b");
    ruling!(
        "Phyrexian Crusader",
        "If Phyrexian Crusader blocks or is blocked by a creature without first strike or double strike, the damage dealt by Phyrexian Crusader results in -1/-1 counters that will affect how much damage the other creature assigns (if it’s still on the battlefield)."
    );
    supported("Phyrexian Crusader");
    let mut t = TestGame::new(2);
    let crusader = t.battlefield(P0, "Phyrexian Crusader");
    // (A green 3/3: the Crusader has protection from red and from white.)
    let giant = t.battlefield(P1, "Trained Armodon");
    to_first_strike_damage(&mut t, &[crusader], &[(giant, crusader)]);
    assert_eq!(t.counters(giant, counters::MINUS1), 2);
    assert_eq!(t.pt(giant), (1, 1));
    finish_combat(&mut t);
    // The 3/3, now 1/1, deals 1 damage: the 2/2 Crusader survives.
    assert!(t.on_battlefield(crusader));
    assert_eq!(damage_on(&t, crusader), 1);
}

#[test]
fn losing_first_strike_after_first_strike_damage_means_no_regular_damage() {
    cr!("510.4", "702.7c");
    ruling!(
        "Mite Overseer",
        "If a creature loses first strike after dealing combat damage in the first combat damage step, it won't deal damage in the second combat damage step unless it has double strike."
    );
    ruling!(
        "Stromkirk Captain",
        "If a creature with first strike deals damage in the first combat damage step, but then loses first strike before regular combat damage, it won't deal combat damage a second time (unless it also has double strike)."
    );
    // (lord, the creature it gives first strike to)
    for (lord, name) in [
        ("Mite Overseer", None),
        ("Stromkirk Captain", Some("Vampire Lacerator")),
    ] {
        supported(lord);
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, lord);
        let c = match name {
            Some(n) => t.battlefield(P0, n),
            None => create_token(&mut t, P0, "Soldier"),
        };
        assert!(t.obj_now(c).has_keyword(KeywordKind::FirstStrike), "{lord}");
        let power = t.pt(c).0;
        to_first_strike_damage(&mut t, &[c], &[]);
        assert_eq!(t.life(P1), 20 - power, "{lord}");
        // The lord leaves: the creature loses first strike before regular damage.
        destroy(&mut t, l);
        assert!(!t.obj_now(c).has_keyword(KeywordKind::FirstStrike));
        finish_combat(&mut t);
        assert_eq!(t.life(P1), 20 - power, "{lord}: no second hit");
    }
}

#[test]
fn pit_spawn_exiles_the_creature_it_damages_before_it_deals_damage_back() {
    cr!("510.4", "702.7b", "603.3");
    ruling!(
        "Pit Spawn",
        "Since this card has First Strike it will often deal damage during first strike damage dealing and its ability will exile the creature it damages before that creature can deal damage back."
    );
    supported("Pit Spawn");
    let mut t = TestGame::new(2);
    let spawn = t.battlefield(P0, "Pit Spawn");
    // An indestructible 11/11 survives the 6 damage; the trigger exiles it.
    let colossus = t.battlefield(P1, "Darksteel Colossus");
    to_first_strike_damage(&mut t, &[spawn], &[(colossus, spawn)]);
    assert!(!t.on_battlefield(colossus));
    assert!(t.in_exile("Darksteel Colossus"));
    finish_combat(&mut t);
    assert!(t.on_battlefield(spawn));
    assert_eq!(damage_on(&t, spawn), 0);
}

#[test]
fn dranas_counters_affect_only_regular_combat_damage() {
    cr!("510.4", "702.7b", "510.2");
    ruling!(
        "Drana, Liberator of Malakir",
        "The +1/+1 counter won't change how much damage Drana or any other attacking creature with first strike or double strike deals during that combat damage step. However, the +1/+1 counters that are put on attacking creatures without first strike will affect the damage those creatures deal during the regular combat damage step."
    );
    supported("Drana, Liberator of Malakir");
    let mut t = TestGame::new(2);
    let drana = t.battlefield(P0, "Drana, Liberator of Malakir");
    let knight = t.battlefield(P0, "White Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_first_strike_damage(&mut t, &[drana, knight, bears], &[]);
    // Drana 2 + White Knight 2 in the first-strike step.
    assert_eq!(t.life(P1), 16);
    for id in [drana, knight, bears] {
        assert_eq!(t.counters(id, counters::PLUS1), 1);
    }
    finish_combat(&mut t);
    // Then the Bears, now 3/3, deals 3.
    assert_eq!(t.life(P1), 13);
}

#[test]
fn gearshift_ace_vehicle_keeps_first_strike_even_if_ace_left() {
    cr!("702.122a", "603.10a", "113.7a");
    ruling!(
        "Gearshift Ace",
        "Gearshift Ace's triggered ability triggers when it becomes tapped to activate a crew ability. Even if Gearshift Ace leaves the battlefield before its triggered ability resolves, that Vehicle still gains first strike until end of turn."
    );
    supported("Gearshift Ace");
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let ace = t.battlefield(P0, "Gearshift Ace");
    assert!(crate::r_s04_common::crew(&mut t, P0, copter, &[ace]));
    t.settle();
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, ace);
    t.resolve_all();
    assert!(t.obj_now(copter).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn zuko_experience_counters_go_on_the_player_and_are_all_counted() {
    cr!("122.1", "122.8", "106.4");
    ruling!(
        "Zuko, Firebending Master",
        "The experience counter goes on you, the player, not on Zuko. You will keep that counter even if Zuko, Firebending Master dies."
    );
    ruling!(
        "Zuko, Firebending Master",
        "All experience counters are identical, no matter how you got them. For example, the firebending ability will count experience counters that you got from the last ability, from another ability, from another copy of Zuko, Firebending Master, and so on."
    );
    supported("Zuko, Firebending Master");
    let mut t = TestGame::new(2);
    let zuko = t.battlefield(P0, "Zuko, Firebending Master");
    // Two experience counters from elsewhere.
    t.g.add_counters(Entity::Player(P0), counters::EXPERIENCE, 2, None);
    attack_with(&mut t, &[(zuko, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 2);
    // A spell cast during combat: one more, on the player.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve();
    assert_eq!(
        t.g.player(P0).counters.get(counters::EXPERIENCE).copied(),
        Some(3)
    );
    assert_eq!(t.counters(zuko, counters::EXPERIENCE), 0);
    t.resolve_all();
    destroy(&mut t, zuko);
    assert_eq!(
        t.g.player(P0).counters.get(counters::EXPERIENCE).copied(),
        Some(3)
    );
}
