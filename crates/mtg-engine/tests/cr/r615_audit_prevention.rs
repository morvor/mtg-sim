//! CR 615 audit: prevention effects against damage that can't be prevented. Such damage
//! is dealt; a shield applied to it isn't reduced or used up, so it still prevents later
//! damage (615.12, 609.7b); and "can't be prevented" doesn't stop redirection, which isn't
//! prevention (614.9).

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_cop_shield_isnt_used_up_by_unpreventable_damage_and_prevents_the_next() {
    cr!("615.12", "609.7b", "615.8");
    supported("Circle of Protection: Red");
    supported("Prodigal Pyromancer");
    supported("Leyline of Punishment");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let leyline = t.battlefield(P0, "Leyline of Punishment");
    let cop = t.battlefield(P1, "Circle of Protection: Red");
    t.lands(P1, "Plains", 1);
    t.answer_choose(P1, &[obj(pyro)]);
    t.activate(P1, cop, 0, &[]).expect("activate the Circle");
    t.resolve_all();
    // Damage can't be prevented: the Pyromancer's 1 damage is dealt, and the shield isn't
    // used up.
    t.activate(P0, pyro, 0, &[Entity::Player(P1)])
        .expect("ping");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Without the Leyline, the same shield prevents the next damage from that source.
    cast_resolve(&mut t, P1, "Disenchant", &[obj(leyline)]);
    t.g.untap(pyro);
    t.activate(P0, pyro, 0, &[Entity::Player(P1)])
        .expect("ping again");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // ... and then it's gone.
    t.g.untap(pyro);
    t.activate(P0, pyro, 0, &[Entity::Player(P1)])
        .expect("ping a third time");
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_healing_salve_shield_isnt_reduced_by_banefire_and_prevents_a_later_bolt() {
    cr!("615.12", "615.7");
    supported("Healing Salve");
    supported("Banefire");
    let mut t = TestGame::new(2);
    // Healing Salve's second mode: prevent the next 3 damage to P1 this turn.
    t.lands(P1, "Plains", 1);
    let salve = t.hand(P1, "Healing Salve");
    t.answer_targets(P1, &[Entity::Player(P1)]);
    t.cast(P1, salve).modes(&[1]).go();
    t.resolve_all();
    // Banefire with X = 5: its damage can't be prevented.
    t.lands(P0, "Mountain", 6);
    let bf = t.hand(P0, "Banefire");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, bf).x(5).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // The shield is still 3.
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 15);
    cast_resolve(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 13);
}

#[test]
fn unpreventable_damage_is_still_redirected_by_palisade_giant() {
    cr!("615.12", "614.9");
    supported("Skullcrack");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Palisade Giant");
    // Skullcrack: damage can't be prevented this turn, and 3 damage to target player.
    cast_resolve(&mut t, P0, "Skullcrack", &[Entity::Player(P1)]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage_on(&t, giant), 3);
}

#[test]
fn protection_doesnt_prevent_damage_that_cant_be_prevented() {
    cr!("615.12", "702.16e");
    supported("Kor Firewalker");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P1, "Kor Firewalker");
    t.battlefield(P0, "Leyline of Punishment");
    cast_resolve(&mut t, P0, "Pyroclasm", &[]);
    assert!(!t.on_battlefield(walker));
    assert!(t.in_graveyard(P1, "Kor Firewalker"));
}

#[test]
fn a_prevention_choice_is_offered_for_unpreventable_damage_but_prevents_none() {
    cr!("615.12", "615.12a", "616.1");
    supported("Urza's Armor");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Urza's Armor");
    t.battlefield(P1, "Urza's Armor");
    t.battlefield(P0, "Leyline of Punishment");
    t.answer(P1, DecisionKind::Replacement, Answer::Index(0));
    cast_resolve(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    // Each Armor is applied once, preventing nothing (no endless loop).
    assert_eq!(t.life(P1), 17);
}
