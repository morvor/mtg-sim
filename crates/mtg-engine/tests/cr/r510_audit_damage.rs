//! CR 510 audit: combat damage assignment with creatures removed from combat (506.4,
//! 506.4c, 509.1h, 510.1b–d), trample's lethal damage counting damage already marked
//! (702.19b) and a creature with double strike and trample (702.19d), what a creature
//! attacking a planeswalker that left does with its damage (702.19e–f), and which
//! creatures assign damage in the second combat damage step (510.4, 702.7c).

use crate::r506_audit_common::*;
use crate::r506_common::{block, declare, go_to};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn attack(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    t.set_step(P0, Step::BeginningOfCombat);
    declare(t, attackers);
    if !blocks.is_empty() {
        block(t, P1, blocks);
    }
    go_to(t, Step::DeclareBlockers);
}

/// Moves a permanent to its owner's graveyard (it leaves the battlefield, CR 506.4).
fn bury(t: &mut TestGame, id: ObjectId) {
    let owner = t.obj_now(id).owner;
    t.g.move_object(
        id,
        Zone::Graveyard(owner),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.recompute();
    t.g.flush_events();
}

#[test]
fn double_strike_and_trample_tramples_over_a_blocker_dealt_lethal_damage_in_the_first_step() {
    cr!("510.4", "702.19b", "702.4b");
    ruling!(
        "Domri Rade",
        "the combat damage it assigned during the first combat damage step will be considered when determining how much damage can trample through in the second combat damage step"
    );
    supported("Crimson Caravaneer");
    supported("Darksteel Myr");
    let mut t = TestGame::new(2);
    // Crimson Caravaneer: 1/2 double strike, trample; +3/+0: a 4/2.
    let cc = t.battlefield(P0, "Crimson Caravaneer");
    pump(&mut t, cc, 3);
    // Darksteel Myr: 0/1 indestructible. It survives lethal damage.
    let myr = t.battlefield(P1, "Darksteel Myr");
    attack(&mut t, &[(cc, Entity::Player(P1))], &[(myr, cc)]);
    t.script.lock().unwrap().asked.clear();
    // First step: 1 is lethal for the Myr, the rest tramples over.
    assign(&mut t, P0, &[1, 3]);
    go_to(&mut t, Step::FirstStrikeDamage);
    t.settle();
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(myr));
    assert_eq!(damage(&t, myr), 1);
    // Second step: the Myr already has lethal damage marked; nothing more is needed for
    // it, so all 4 may go to the player.
    assign(&mut t, P0, &[0, 4]);
    go_to(&mut t, Step::CombatDamage);
    let d = damage_decisions(&t);
    assert_eq!(d.len(), 2);
    assert_eq!(d[0].3, vec![1]);
    assert_eq!(d[1].3, vec![0]);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn double_strike_and_trample_assigns_all_to_the_player_once_its_blocker_died() {
    cr!("510.4", "702.19d", "509.1h");
    ruling!(
        "Temur Battle Rage",
        "If an attacking creature with double strike and trample destroys all of its blocking creatures with first-strike combat damage, all of its normal combat damage is assigned to the player, planeswalker, or battle that creature's attacking."
    );
    let mut t = TestGame::new(2);
    let cc = t.battlefield(P0, "Crimson Caravaneer");
    pump(&mut t, cc, 3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack(&mut t, &[(cc, Entity::Player(P1))], &[(bears, cc)]);
    go_to(&mut t, Step::FirstStrikeDamage);
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 18);
    // Still blocked (509.1h), but no creature is blocking it.
    assert!(blocked(&t, cc));
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 14);
    // The Caravaneer took no damage from the Bears (they were dead before the regular
    // step).
    assert_eq!(damage(&t, cc), 0);
}

#[test]
fn double_strike_and_trample_attacking_a_planeswalker_it_destroyed_deals_nothing_more() {
    cr!("510.1b", "506.4c", "702.19f", "510.4");
    supported("Jace Beleren");
    let mut t = TestGame::new(2);
    let cc = t.battlefield(P0, "Crimson Caravaneer");
    pump(&mut t, cc, 4);
    let jace = t.battlefield(P1, "Jace Beleren");
    attack(&mut t, &[(cc, Entity::Object(jace))], &[]);
    go_to(&mut t, Step::FirstStrikeDamage);
    t.settle();
    assert!(!t.on_battlefield(jace));
    assert_eq!(t.life(P1), 20, "trample doesn't reach the player");
    go_to(&mut t, Step::CombatDamage);
    // It's attacking nothing: its regular damage isn't assigned (and not to P1).
    assert_eq!(attacks(&t, cc), None);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn trample_over_planeswalkers_reaches_the_controller_once_the_planeswalker_is_gone() {
    cr!("702.19c", "702.19e", "510.4");
    supported("Thrasta, Tempest's Roar");
    let mut t = TestGame::new(2);
    // Thrasta: 7/7 trample, trample over planeswalkers; with double strike.
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    give(&mut t, thrasta, KeywordKind::DoubleStrike);
    let jace = t.battlefield(P1, "Jace Beleren");
    attack(&mut t, &[(thrasta, Entity::Object(jace))], &[]);
    // First step: 3 (Jace's loyalty) to Jace, the rest to P1 (CR 702.19c).
    assign(&mut t, P0, &[3, 4]);
    go_to(&mut t, Step::FirstStrikeDamage);
    t.settle();
    assert!(!t.on_battlefield(jace));
    assert_eq!(t.life(P1), 16);
    // Second step: Jace was removed from combat; all its damage goes to P1 (CR 702.19e).
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 9);
}

#[test]
fn a_trampler_attacking_a_removed_planeswalker_assigns_everything_to_its_blockers() {
    cr!("506.4c", "702.19f", "510.1c");
    supported("Colossal Dreadmaw");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let jace = t.battlefield(P1, "Jace Beleren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack(&mut t, &[(maw, Entity::Object(jace))], &[(bears, maw)]);
    bury(&mut t, jace);
    assert!(t.g.is_attacking(maw));
    t.script.lock().unwrap().asked.clear();
    go_to(&mut t, Step::CombatDamage);
    // No player is among the recipients: P1 isn't attacked.
    for (_, _, recipients, _, _) in damage_decisions(&t) {
        assert!(!recipients.contains(&Entity::Player(P1)));
    }
    assert_eq!(t.life(P1), 20);
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn a_trampler_must_assign_lethal_damage_to_a_blocker_whose_protection_will_prevent_it() {
    cr!("702.19b");
    supported("Zombie Outlander");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    // Zombie Outlander: 2/2 protection from green. The damage will be prevented, but
    // lethal damage is judged without that.
    let zombie = t.battlefield(P1, "Zombie Outlander");
    attack(&mut t, &[(maw, Entity::Player(P1))], &[(zombie, maw)]);
    // All 6 to the player: illegal (the zombie hasn't been assigned lethal damage).
    assign(&mut t, P0, &[0, 6]);
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 16);
    t.settle();
    assert!(t.on_battlefield(zombie));
    assert_eq!(damage(&t, zombie), 0);
}

#[test]
fn a_first_striker_that_loses_first_strike_doesnt_deal_damage_again() {
    cr!("702.7c", "510.4");
    supported("Youthful Knight");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    attack(&mut t, &[(knight, Entity::Player(P1))], &[]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    take(&mut t, knight, KeywordKind::FirstStrike);
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_first_striker_that_assigned_no_damage_doesnt_assign_in_the_second_step() {
    cr!("510.4", "510.1a");
    supported("Kraken Hatchling");
    let mut t = TestGame::new(2);
    // Kraken Hatchling: 0/4. With first strike it assigns nothing in the first step.
    let kraken = t.battlefield(P0, "Kraken Hatchling");
    give(&mut t, kraken, KeywordKind::FirstStrike);
    attack(&mut t, &[(kraken, Entity::Player(P1))], &[]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 20);
    // It had first strike as the first step began: pumped now, it still deals nothing
    // in the second step.
    pump(&mut t, kraken, 3);
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_creature_that_gains_first_strike_after_the_first_step_still_deals_damage() {
    cr!("702.7c", "510.4");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    // The Bears block the Giant; the Knight is unblocked.
    attack(
        &mut t,
        &[(knight, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[(bears, giant)],
    );
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(damage(&t, giant), 0);
    // The blocking Bears gain first strike after the first step: they had neither as it
    // began, so they assign damage in the second step.
    give(&mut t, bears, KeywordKind::FirstStrike);
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(damage(&t, giant), 2);
}

#[test]
fn a_ninja_with_first_strike_entering_in_the_first_strike_step_deals_regular_damage() {
    cr!("510.4", "702.49a");
    ruling!(
        "Ninja of the Deep Hours",
        "The Ninja will deal combat damage during the regular combat damage step in this case, even if it has first strike."
    );
    supported("Ninja of the Deep Hours");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    attack(&mut t, &[(knight, Entity::Player(P1))], &[]);
    go_to(&mut t, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18);
    t.answer_choose(P0, &[Entity::Object(knight)]);
    t.activate(P0, ninja, 0, &[]).unwrap();
    t.resolve_all();
    let n = t.named_on_battlefield("Ninja of the Deep Hours")[0];
    give(&mut t, n, KeywordKind::FirstStrike);
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn an_attacker_whose_blocker_was_removed_stays_blocked_for_ninjutsu() {
    cr!("509.1h", "506.4");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 2);
    let ninja = t.hand(P0, "Ninja of the Deep Hours");
    attack(&mut t, &[(bears, Entity::Player(P1))], &[(giant, bears)]);
    remove_from_combat(&mut t, giant);
    assert!(blocked(&t, bears));
    t.answer_choose(P0, &[Entity::Object(bears)]);
    assert!(t.activate(P0, ninja, 0, &[]).is_err());
    go_to(&mut t, Step::CombatDamage);
    // Blocked with no blockers: no damage anywhere.
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage(&t, giant), 0);
}

#[test]
fn a_blocker_whose_other_attacker_left_combat_assigns_all_its_damage_to_the_one_left() {
    cr!("510.1d", "506.4");
    supported("Night Market Guard");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    // Night Market Guard: 3/1, can block an additional creature.
    let guard = t.battlefield(P1, "Night Market Guard");
    attack(
        &mut t,
        &[(a, Entity::Player(P1)), (b, Entity::Player(P1))],
        &[(guard, a), (guard, b)],
    );
    assert_eq!(blocking(&t, guard).len(), 2);
    remove_from_combat(&mut t, a);
    t.script.lock().unwrap().asked.clear();
    // P1 tries to put 1 damage on the removed Bears: not a recipient.
    assign(&mut t, P1, &[1, 2]);
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(damage(&t, a), 0);
    t.settle();
    assert!(!t.on_battlefield(b));
    assert!(t.on_battlefield(a));
    // Only one blocking assignment was asked of nobody: exactly one creature, all to it.
    assert!(damage_decisions(&t).iter().all(|d| d.0 != P1));
}

#[test]
fn dazzling_beauty_blocks_a_trampler_that_then_assigns_everything_to_the_player() {
    cr!("509.1h", "702.19d", "510.1c");
    ruling!(
        "Dazzling Beauty",
        "Trample damage comes through as if a zero toughness creature blocked the Trampler"
    );
    supported("Dazzling Beauty");
    for trample in [true, false] {
        let mut t = TestGame::new(2);
        let a = if trample {
            t.battlefield(P0, "Colossal Dreadmaw")
        } else {
            t.battlefield(P0, "Craw Wurm")
        };
        t.lands(P1, "Plains", 3);
        let beauty = t.hand(P1, "Dazzling Beauty");
        attack(&mut t, &[(a, Entity::Player(P1))], &[]);
        assert!(!blocked(&t, a));
        t.cast(P1, beauty).target(a).go();
        t.resolve();
        assert!(blocked(&t, a));
        go_to(&mut t, Step::CombatDamage);
        assert_eq!(t.life(P1), if trample { 14 } else { 20 });
    }
}

#[test]
fn removing_the_only_first_striker_before_damage_means_one_damage_step() {
    cr!("510.4", "506.1");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Youthful Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack(
        &mut t,
        &[(knight, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[],
    );
    remove_from_combat(&mut t, knight);
    go_to(&mut t, Step::CombatDamage);
    assert!(!t.g.turn.step_log.contains(&Step::FirstStrikeDamage));
    assert_eq!(t.life(P1), 18);
}
