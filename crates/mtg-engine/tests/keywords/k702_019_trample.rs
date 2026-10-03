//! CR 702.19 Trample and trample over planeswalkers.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Sets a planeswalker's loyalty.
fn set_loyalty(t: &mut TestGame, pw: ObjectId, n: u32) {
    let have = t.counters(pw, "loyalty");
    if have > n {
        t.g.remove_counters(Entity::Object(pw), "loyalty", have - n);
    } else if n > have {
        t.g.add_counters(Entity::Object(pw), "loyalty", n - have, None);
    }
    assert_eq!(t.counters(pw, "loyalty"), n);
}

#[test]
fn trample_assigns_excess_damage_to_the_player() {
    cr!("702.19", "702.19a", "702.19b");
    assert_supported("Colossal Dreadmaw");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(bears, maw)]);
    // By default: lethal damage to the blocker, the rest to the player.
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 16);
    let d = damage_decisions(&t);
    assert_eq!(d.len(), 1);
    assert_eq!(
        d[0].2,
        vec![Entity::Object(bears), Entity::Player(P1)],
        "the controller may assign among the blocker and the player"
    );
}

#[test]
fn trample_excess_beyond_lethal_may_go_to_either() {
    cr!("702.19b");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Excess may stay on the blocker.
    assign_damage(&mut t, P0, &[4, 2]);
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(bears, maw)]);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn trample_cant_assign_to_the_player_without_lethal_to_blockers() {
    cr!("702.19b");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let giant = t.battlefield(P1, "Hill Giant");
    // 2 to a 3/3 isn't lethal: the player can't be assigned damage. The illegal
    // assignment is replaced by the default.
    assign_damage(&mut t, P0, &[2, 4]);
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, maw)]);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn trample_need_not_assign_lethal_to_all_blockers() {
    cr!("702.19b");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Hill Giant");
    // All damage to the first blocker, none to the second: legal, but then none to the
    // player.
    assign_damage(&mut t, P0, &[6, 0, 0]);
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(b1, maw), (b2, maw)]);
    assert!(!t.on_battlefield(b1));
    assert!(t.on_battlefield(b2));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn lethal_damage_accounts_for_damage_already_marked() {
    cr!("702.19b");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(giant, maw)]);
    // 2 damage marked on the 3/3 before combat damage: 1 more is lethal.
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 15);
}

#[test]
fn deathtouch_makes_one_damage_lethal_for_trample() {
    cr!("702.19b", "702.2c");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let collar = t.battlefield(P0, "Basilisk Collar");
    t.g.attach(collar, Entity::Object(maw));
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, maw)]);
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 15);
}

#[test]
fn lethal_damage_accounts_for_other_creatures_assigning_in_the_same_step() {
    cr!("702.19b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let guard = t.battlefield(P1, "Palace Guard");
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (maw, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(guard, bears), (guard, maw)]);
    // The bears assign 2 to the 1/4 guard, so 2 more from the Dreadmaw is lethal.
    let d = damage_decisions(&t);
    let maw_decision = d.iter().find(|(_, c, _)| *c == maw).unwrap();
    assert_eq!(maw_decision.2.len(), 2);
    assert!(!t.on_battlefield(guard));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn deathtouch_damage_assigned_by_another_attacker_counts_as_lethal() {
    cr!("702.19b", "702.2c");
    assert_supported("Typhoid Rats");
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Typhoid Rats");
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let guard = t.battlefield(P1, "Palace Guard");
    attack_with(
        &mut t,
        &[(rats, Entity::Player(P1)), (maw, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(guard, rats), (guard, maw)]);
    // The rats' 1 deathtouch damage is already lethal for the 1/4 guard, so all of the
    // Dreadmaw's damage can go to the player.
    assert!(!t.on_battlefield(guard));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn lethal_damage_ignores_effects_that_would_prevent_it() {
    cr!("702.19b");
    assert_supported("Vodalian Zombie");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let zombie = t.battlefield(P1, "Vodalian Zombie");
    // Protection from green will prevent the damage, but 2 must still be assigned to the
    // 2/2 before any goes to the player: this all-to-the-player assignment is illegal
    // and the default (2 to the blocker, 4 to the player) is used instead.
    assign_damage(&mut t, P0, &[0, 6]);
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(zombie, maw)]);
    assert!(t.on_battlefield(zombie));
    assert_eq!(t.obj_now(zombie).damage, 0);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn trample_has_no_effect_when_blocking() {
    cr!("702.19a");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    block_and_finish(&mut t, P0, &[(maw, bears)]);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn trample_has_no_effect_on_noncombat_damage() {
    cr!("702.19a");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey)
        .target(maw)
        .target(bears)
        .go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn blocked_trampler_with_no_blockers_left_hits_the_player() {
    cr!("702.19d");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let hill = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(maw, Entity::Player(P1)), (hill, Entity::Player(P1))],
    );
    declare_blocks(&mut t, P1, &[(b1, maw), (b2, hill)]);
    t.lands(P0, "Mountain", 2);
    for b in [b1, b2] {
        let bolt = t.hand(P0, "Lightning Bolt");
        t.cast(P0, bolt).target(b).go();
        t.resolve_all();
    }
    assert!(!t.on_battlefield(b1) && !t.on_battlefield(b2));
    t.advance_to(P0, Step::EndOfCombat);
    // Only the trampler's damage is assigned (all of it, to the player); the blocked
    // Hill Giant without trample deals none.
    assert_eq!(t.life(P1), 14);
}

#[test]
fn trample_over_planeswalkers_assigns_excess_to_the_controller() {
    cr!("702.19c");
    ruling!(
        "Thrasta, Tempest's Roar",
        "Trample and trample over planeswalkers can both apply during the same combat"
    );
    let mut t = TestGame::new(2);
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    assert!(mtg_engine::kw::trample::over_planeswalkers(&t.g, thrasta));
    let jace = t.battlefield(P1, "Jace Beleren");
    set_loyalty(&mut t, jace, 2);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(thrasta, Entity::Object(jace))]);
    block_and_finish(&mut t, P1, &[(giant, thrasta)]);
    // 3 to the 3/3 blocker, 2 to the planeswalker, 2 to its controller.
    let d = damage_decisions(&t);
    assert_eq!(
        d[0].2,
        vec![Entity::Object(giant), Entity::Object(jace), Entity::Player(P1)]
    );
    assert!(!t.on_battlefield(giant));
    assert!(!t.on_battlefield(jace));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn trample_over_planeswalkers_needs_the_loyalty_assigned_first() {
    cr!("702.19c");
    let mut t = TestGame::new(2);
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    let jace = t.battlefield(P1, "Jace Beleren");
    set_loyalty(&mut t, jace, 2);
    let giant = t.battlefield(P1, "Hill Giant");
    // 1 to the planeswalker isn't enough to assign any to its controller: illegal, so the
    // default (3, 2, 2) is used.
    assign_damage(&mut t, P0, &[3, 1, 3]);
    attack_with(&mut t, &[(thrasta, Entity::Object(jace))]);
    block_and_finish(&mut t, P1, &[(giant, thrasta)]);
    assert!(!t.on_battlefield(jace));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn trample_over_planeswalkers_may_keep_damage_on_the_planeswalker() {
    cr!("702.19c");
    let mut t = TestGame::new(2);
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    let jace = t.battlefield(P1, "Jace Beleren");
    set_loyalty(&mut t, jace, 2);
    assign_damage(&mut t, P0, &[7, 0]);
    attack_with(&mut t, &[(thrasta, Entity::Object(jace))]);
    block_and_finish(&mut t, P1, &[]);
    assert!(!t.on_battlefield(jace));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn trample_over_planeswalkers_counts_other_damage_to_the_planeswalker() {
    cr!("702.19c");
    ruling!(
        "Thrasta, Tempest's Roar",
        "Trample over planeswalkers takes into account any other damage being assigned to the planeswalker at the same time"
    );
    let mut t = TestGame::new(2);
    let cub = t.battlefield(P0, "Bear Cub");
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    let jace = t.battlefield(P1, "Jace Beleren");
    set_loyalty(&mut t, jace, 7);
    attack_with(
        &mut t,
        &[(cub, Entity::Object(jace)), (thrasta, Entity::Object(jace))],
    );
    block_and_finish(&mut t, P1, &[]);
    // The cub assigns 2 to Jace; Thrasta assigns 5 to Jace and 2 to P1.
    assert!(!t.on_battlefield(jace));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn trample_over_planeswalkers_when_the_planeswalker_is_removed() {
    cr!("702.19e");
    let mut t = TestGame::new(2);
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    let jace = t.battlefield(P1, "Jace Beleren");
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(thrasta, Entity::Object(jace))]);
    declare_blocks(&mut t, P1, &[(giant, thrasta)]);
    // The planeswalker leaves combat (and the battlefield).
    t.g.destroy(jace, None);
    t.resolve_all();
    // Thrasta isn't attacking the player.
    let target = t.g.combat.as_ref().unwrap().attacker(thrasta).unwrap().target;
    assert_ne!(target, Some(Entity::Player(P1)));
    t.advance_to(P0, Step::EndOfCombat);
    // Lethal damage to the blocker, the rest to the defending player.
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn unblocked_trample_over_planeswalkers_when_the_planeswalker_is_removed() {
    cr!("702.19e");
    let mut t = TestGame::new(2);
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    let jace = t.battlefield(P1, "Jace Beleren");
    attack_with(&mut t, &[(thrasta, Entity::Object(jace))]);
    declare_blocks(&mut t, P1, &[]);
    t.g.destroy(jace, None);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn trample_without_over_planeswalkers_never_reaches_the_player() {
    cr!("702.19f");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let jace = t.battlefield(P1, "Jace Beleren");
    set_loyalty(&mut t, jace, 2);
    let maw2 = t.battlefield(P0, "Colossal Dreadmaw");
    let jace2 = t.battlefield(P1, "Liliana of the Veil");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(
        &mut t,
        &[(maw, Entity::Object(jace)), (maw2, Entity::Object(jace2))],
    );
    declare_blocks(&mut t, P1, &[(bears, maw2)]);
    // Liliana is removed from combat; Jace stays with only 2 loyalty.
    t.g.destroy(jace2, None);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    // All 6 of the first Dreadmaw's damage went to Jace; the second's went to the
    // blocker only.
    assert!(!t.on_battlefield(jace));
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn multiple_instances_of_trample_are_redundant() {
    cr!("702.19g");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let rancor = t.battlefield(P0, "Rancor");
    t.g.attach(rancor, Entity::Object(maw));
    t.g.recompute();
    assert_eq!(keyword_count(&t, maw, KeywordKind::Trample), 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(maw, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(bears, maw)]);
    assert_eq!(damage_decisions(&t).len(), 1);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn multiple_instances_of_trample_over_planeswalkers_are_redundant() {
    cr!("702.19g");
    let def = custom_card(
        "Twice Over",
        "Creature — Dinosaur",
        Some((7, 7)),
        "Trample over planeswalkers\nTrample over planeswalkers",
    );
    let mut t = TestGame::new(2);
    let dino = bf(&mut t, P0, def);
    assert_eq!(keyword_count(&t, dino, KeywordKind::Trample), 2);
    let jace = t.battlefield(P1, "Jace Beleren");
    set_loyalty(&mut t, jace, 3);
    attack_with(&mut t, &[(dino, Entity::Object(jace))]);
    block_and_finish(&mut t, P1, &[]);
    let d = damage_decisions(&t);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].2, vec![Entity::Object(jace), Entity::Player(P1)]);
    assert_eq!(t.life(P1), 16);
}
