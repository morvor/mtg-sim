//! Rulings batch P062 — "Whenever this creature blocks a creature, [tap that creature.]
//! That creature doesn't untap during its controller's next untap step." and "Whenever this
//! deals combat damage to a creature, tap that creature and it doesn't untap ...": the
//! effect applies to the creature even if it was already tapped (an attacking creature),
//! doesn't tap a creature the ability doesn't say to tap, lasts only through its
//! controller's next untap step (CR 502.3), and doesn't depend on its source staying on
//! the battlefield (CR 611.2a).

use crate::r_p062_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s02_common::can_attack;
use crate::r_s04_common::run_with;
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s20_common::tap_for_mana;
use mtg_engine::ability::{Effect, Sel};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 attacks P0 with `attacker`; P0 blocks it with `blockers`; combat ends and the
/// triggered abilities resolve.
fn p1_attacks_into(t: &mut TestGame, attacker: ObjectId, blockers: &[ObjectId]) {
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(t, &[(attacker, Entity::Player(P0))]);
    let blocks: Vec<(ObjectId, ObjectId)> = blockers.iter().map(|b| (*b, attacker)).collect();
    block_and_finish(t, P0, &blocks);
    t.resolve_all();
}

#[test]
fn vertigo_spawns_ability_applies_to_an_already_tapped_creature() {
    cr!("502.3", "508.1f", "509.1a");
    ruling!(
        "Vertigo Spawn",
        "If the creature was already tapped when Vertigo Spawn blocked it, that creature still won't untap during its controller's next untap step."
    );
    supported("Vertigo Spawn");
    // Hill Giant became tapped as it attacked.
    let mut t = TestGame::new(2);
    let spawn = t.battlefield(P0, "Vertigo Spawn");
    let giant = t.battlefield(P1, "Hill Giant");
    p1_attacks_into(&mut t, giant, &[spawn]);
    assert!(is_tapped(&t, giant));
    misses_one_untap(&mut t, giant, P1);
}

#[test]
fn two_vertigo_spawns_blocking_affect_only_the_next_untap_step() {
    cr!("502.3", "509.1a", "603.2");
    ruling!(
        "Vertigo Spawn",
        "If two Vertigo Spawns block the same creature, both abilities trigger, but each only affects the creature during its controller's next untap step. The creature can untap during its controller's untap step following that one."
    );
    supported("Vertigo Spawn");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Vertigo Spawn");
    let b = t.battlefield(P0, "Vertigo Spawn");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(a, giant), (b, giant)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    t.settle();
    // Both abilities triggered.
    assert_eq!(
        crate::r_s01_common::triggers_on_stack(&t, "doesn't untap"),
        2
    );
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    misses_one_untap(&mut t, giant, P1);
}

#[test]
fn cleric_of_chill_depths_doesnt_tap_an_untapped_blocked_creature() {
    cr!("502.3", "702.20b");
    ruling!(
        "Cleric of Chill Depths",
        "If the blocked creature is untapped (most likely because it has vigilance), the ability doesn’t tap it."
    );
    supported("Cleric of Chill Depths");
    // Wary Okapi (3/2 vigilance) attacks and stays untapped.
    let mut t = TestGame::new(2);
    let cleric = t.battlefield(P0, "Cleric of Chill Depths");
    let okapi = t.battlefield(P1, "Wary Okapi");
    p1_attacks_into(&mut t, okapi, &[cleric]);
    assert!(!is_tapped(&t, okapi));
    // It's still affected: tapped later this turn, it doesn't untap during P1's next
    // untap step.
    tap(&mut t, okapi);
    misses_one_untap(&mut t, okapi, P1);
}

#[test]
fn the_creature_blocked_by_cleric_of_chill_depths_stays_tapped_after_the_cleric_dies() {
    cr!("502.3", "611.2a", "510.2");
    ruling!(
        "Cleric of Chill Depths",
        "The blocked creature doesn’t untap even if Cleric of Chill Depths leaves the battlefield before that creature’s controller’s next upkeep."
    );
    supported("Cleric of Chill Depths");
    // Hill Giant (3/3) kills the Cleric (1/3) in combat.
    let mut t = TestGame::new(2);
    let cleric = t.battlefield(P0, "Cleric of Chill Depths");
    let giant = t.battlefield(P1, "Hill Giant");
    p1_attacks_into(&mut t, giant, &[cleric]);
    assert!(t.in_graveyard(P0, "Cleric of Chill Depths"));
    misses_one_untap(&mut t, giant, P1);
}

#[test]
fn wall_of_frost_does_nothing_if_the_creature_is_untapped_by_its_next_untap_step() {
    cr!("502.3", "701.26b");
    ruling!(
        "Wall of Frost",
        "If the creature isn’t tapped during its controller’s next untap step (perhaps because it was untapped by a spell), Wall of Frost’s ability has no effect at that time. It won’t try to keep the creature tapped on subsequent turns."
    );
    supported("Wall of Frost");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Wall of Frost");
    let giant = t.battlefield(P1, "Hill Giant");
    p1_attacks_into(&mut t, giant, &[wall]);
    assert!(is_tapped(&t, giant));
    // An untap effect untaps the Giant before P1's next untap step.
    run_with(
        &mut t,
        P1,
        Effect::Untap {
            what: Sel::Target(0),
        },
        &[obj(giant)],
    );
    assert!(!is_tapped(&t, giant));
    through_untap_step(&mut t, P1);
    assert!(!is_tapped(&t, giant));
    // Tapped again, it untaps as normal in P1's following untap step.
    tap(&mut t, giant);
    untaps_next(&mut t, giant, P1);
}

#[test]
fn the_illusion_tokens_ability_doesnt_tap_an_untapped_attacker() {
    cr!("502.3", "702.20b");
    ruling!(
        "Mesmerizing Benthid",
        "The triggered ability of the Illusion tokens doesn't tap the attacking creature if it's untapped, most likely because it has vigilance."
    );
    supported("Mesmerizing Benthid");
    let mut t = TestGame::new(2);
    crate::r_s05_common::enter(&mut t, P0, "Mesmerizing Benthid");
    t.resolve_all();
    let illusions = crate::r_s01_common::with_subtype(&t, P0, "Illusion");
    assert_eq!(illusions.len(), 2);
    let okapi = t.battlefield(P1, "Wary Okapi");
    p1_attacks_into(&mut t, okapi, &[illusions[0]]);
    assert!(!is_tapped(&t, okapi));
    // A non-vigilant attacker blocked by the other token stays tapped.
    let giant = t.battlefield(P1, "Hill Giant");
    p1_attacks_into(&mut t, giant, &[illusions[1]]);
    assert!(is_tapped(&t, giant));
    misses_one_untap(&mut t, giant, P1);
}

#[test]
fn mesmerizing_benthid_has_hexproof_with_any_illusion() {
    cr!("702.11b", "604.2");
    ruling!(
        "Mesmerizing Benthid",
        "Mesmerizing Benthid has hexproof as long as you control any Illusion, not just Illusion tokens like the ones created by its first ability."
    );
    supported("Mesmerizing Benthid");
    let mut t = TestGame::new(2);
    let benthid = t.battlefield(P0, "Mesmerizing Benthid");
    t.g.recompute();
    assert!(!has_kw(&t, benthid, KeywordKind::Hexproof));
    // Vertigo Spawn is a nontoken Illusion.
    let spawn = t.battlefield(P0, "Vertigo Spawn");
    t.g.recompute();
    assert!(has_kw(&t, benthid, KeywordKind::Hexproof));
    // An opponent's Illusion doesn't count.
    crate::r_s06_common::give_control(&mut t, spawn, P1);
    t.g.recompute();
    assert!(!has_kw(&t, benthid, KeywordKind::Hexproof));
}

/// Untaps the permanent directly (no untap event).
fn untap_directly(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.objects[id.0 as usize].tapped = false;
}

/// P0's Frostwalk Bastion, animated by its own ability (paid with snow mana).
fn animated_bastion(t: &mut TestGame) -> ObjectId {
    let bastion = t.battlefield(P0, "Frostwalk Bastion");
    t.lands(P0, "Snow-Covered Island", 2);
    activate_containing(t, P0, bastion, "becomes a 2/3").expect("animate the Bastion");
    t.resolve_all();
    // (The payment may have tapped the Bastion itself for {1}.)
    untap_directly(t, bastion);
    assert_eq!(t.pt(bastion), (2, 3));
    bastion
}

#[test]
fn frostwalk_bastions_damage_keeps_an_already_tapped_creature_tapped_after_it_leaves() {
    cr!("502.3", "510.2", "611.2a");
    ruling!(
        "Frostwalk Bastion",
        "A creature dealt combat damage by Frostwalk Bastion won't untap during its controller's next untap step, even if it was already tapped and even if Frostwalk Bastion leaves the battlefield before then."
    );
    supported("Frostwalk Bastion");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    let bastion = animated_bastion(&mut t);
    p1_attacks_into(&mut t, giant, &[bastion]);
    // The Giant (3/3) destroyed the Bastion (2/3); the attacking Giant was tapped.
    assert!(!t.on_battlefield(bastion));
    assert_eq!(t.obj_now(giant).damage, 2);
    misses_one_untap(&mut t, giant, P1);
}

#[test]
fn frostwalk_bastion_animated_the_turn_it_entered_cant_tap_for_mana_or_attack() {
    cr!("302.6", "508.1a");
    ruling!(
        "Frostwalk Bastion",
        "If you turn Frostwalk Bastion into a creature but haven't controlled it continuously since your most recent turn began, you won't be able to activate its mana ability or attack with it."
    );
    supported("Frostwalk Bastion");
    let mut t = TestGame::new(2);
    let bastion = crate::r_s20_common::entered_this_turn(&mut t, P0, "Frostwalk Bastion");
    t.lands(P0, "Snow-Covered Island", 2);
    activate_containing(&mut t, P0, bastion, "becomes a 2/3").expect("animate");
    t.resolve_all();
    untap_directly(&mut t, bastion);
    assert!(!tap_for_mana(&mut t, P0, bastion, "{C}"));
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, bastion));
    // One that P0 has controlled since the turn began can do both.
    let mut t = TestGame::new(2);
    let bastion = animated_bastion(&mut t);
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, bastion));
    assert!(tap_for_mana(&mut t, P0, bastion, "{C}"));
}

#[test]
fn queen_of_ices_damage_keeps_the_creature_tapped_after_the_queen_dies() {
    cr!("502.3", "510.2", "611.2a");
    ruling!(
        "Queen of Ice // Rage of Winter",
        "A creature dealt combat damage by Queen of Ice won't untap during its controller's next untap step, even if it was already tapped and even if Queen of Ice leaves the battlefield before then (perhaps because that creature dealt lethal combat damage to Queen of Ice)."
    );
    supported("Queen of Ice // Rage of Winter");
    // Queen of Ice (2/3) blocks Hill Giant (3/3) and dies.
    let mut t = TestGame::new(2);
    let queen = t.battlefield(P0, "Queen of Ice // Rage of Winter");
    let giant = t.battlefield(P1, "Hill Giant");
    p1_attacks_into(&mut t, giant, &[queen]);
    assert!(!t.on_battlefield(queen));
    misses_one_untap(&mut t, giant, P1);
}
