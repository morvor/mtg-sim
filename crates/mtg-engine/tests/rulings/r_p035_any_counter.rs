//! Rulings batch P035 — abilities that look for "a counter" or count "counters" mean
//! counters of any kind, not only +1/+1 counters (CR 122.1).

use crate::r_p035_common::*;
use crate::r_s01_common::{supported, with_subtype};
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s04_common::{ability_targets, add_mana};
use crate::r_s06_common::attach_new;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn kyler_counts_every_kind_of_counter() {
    cr!("122.1", "613.4c");
    ruling!(
        "Kyler, Sigardian Emissary",
        "Kyler's second ability counts all counters, not just +1/+1 counters."
    );
    supported("Kyler, Sigardian Emissary");
    let mut t = TestGame::new(2);
    let kyler = t.battlefield(P0, "Kyler, Sigardian Emissary");
    let human = t.battlefield(P0, "Elite Vanguard");
    assert_eq!(t.pt(human), (2, 1));
    put(&mut t, kyler, counters::PLUS1, 1);
    put(&mut t, kyler, counters::SHIELD, 1);
    put(&mut t, kyler, counters::CHARGE, 2);
    // Four counters: other Humans get +4/+4.
    assert_eq!(t.pt(human), (6, 5));
}

#[test]
fn thirteenth_doctor_untaps_creatures_with_any_counter() {
    cr!("122.1");
    ruling!(
        "The Thirteenth Doctor",
        "The Thirteenth Doctor's last ability untaps creatures you control with any kind of counter on them, not just creatures you control with +1/+1 counters on them."
    );
    supported("The Thirteenth Doctor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Thirteenth Doctor");
    let stunned = t.battlefield(P0, "Grizzly Bears");
    let oiled = t.battlefield(P0, "Hill Giant");
    let plain = t.battlefield(P0, "Elite Vanguard");
    for id in [stunned, oiled, plain] {
        t.g.objects[id.0 as usize].tapped = true;
    }
    put(&mut t, oiled, counters::OIL, 1);
    put(&mut t, stunned, counters::SHIELD, 1);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert!(!t.obj_now(oiled).tapped, "oil counter: untapped");
    assert!(!t.obj_now(stunned).tapped, "shield counter: untapped");
    assert!(t.obj_now(plain).tapped, "no counter: stays tapped");
}

#[test]
fn hunter_of_eyeblights_targets_a_creature_with_any_counter() {
    cr!("122.1", "115.1c");
    ruling!(
        "Hunter of Eyeblights",
        "The activated ability can target a creature with any kind of counter on it, not just a +1/+1 counter."
    );
    supported("Hunter of Eyeblights");
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Hunter of Eyeblights");
    let charged = t.battlefield(P1, "Grizzly Bears");
    let plain = t.battlefield(P1, "Hill Giant");
    put(&mut t, charged, counters::CHARGE, 1);
    t.lands(P0, "Swamp", 3);
    // Only the creature with a counter (of any kind) is a legal target.
    let legal = ability_targets(&mut t, hunter, 0);
    assert!(legal.contains(&Entity::Object(charged)));
    assert!(!legal.contains(&Entity::Object(plain)));
    t.activate(P0, hunter, 0, &[Entity::Object(charged)])
        .expect("a creature with a charge counter is a legal target");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(plain));
}

#[test]
fn cathedral_acolyte_ward_for_any_counter_and_stays_after_it_leaves() {
    cr!("122.1", "702.21a", "113.7a");
    ruling!(
        "Cathedral Acolyte",
        "The effect of Cathedral Acolyte’s first ability applies to any creature you control with a counter on it, not just ones with +1/+1 counters."
    );
    ruling!(
        "Cathedral Acolyte",
        "Once a creature’s ward ability has triggered, causing that creature to lose ward by removing Cathedral Acolyte won’t affect the ability."
    );
    supported("Cathedral Acolyte");
    // A creature with a charge counter has ward {1}.
    let mut t = TestGame::new(2);
    let acolyte = t.battlefield(P0, "Cathedral Acolyte");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::CHARGE, 1);
    opponent_bolts(&mut t, bears);
    assert_eq!(t.stack_len(), 2, "ward triggered");
    // Removing the Acolyte now doesn't stop the ward ability.
    destroy(&mut t, acolyte);
    t.resolve_all();
    assert!(
        t.on_battlefield(bears),
        "Bolt countered: P1 couldn't pay {{1}}"
    );
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // Without a counter, no ward.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cathedral Acolyte");
    let bears = t.battlefield(P0, "Grizzly Bears");
    opponent_bolts(&mut t, bears);
    assert_eq!(t.stack_len(), 1, "no ward trigger");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn innkeepers_talent_ward_survives_losing_it() {
    cr!("702.21a", "113.7a");
    ruling!(
        "Innkeeper's Talent",
        "Once a ward ability of a permanent with a counter on it has triggered, causing that permanent to lose ward by removing Innkeeper's Talent or removing the counters from that permanent won't affect that ability."
    );
    supported("Innkeeper's Talent");
    for remove_class in [true, false] {
        let mut t = TestGame::new(2);
        let class = t.battlefield(P0, "Innkeeper's Talent");
        mtg_engine::classes::set_level(&mut t.g, class, 2);
        t.g.recompute();
        let bears = t.battlefield(P0, "Grizzly Bears");
        put(&mut t, bears, counters::PLUS1, 1);
        opponent_bolts(&mut t, bears);
        assert_eq!(t.stack_len(), 2, "ward triggered");
        if remove_class {
            destroy(&mut t, class);
        } else {
            let b = t.g.current(bears);
            t.g.remove_counters(Entity::Object(b), counters::PLUS1, 1);
            t.g.recompute();
            t.g.flush_events();
        }
        t.resolve_all();
        assert!(
            t.on_battlefield(bears),
            "remove class {remove_class}: Bolt countered anyway"
        );
        assert!(t.in_graveyard(P1, "Lightning Bolt"));
    }
}

#[test]
fn kinsbaile_borderguard_counts_every_kind_of_counter() {
    cr!("122.1", "603.10a");
    ruling!(
        "Kinsbaile Borderguard",
        "The last ability puts a token creature onto the battlefield for each counter of any kind that was on Kinsbaile Borderguard when it left the battlefield"
    );
    supported("Kinsbaile Borderguard");
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Kinsbaile Borderguard");
    put(&mut t, guard, counters::PLUS1, 1);
    put(&mut t, guard, counters::CHARGE, 2);
    destroy(&mut t, guard);
    t.resolve_all();
    let kithkin: Vec<_> = with_subtype(&t, P0, "Kithkin")
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .collect();
    assert_eq!(kithkin.len(), 3, "one token per counter of any kind");
}

#[test]
fn kulrath_knight_checks_any_kind_of_counter() {
    cr!("122.1", "508.1c");
    ruling!(
        "Kulrath Knight",
        "This checks your opponents' creatures for any kind of counters, not just -1/-1 counters."
    );
    supported("Kulrath Knight");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kulrath Knight");
    let charged = t.battlefield(P1, "Grizzly Bears");
    let plus = t.battlefield(P1, "Hill Giant");
    let plain = t.battlefield(P1, "Elite Vanguard");
    put(&mut t, charged, counters::CHARGE, 1);
    put(&mut t, plus, counters::PLUS1, 1);
    t.set_step(P1, mtg_engine::turn::Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, charged), "charge counter");
    assert!(!can_attack(&mut t, plus), "+1/+1 counter");
    assert!(can_attack(&mut t, plain), "no counters");
}

#[test]
fn unwilling_vessel_counts_plus_and_minus_counters_when_it_dies() {
    cr!("704.5f", "704.5q", "704.3", "603.10a");
    ruling!(
        "Unwilling Vessel",
        "use the total number of counters that were on Unwilling Vessel when it died, including all of the +1/+1 and -1/-1 counters"
    );
    supported("Unwilling Vessel");
    let mut t = TestGame::new(2);
    let vessel = t.battlefield(P0, "Unwilling Vessel");
    put(&mut t, vessel, counters::PLUS1, 1);
    assert_eq!(t.pt(vessel), (4, 3));
    // Three -1/-1 counters: 1/0. It dies at the same time as the counters annihilate.
    put(&mut t, vessel, counters::MINUS1, 3);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Unwilling Vessel"));
    let spirits = with_subtype(&t, P0, "Spirit");
    assert_eq!(spirits.len(), 1);
    assert_eq!(t.pt(spirits[0]), (4, 4), "X counts all four counters");
}

#[test]
fn banshees_blade_keeps_its_counters_when_moved() {
    cr!("122.1", "702.6a");
    ruling!(
        "Banshee's Blade",
        "The counters stay on Banshee's Blade even if it becomes unattached, or moves from one creature to another."
    );
    supported("Banshee's Blade");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let blade = attach_new(&mut t, P0, "Banshee's Blade", a);
    put(&mut t, blade, counters::CHARGE, 2);
    assert_eq!(t.pt(a), (4, 4));
    // Equip to the other creature (a real equip activation).
    add_mana(&mut t, P0, ManaType::C, 2);
    t.activate(P0, blade, 0, &[Entity::Object(b)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(blade, counters::CHARGE), 2, "counters stay");
    assert_eq!(t.pt(a), (2, 2));
    assert_eq!(t.pt(b), (5, 5));
    // Unattached: still there.
    let blade_now = t.g.current(blade);
    t.g.unattach(blade_now);
    t.g.recompute();
    assert_eq!(t.counters(blade, counters::CHARGE), 2);
}
