//! CR 702.32 Fading.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use crate::common_k702_027_037::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FADING: &str = "Fading";
const FADE: &str = "fade";

#[test]
fn a_permanent_with_fading_enters_with_fade_counters() {
    cr!("702.32", "702.32a");
    assert_supported("Skyshroud Ridgeback");
    assert_supported("Rusting Golem");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let ridgeback = t.hand(P0, "Skyshroud Ridgeback");
    t.cast(P0, ridgeback).go();
    t.resolve();
    assert!(t.on_battlefield(ridgeback));
    assert_eq!(t.counters(ridgeback, FADE), 2);
    // Put onto the battlefield by an effect: still enters with them.
    let golem = t.enter(P0, "Rusting Golem");
    assert_eq!(t.counters(golem, FADE), 5);
    assert_eq!(t.pt(golem), (5, 5));
}

#[test]
fn a_fade_counter_is_removed_each_upkeep_until_it_cant_be() {
    cr!("702.32a");
    assert_supported("Blastoderm");
    let mut t = TestGame::new(2);
    let derm = t.enter(P0, "Blastoderm");
    assert_eq!(t.counters(derm, FADE), 3);
    for left in [2, 1, 0] {
        next_upkeep(&mut t, P0);
        assert_eq!(triggers_on_stack(&t, FADING), 1);
        t.resolve();
        assert!(t.on_battlefield(derm));
        assert_eq!(t.counters(derm, FADE), left);
    }
    // No fade counter to remove: it's sacrificed.
    next_upkeep(&mut t, P0);
    t.resolve();
    assert!(!t.on_battlefield(derm));
    assert!(t.in_graveyard(P0, "Blastoderm"));
}

#[test]
fn fading_triggers_only_in_its_controllers_upkeep() {
    cr!("702.32a");
    let mut t = TestGame::new(2);
    let derm = t.enter(P0, "Blastoderm");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_on_stack(&t, FADING), 0);
    assert_eq!(t.counters(derm, FADE), 3);
}

#[test]
fn fade_counters_removed_another_way_hasten_the_sacrifice() {
    cr!("702.32a");
    assert_supported("Phyrexian Prowler");
    let mut t = TestGame::new(2);
    let prowler = t.enter(P0, "Phyrexian Prowler");
    // "Remove a fade counter from ~: ~ gets +1/+1 until end of turn", three times.
    for _ in 0..3 {
        t.activate(P0, prowler, 0, &[]).unwrap();
        t.resolve();
    }
    assert_eq!(t.counters(prowler, FADE), 0);
    next_upkeep(&mut t, P0);
    t.resolve();
    assert!(!t.on_battlefield(prowler));
}

#[test]
fn a_permanent_that_somehow_has_no_fade_counters_is_sacrificed() {
    cr!("702.32a");
    let mut t = TestGame::new(2);
    // Put directly onto the battlefield without entering (no fade counters).
    let ridgeback = t.battlefield(P0, "Skyshroud Ridgeback");
    assert_eq!(t.counters(ridgeback, FADE), 0);
    next_upkeep(&mut t, P0);
    t.resolve();
    assert!(!t.on_battlefield(ridgeback));
}

#[test]
fn each_instance_of_fading_works_separately() {
    cr!("702.32a");
    let mut t = TestGame::new(2);
    let def = custom_card(
        "Twice-Faded Wisp",
        "Creature — Spirit",
        Some((1, 1)),
        "Fading 2\nFading 1",
    );
    let id = t.custom(P0, def, Zone::Nowhere);
    let wisp = t
        .g
        .move_object_ev(mtg_engine::replacement::MoveEv {
            obj: id,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(P0),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(P0),
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    // Enters with 2 + 1 fade counters; two counters are removed each upkeep.
    assert_eq!(t.counters(wisp, FADE), 3);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, FADING), 2);
    t.resolve_all();
    assert_eq!(t.counters(wisp, FADE), 1);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    // One counter removed, then the other instance can't remove one: sacrificed.
    assert!(!t.on_battlefield(wisp));
}
