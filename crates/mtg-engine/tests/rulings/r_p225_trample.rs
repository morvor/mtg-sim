//! Rulings batch P225 — trample and related combat damage rulings: simultaneous combat
//! damage and state-based actions (CR 510.2, 704.5g), life-gain events from lifelink
//! (CR 702.15b, 119.9), damage events (CR 120), "dealt damage" triggers that see a
//! creature dying at the same time (CR 603.10a), trample assignment with previously
//! dealt damage (CR 702.19b, 702.19d), trample over planeswalkers (CR 702.19c), and effects whose
//! affected set is locked in on resolution (CR 611.2c).

use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::{activate_containing, attach_new, damage};
use crate::r_s10_common::poison;
use crate::r_s17_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const HIDETSUGU: &str = "Hidetsugu Consumes All // Vessel of the All-Consuming";

/// Clears summoning sickness on everything `p` controls.
fn unsick(t: &mut TestGame, p: PlayerId) {
    for id in t.g.battlefield.clone() {
        if t.obj(id).controller == p {
            t.g.objects[id.0 as usize].summoning_sick = false;
        }
    }
}

/// Advances to the combat damage step (damage dealt) and settles state-based actions and
/// triggers.
fn to_damage(t: &mut TestGame) {
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::CombatDamage);
    t.settle();
}

#[test]
fn vishgraz_gets_its_bonus_at_the_same_time_as_its_combat_damage() {
    cr!("510.2", "704.5g", "702.164c");
    ruling!(
        "Vishgraz, the Doomhive",
        "The end result is that it will stay alive and be a 4/4 creature with 3 damage marked on it."
    );
    supported("Vishgraz, the Doomhive");
    let mut t = TestGame::new(2);
    let vish = t.enter(P0, "Vishgraz, the Doomhive");
    t.resolve_all();
    unsick(&mut t, P0);
    let mite = with_subtype(&t, P0, "Mite")[0];
    // (Vishgraz has menace: a 3/3 and a 0/2 block it, dealing 3 damage to it.)
    let giant = t.battlefield(P1, "Hill Giant");
    let thopter = t.battlefield(P1, "Ornithopter");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(vish, Entity::Player(P1)), (mite, Entity::Player(P1))],
        &[(giant, vish), (thopter, vish)],
    );
    assert_eq!(poison(&t, P1), 1);
    assert!(t.on_battlefield(vish));
    assert_eq!(t.pt(vish), (4, 4));
    assert_eq!(t.obj_now(vish).damage, 3);
}

#[test]
fn kavu_predator_triggers_once_per_lifelink_creature() {
    cr!("702.15b", "119.9", "510.2");
    ruling!(
        "Kavu Predator",
        "Each creature with lifelink dealing combat damage is a single life-gaining event."
    );
    supported("Kavu Predator");
    let kavu_triggers = |t: &TestGame| triggers_on_stack(t, "+1/+1 counters");
    // Two unblocked lifelink creatures: two triggers.
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Kavu Predator");
    let a = t.battlefield(P1, "Child of Night");
    let b = t.battlefield(P1, "Child of Night");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(
        &mut t,
        &[(a, Entity::Player(P0)), (b, Entity::Player(P0))],
        &[],
    );
    to_damage(&mut t);
    assert_eq!(kavu_triggers(&t), 2);
    t.resolve_all();
    assert_eq!(t.counters(kavu, counters::PLUS1), 4);
    // One lifelink creature blocked by two creatures: one trigger.
    let mut t = TestGame::new(2);
    let kavu = t.battlefield(P0, "Kavu Predator");
    let a = t.battlefield(P1, "Child of Night");
    let m1 = t.battlefield(P0, "Memnite");
    let m2 = t.battlefield(P0, "Memnite");
    t.set_step(P1, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(a, Entity::Player(P0))], &[(m1, a), (m2, a)]);
    to_damage(&mut t);
    assert_eq!(kavu_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(t.counters(kavu, counters::PLUS1), 2);
}

#[test]
fn tori_buffs_gives_trample_and_untaps_a_red_and_white_attacker() {
    cr!("508.1f", "611.2c");
    ruling!(
        "Tori D'Avenant, Fury Rider",
        "If another attacking creature is both red and white, it will get +1/+1 until end of turn, gain trample until end of turn, and be untapped."
    );
    supported("Tori D'Avenant, Fury Rider");
    let mut t = TestGame::new(2);
    let tori = t.battlefield(P0, "Tori D'Avenant, Fury Rider");
    let rw = t.battlefield(P0, "Boros Swiftblade");
    attack_with(
        &mut t,
        &[(tori, Entity::Player(P1)), (rw, Entity::Player(P1))],
    );
    assert!(t.obj_now(rw).tapped);
    t.resolve_all();
    assert_eq!(t.pt(rw), (2, 3));
    assert!(t.obj_now(rw).has_keyword(KeywordKind::Trample));
    assert!(!t.obj_now(rw).tapped);
}

#[test]
fn scion_of_darkness_can_return_the_blocker_it_killed() {
    cr!("510.2", "603.2", "702.19b");
    ruling!(
        "Scion of Darkness",
        "you will be able to target the destroyed creature (if it was a card and not a token)"
    );
    supported("Scion of Darkness");
    let mut t = TestGame::new(2);
    let scion = t.battlefield(P0, "Scion of Darkness");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(scion, Entity::Player(P1))], &[(bears, scion)]);
    to_damage(&mut t);
    assert_eq!(t.life(P1), 16);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.answer_yes(P0, true);
    t.resolve_all();
    let now = t.g.current(bears);
    assert_eq!(t.zone(now), Zone::Battlefield);
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn mass_pump_triggers_affect_only_creatures_controlled_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Rumbleweed",
        "Creatures you begin to control later in the turn and noncreature permanents that become creatures later in the turn won’t get +3/+3 or gain trample."
    );
    ruling!(
        "Decimator of the Provinces",
        "The set of creatures affected by the triggered ability is determined as the ability resolves."
    );
    ruling!(
        "End-Raze Forerunners",
        "Creatures you begin to control later in the turn won't get +2/+2 or gain vigilance or trample."
    );
    ruling!(
        "Thundering Ceratok",
        "Creatures you begin to control later in the turn won’t gain trample."
    );
    // (card, bonus to a creature already there)
    let cases: [(&str, (i32, i32)); 4] = [
        ("Rumbleweed", (5, 5)),
        ("Decimator of the Provinces", (4, 4)),
        ("End-Raze Forerunners", (4, 4)),
        ("Thundering Ceratok", (2, 2)),
    ];
    for (name, pumped) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let before = t.battlefield(P0, "Grizzly Bears");
        if name == "Decimator of the Provinces" {
            // "When you cast this spell" — cast it.
            give_mana_for(&mut t, P0, name);
            let c = t.hand(P0, name);
            t.cast(P0, c).go();
        } else {
            t.enter(P0, name);
        }
        t.resolve_all();
        assert_eq!(t.pt(before), pumped, "{name}");
        assert!(
            t.obj_now(before).has_keyword(KeywordKind::Trample),
            "{name}"
        );
        // A creature that comes under P0's control later, and an artifact that becomes a
        // creature later, aren't affected.
        let later = t.battlefield(P1, "Grizzly Bears");
        crate::r_s06_common::give_control(&mut t, later, P0);
        assert_eq!(t.pt(later), (2, 2), "{name}");
        assert!(
            !t.obj_now(later).has_keyword(KeywordKind::Trample),
            "{name}"
        );
        if name == "Rumbleweed" {
            let vehicle = t.battlefield(P0, "Smuggler's Copter");
            let pilot = t.battlefield(P0, "Memnite");
            assert!(crate::r_s04_common::crew(&mut t, P0, vehicle, &[pilot]));
            t.resolve_all();
            assert_eq!(t.pt(vehicle), (3, 3));
            assert!(!t.obj_now(vehicle).has_keyword(KeywordKind::Trample));
        }
    }
}

#[test]
fn thrasta_tramples_over_planeswalkers_and_deathtouch_isnt_lethal_to_them() {
    cr!("702.19c", "702.2c", "510.1c");
    ruling!(
        "Thrasta, Tempest's Roar",
        "Deathtouch does not cause damage dealt to a planeswalker to be lethal."
    );
    // (Thrasta's cost reduction doesn't compile; its combat keywords do.)
    let mut t = TestGame::new(2);
    let thrasta = t.battlefield(P0, "Thrasta, Tempest's Roar");
    attach_new(&mut t, P0, "Basilisk Collar", thrasta);
    assert!(t.obj_now(thrasta).has_keyword(KeywordKind::Deathtouch));
    let jace = t.battlefield(P1, "Jace Beleren");
    assert_eq!(t.counters(jace, counters::LOYALTY), 3);
    let from = t.asked().len();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(thrasta, Entity::Object(jace))], &[]);
    // The assignment offered the planeswalker and its controller, with 3 lethal to Jace.
    let asks: Vec<_> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::AssignCombatDamage {
                recipients, lethal, ..
            } => Some((recipients, lethal)),
            _ => None,
        })
        .collect();
    assert_eq!(asks.len(), 1);
    assert_eq!(
        asks[0].0,
        vec![Entity::Object(jace), Entity::Player(P1)],
        "recipients"
    );
    assert_eq!(asks[0].1[0], 3);
    // Default: 3 to Jace, the excess 4 to P1.
    assert!(!t.on_battlefield(jace));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn vessel_triggers_once_per_damage_event_combat_or_not() {
    cr!("120.2", "603.2c", "702.19b");
    ruling!(
        "Hidetsugu Consumes All // Vessel of the All-Consuming",
        "that ability triggers only once"
    );
    let mut t = TestGame::new(2);
    let vessel = enter_transformed(&mut t, P0, HIDETSUGU);
    unsick(&mut t, P0);
    let memnite = t.battlefield(P1, "Memnite");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(
        &mut t,
        &[(vessel, Entity::Player(P1))],
        &[(memnite, vessel)],
    );
    to_damage(&mut t);
    assert_eq!(t.life(P1), 18);
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(vessel, counters::PLUS1), 1);
    // Noncombat damage triggers it too.
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, vessel, 1, bears);
    t.resolve_all();
    assert_eq!(t.counters(vessel, counters::PLUS1), 2);
}

#[test]
fn zacama_can_shoot_its_blocker_and_trample_over_it() {
    cr!("702.19b", "702.19d", "510.1c");
    ruling!(
        "Zacama, Primal Calamity",
        "If the blocking creatures are dealt nonlethal damage, that damage is considered when assigning trample damage."
    );
    supported("Zacama, Primal Calamity");
    // A 6/4 blocker dealt 3 damage: 1 is lethal, 8 tramples over.
    let mut t = TestGame::new(2);
    let zacama = t.battlefield(P0, "Zacama, Primal Calamity");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(zacama, Entity::Player(P1))], &[(wurm, zacama)]);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    activate_containing(&mut t, P0, zacama, "3 damage").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 3);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.life(P1), 12);
    // Its blocker destroyed: all 9 damage to the player.
    let mut t = TestGame::new(2);
    let zacama = t.battlefield(P0, "Zacama, Primal Calamity");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    to_blockers(&mut t, &[(zacama, Entity::Player(P1))], &[(bears, zacama)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, zacama, "3 damage").unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 11);
}
