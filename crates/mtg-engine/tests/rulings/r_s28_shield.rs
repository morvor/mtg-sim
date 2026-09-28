//! Rulings batch S28 — shield counters (CR 122.1c): one or more shield counters create a
//! single replacement effect ("If this permanent would be destroyed as the result of an
//! effect, instead remove a shield counter from it") and a single prevention effect ("If
//! damage would be dealt to this permanent, prevent that damage and remove a shield
//! counter from it"). They don't stop state-based actions or sacrifice, still lose a
//! counter to unpreventable damage (CR 615.12), aren't regeneration (CR 701.19), and
//! aren't an ability the permanent has.

use crate::r_s01_common::{attack_with, block_and_finish, supported};
use crate::r_s06_common::damage;
use crate::r_s28_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

const SHIELD: &str = "shield";

/// Casts Boon of Safety ("Put a shield counter on target creature. Scry 1.") on `target`.
fn boon(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    t.answer_targets(p, &[Entity::Object(target)]);
    cast_card(t, p, "Boon of Safety");
    t.resolve_all();
}

/// Casts `name` from P1's hand targeting `target`, and resolves it.
fn p1_casts_at(t: &mut TestGame, name: &str, target: Entity) {
    t.answer_targets(P1, &[target]);
    cast_card(t, P1, name);
    t.resolve_all();
}

#[test]
fn two_shield_counters_prevent_damage_once_and_only_one_is_removed() {
    cr!("122.1c", "615.1a");
    ruling!(
        "Boon of Safety",
        "If a permanent that would be dealt damage has more than one shield counter on it, that damage is prevented and only one shield counter is removed."
    );
    supported("Boon of Safety");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    boon(&mut t, P0, giant);
    boon(&mut t, P0, giant);
    assert_eq!(t.counters(giant, SHIELD), 2);
    p1_casts_at(&mut t, "Lightning Bolt", Entity::Object(giant));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.counters(giant, SHIELD), 1);
    // The remaining counter still protects it.
    p1_casts_at(&mut t, "Lightning Bolt", Entity::Object(giant));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.counters(giant, SHIELD), 0);
}

#[test]
fn a_shield_counter_doesnt_stop_state_based_destruction() {
    cr!("122.1c", "704.5g", "704.5h", "702.2b");
    ruling!(
        "Protection Magic",
        "A creature with a shield counter on it may still be destroyed by state-based actions if it has damage marked on it equal to its toughness or has been dealt unpreventable damage by a source with deathtouch."
    );
    supported("Protection Magic");
    // Lethal damage: a 2/2 with 1 damage marked gets a shield counter, then -1/-1.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pinger = t.battlefield(P1, "Hill Giant");
    damage(&mut t, pinger, 1, bears);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_card(&mut t, P0, "Protection Magic");
    t.resolve_all();
    assert_eq!(t.counters(bears, SHIELD), 1);
    p1_casts_at(&mut t, "Tragic Slip", Entity::Object(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Deathtouch: with Leyline of Punishment, the damage can't be prevented.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    cast_card(&mut t, P0, "Protection Magic");
    t.resolve_all();
    t.battlefield(P1, "Leyline of Punishment");
    let rats = t.battlefield(P1, "Typhoid Rats");
    damage(&mut t, rats, 1, giant);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn unpreventable_damage_is_dealt_and_still_removes_a_shield_counter() {
    cr!("122.1c", "615.12");
    ruling!(
        "Boon of Safety",
        "If a permanent with a shield counter is dealt unpreventable damage, that damage will be dealt and a shield counter will still be removed."
    );
    supported("Leyline of Punishment");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    boon(&mut t, P0, giant);
    boon(&mut t, P0, giant);
    t.battlefield(P1, "Leyline of Punishment");
    p1_casts_at(&mut t, "Shock", Entity::Object(giant));
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.counters(giant, SHIELD), 1);
}

#[test]
fn shield_counters_dont_stop_sacrifice() {
    cr!("122.1c", "701.21a");
    ruling!(
        "Protection Magic",
        "Shield counters don't prevent players from sacrificing creatures."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_card(&mut t, P0, "Protection Magic");
    t.resolve_all();
    assert_eq!(t.counters(bears, SHIELD), 1);
    // Diabolic Edict: "Target player sacrifices a creature of their choice."
    p1_casts_at(&mut t, "Diabolic Edict", Entity::Player(P0));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn a_shield_counter_isnt_regeneration() {
    cr!("122.1c", "701.19a", "701.19c");
    ruling!(
        "Boon of Safety",
        "Removing a shield counter in this way isn't the same as regenerating a creature."
    );
    // Wrath of God: "Destroy all creatures. They can't be regenerated." The shield counter
    // still protects the creature, which isn't tapped the way a regenerated one would be.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    boon(&mut t, P0, bears);
    let other = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    let wrath = t.hand(P1, "Wrath of God");
    t.lands(P1, "Plains", 4);
    t.cast(P1, wrath).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(t.counters(bears, SHIELD), 0);
    assert!(!t.on_battlefield(other));
}

#[test]
fn a_shielded_attacker_stays_in_combat_when_it_would_be_destroyed() {
    cr!("122.1c", "701.19a", "510.1");
    ruling!(
        "Proud Pack-Rhino",
        "Removing a shield counter when a permanent would be dealt damage or destroyed isn't the same as regenerating that permanent."
    );
    supported("Proud Pack-Rhino");
    // Proud Pack-Rhino: "When this creature enters, choose one — • Put a shield counter on
    // target permanent. • Proliferate."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Modes,
        mtg_engine::decision::Answer::Indices(vec![0]),
    );
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Proud Pack-Rhino");
    t.resolve_all();
    assert_eq!(t.counters(bears, SHIELD), 1);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    // Murdered while attacking: the counter is removed instead, and unlike a regenerated
    // creature it isn't removed from combat, so it deals its combat damage.
    p1_casts_at(&mut t, "Murder", Entity::Object(bears));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, SHIELD), 0);
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_shield_counter_protects_a_creature_that_lost_its_abilities() {
    cr!("122.1c", "613.1f");
    ruling!(
        "Swooping Protector",
        "\"Shield\" is not an ability that creatures have and shield counters are not keyword counters. If a creature with a shield counter loses its abilities, the shield counter will still protect it as normal."
    );
    supported("Swooping Protector");
    supported("Humility");
    // Swooping Protector: "This creature enters with a shield counter on it."
    let mut t = TestGame::new(2);
    let protector = t.enter(P0, "Swooping Protector");
    t.resolve_all();
    assert_eq!(t.counters(protector, SHIELD), 1);
    t.battlefield(P1, "Humility");
    t.g.recompute();
    assert_eq!(t.pt(protector), (1, 1));
    assert!(t.obj_now(protector).chars.abilities.is_empty());
    p1_casts_at(&mut t, "Shock", Entity::Object(protector));
    assert!(t.on_battlefield(protector));
    assert_eq!(t.obj_now(protector).damage, 0);
    assert_eq!(t.counters(protector, SHIELD), 0);
}

#[test]
fn a_shield_counter_saves_a_creature_without_abilities_from_destruction() {
    cr!("122.1c", "613.1f");
    ruling!(
        "Dapper Shieldmate",
        "“Shield” is not an ability that creatures have and shield counters are not keyword counters. If a creature with a shield counter loses its abilities, the shield counter will still protect it as normal."
    );
    supported("Dapper Shieldmate");
    supported("Dress Down");
    let mut t = TestGame::new(2);
    let mate = t.enter(P0, "Dapper Shieldmate");
    t.resolve_all();
    assert_eq!(t.counters(mate, SHIELD), 1);
    // Dress Down: "Creatures lose all abilities."
    t.battlefield(P1, "Dress Down");
    t.g.recompute();
    assert!(t.obj_now(mate).chars.abilities.is_empty());
    p1_casts_at(&mut t, "Murder", Entity::Object(mate));
    assert!(t.on_battlefield(mate));
    assert_eq!(t.counters(mate, SHIELD), 0);
}

#[test]
fn simultaneous_damage_from_several_sources_removes_one_shield_counter() {
    cr!("122.1c", "510.2");
    // Blocked by two creatures: the single prevention effect applies to all of the
    // combat damage dealt at once, preventing it and removing one shield counter.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    boon(&mut t, P0, giant);
    boon(&mut t, P0, giant);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(a, giant), (b, giant)]);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.counters(giant, SHIELD), 1);
    // With a single shield counter, all of the damage is still prevented.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    boon(&mut t, P0, giant);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(a, giant), (b, giant)]);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 0);
    assert_eq!(t.counters(giant, SHIELD), 0);
}
