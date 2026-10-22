//! Rulings batch P107 — Pizza Face, Gastromancer: "put three +1/+1 counters on up to one
//! other target artifact or creature. If it isn't a creature, it becomes a 0/0 Mutant
//! creature in addition to its other types." (CR 613.1d, 613.4b, 301.5c, 702.184).

use crate::r_p107_common::*;
use crate::r_s02_common::can_attack;
use crate::r_s04_common::crew;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Pizza Face's end step trigger targeting `target`: a Grizzly Bears of P0's dies first so
/// that a permanent left the battlefield under P0's control this turn.
fn disappear_onto(t: &mut TestGame, target: ObjectId) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(t, bears);
    t.answer_targets(P0, &[obj(target)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(target, counters::PLUS1), 3);
}

#[test]
fn a_vehicle_becomes_a_0_0_mutant_and_crewing_doesnt_restore_its_pt() {
    cr!("613.4b", "702.122a", "205.1b");
    ruling!(
        "Pizza Face, Gastromancer",
        "If the target permanent is a Vehicle, its base power and toughness will be set to 0/0. Crewing that Vehicle will not restore its power and toughness."
    );
    ruling!(
        "Pizza Face, Gastromancer",
        "The target permanent retains any types, subtypes, or supertypes it has."
    );
    ruling!(
        "Pizza Face, Gastromancer",
        "Pizza Face's second ability doesn't remove any abilities the target permanent has."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pizza Face, Gastromancer");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    disappear_onto(&mut t, copter);
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (3, 3));
    let o = t.obj_now(copter);
    assert!(o.is(CardType::Artifact));
    assert!(o.chars.has_subtype("Vehicle") && o.chars.has_subtype("Mutant"));
    assert!(has_kw(&t, copter, mtg_engine::keywords::KeywordKind::Flying));
    assert!(o.chars.abilities.iter().any(|a| a.text.contains("Crew")));
    // Next turn, crew it: still 3/3 (its printed 3/3 doesn't come back).
    t.advance_to(P0, Step::PrecombatMain);
    let crew_bear = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[crew_bear]));
    t.resolve_all();
    assert_eq!(t.pt(copter), (3, 3));
}

#[test]
fn a_spacecraft_with_station_stays_0_0_based_with_more_charge_counters() {
    cr!("613.4b", "721.2");
    ruling!(
        "Pizza Face, Gastromancer",
        "If the target permanent has station, its base power and toughness will be set to 0/0."
    );
    // Wedgelight Rammer: "Station ... It's an artifact creature at 9+." with a 3/4 striation.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pizza Face, Gastromancer");
    let ship = t.battlefield(P0, "Wedgelight Rammer");
    disappear_onto(&mut t, ship);
    assert!(is_creature(&t, ship));
    assert_eq!(t.pt(ship), (3, 3));
    put_counters(&mut t, ship, counters::CHARGE, 9);
    assert!(is_creature(&t, ship));
    assert_eq!(t.pt(ship), (3, 3));
}

#[test]
fn an_attached_equipment_becomes_unattached_and_cant_be_reattached() {
    cr!("301.5c", "702.6a");
    ruling!(
        "Pizza Face, Gastromancer",
        "If the target permanent is an attached Equipment, it becomes unattached."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pizza Face, Gastromancer");
    let holder = t.battlefield(P0, "Hill Giant");
    let sword = attach_new(&mut t, P0, "Bonesplitter", holder);
    disappear_onto(&mut t, sword);
    assert!(is_creature(&t, sword));
    assert_eq!(t.obj_now(sword).attached_to, None);
    assert_eq!(t.pt(holder), (3, 3));
    assert_eq!(t.pt(sword), (3, 3));
    // Its equip ability can't attach it to a creature any more.
    t.advance_to(P0, Step::PrecombatMain);
    mana(&mut t, P0, ManaType::C, 1);
    let _ = act(&mut t, P0, sword, "Equip", &[obj(holder)]);
    t.resolve_all();
    assert_eq!(t.obj_now(sword).attached_to, None);
    assert_eq!(t.pt(holder), (3, 3));
}

#[test]
fn the_new_creature_can_attack_if_controlled_since_the_turn_began() {
    cr!("302.6");
    ruling!(
        "Pizza Face, Gastromancer",
        "The resulting creature will be able to attack on your turn if it's been under your control continuously since the turn began."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pizza Face, Gastromancer");
    let sword = t.battlefield(P0, "Bonesplitter");
    disappear_onto(&mut t, sword);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, sword));
}
