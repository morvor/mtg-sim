//! CR 702.22 audit (banding in combat, with 510.1c–d's exceptions in 702.22j–k): a band
//! is blocked by any creature that could block one of its members, even a member it
//! couldn't block itself (flying, protection: 702.22h), and the active player then divides
//! that blocker's damage, even onto a creature whose protection prevents it (702.22k).

use crate::r506_audit_common::*;
use crate::r506_common::{block, declare, go_to};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 attacks P1 with Benalish Hero banded with `other`; P1 blocks with Hill Giant as
/// `blocks` says. Returns (hero, other, giant) at the declare blockers step.
fn band_attack(t: &mut TestGame, other: &str, block_other: bool) -> (ObjectId, ObjectId, ObjectId) {
    supported("Benalish Hero");
    supported(other);
    let hero = t.battlefield(P0, "Benalish Hero");
    let o = t.battlefield(P0, other);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(t, &[(hero, Entity::Player(P1)), (o, Entity::Player(P1))]);
    // Band the other creature with the Hero (CR 508.1e).
    t.answer_choose(P0, &[Entity::Object(o)]);
    block(t, P1, &[(giant, if block_other { o } else { hero })]);
    go_to(t, Step::DeclareBlockers);
    let c = t.g.combat.as_ref().unwrap();
    let band = c.attacker(hero).unwrap().band;
    assert!(band.is_some());
    assert_eq!(c.attacker(o).unwrap().band, band);
    (hero, o, giant)
}

#[test]
fn a_ground_creature_blocking_a_band_blocks_its_flier_too() {
    cr!("702.22h", "702.22k", "510.1d");
    ruling!(
        "Benalish Hero",
        "The band can be blocked by any creature that could block a single creature in the band."
    );
    let mut t = TestGame::new(2);
    let (hero, angel, giant) = band_attack(&mut t, "Serra Angel", false);
    let mut b = blocking(&t, giant);
    b.sort();
    let mut want = vec![hero, angel];
    want.sort();
    assert_eq!(b, want);
    assert!(blocked(&t, angel));
    // The active player divides the Giant's 3 damage: all of it to the 4/4 Angel.
    t.script.lock().unwrap().asked.clear();
    let order: Vec<i64> = blocking(&t, giant)
        .iter()
        .map(|x| if *x == angel { 3 } else { 0 })
        .collect();
    assign(&mut t, P0, &order);
    go_to(&mut t, Step::CombatDamage);
    t.settle();
    assert!(damage_decisions(&t)
        .iter()
        .any(|d| d.0 == P0 && d.1 == giant));
    assert!(t.on_battlefield(hero));
    assert!(t.on_battlefield(angel));
    assert_eq!(damage(&t, angel), 3);
    // The blocked Angel deals its damage to the Giant, not to P1.
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn the_flier_in_a_band_cant_itself_be_blocked_by_a_ground_creature() {
    cr!("509.1b", "702.9b", "702.22h");
    let mut t = TestGame::new(2);
    // P1 declares the Giant blocking the Angel: illegal, undone (no block).
    let (hero, angel, giant) = band_attack(&mut t, "Serra Angel", true);
    assert!(blocking(&t, giant).is_empty());
    assert!(!blocked(&t, hero));
    assert!(!blocked(&t, angel));
}

#[test]
fn a_red_blocker_of_a_band_blocks_its_protected_member_and_its_damage_can_be_put_there() {
    cr!("702.22h", "702.22k", "702.16e", "702.16f");
    let mut t = TestGame::new(2);
    // Galina's Knight: 2/2 protection from red. The red Hill Giant can't block it, but
    // blocking the Hero blocks the band.
    let (hero, knight, giant) = band_attack(&mut t, "Galina's Knight", false);
    assert!(blocked(&t, knight));
    let order: Vec<i64> = blocking(&t, giant)
        .iter()
        .map(|x| if *x == knight { 3 } else { 0 })
        .collect();
    assign(&mut t, P0, &order);
    go_to(&mut t, Step::CombatDamage);
    t.settle();
    // All 3 assigned to the Knight are prevented by protection.
    assert!(t.on_battlefield(hero));
    assert!(t.on_battlefield(knight));
    assert_eq!(damage(&t, knight), 0);
    // Hero 1 + Knight 2 = 3: the Giant dies.
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn a_band_member_removed_from_combat_stays_blocked_and_its_blocker_still_divides_by_the_active_player(
) {
    cr!("702.22f", "702.22k", "509.1h");
    let mut t = TestGame::new(2);
    let (hero, angel, giant) = band_attack(&mut t, "Serra Angel", false);
    // The Hero (with banding) leaves combat: the Angel is still blocked, by a creature
    // that now blocks only it, so the Giant's controller assigns all of its damage there.
    remove_from_combat(&mut t, hero);
    assert!(blocked(&t, angel));
    assert_eq!(blocking(&t, giant), vec![angel]);
    t.script.lock().unwrap().asked.clear();
    go_to(&mut t, Step::CombatDamage);
    assert_eq!(damage(&t, angel), 3);
    assert_eq!(damage(&t, hero), 0);
    assert_eq!(t.life(P1), 20);
}
