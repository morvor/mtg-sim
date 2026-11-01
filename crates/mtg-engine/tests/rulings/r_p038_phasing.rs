//! Rulings batch P038 — Out of Time: creatures phased out "until ~ leaves the
//! battlefield" (CR 702.26, 610.3).

use crate::r_p038_common::*;
use crate::r_s01_common::supported;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Out of Time enters (its ETB trigger on the stack).
fn out_of_time(t: &mut TestGame) -> ObjectId {
    enter(t, P0, "Out of Time")
}

#[test]
fn phased_out_creatures_stay_out_through_untap_steps_until_it_leaves() {
    cr!("702.26a", "702.26b", "610.3", "702.63a");
    ruling!(
        "Out of Time",
        "A creature phased out by Out of Time doesn't phase in during its controller's untap step as normal."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    tap(&mut t, a);
    let oot = out_of_time(&mut t);
    t.resolve_all();
    assert!(phased_out(&t, a) && phased_out(&t, b));
    assert!(!tapped(&t, a), "untapped before phasing out");
    assert_eq!(t.counters(oot, "time"), 2);
    // P1's untap step, then P0's: neither creature phases in.
    next_p0_upkeep(&mut t);
    t.resolve_all();
    assert!(phased_out(&t, a) && phased_out(&t, b));
    assert_eq!(t.counters(oot, "time"), 1);
    // The last counter is removed: Out of Time is sacrificed and they phase in.
    next_p0_upkeep(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(oot));
    assert!(!phased_out(&t, a) && !phased_out(&t, b));
}

#[test]
fn as_long_as_effects_end_when_their_source_phases_out() {
    cr!("702.26d", "611.2b");
    ruling!(
        "Out of Time",
        "Any continuous effects with a \"for as long as\" duration, such as that of Tide Shaper, ignore phased-out objects."
    );
    supported("Tide Shaper");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let shaper = t.hand(P0, "Tide Shaper");
    t.answer_targets(P0, &[obj(forest)]);
    t.cast(P0, shaper).kicked(true).go();
    t.resolve_all();
    assert!(t.obj_now(forest).chars.has_subtype("Island"));
    let shaper = t.named_on_battlefield("Tide Shaper")[0];
    out_of_time(&mut t);
    t.resolve_all();
    assert!(phased_out(&t, shaper));
    assert!(!t.obj_now(forest).chars.has_subtype("Island"));
    // The effect expired: it doesn't come back when Tide Shaper phases in.
    let oot = t.named_on_battlefield("Out of Time")[0];
    kill(&mut t, oot);
    t.resolve_all();
    assert!(!phased_out(&t, shaper));
    assert!(!t.obj_now(forest).chars.has_subtype("Island"));
    assert!(t.obj_now(forest).chars.has_subtype("Forest"));
}

#[test]
fn phasing_out_is_not_leaving_the_battlefield_for_banisher_priest() {
    cr!("702.26d", "610.3a");
    ruling!(
        "Out of Time",
        "Any one-shot effects that are waiting \"until [this] leaves the battlefield,\" such as that of Banisher Priest, won't happen when a permanent phases out."
    );
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(victim)]);
    let priest = enter(&mut t, P0, "Banisher Priest");
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    out_of_time(&mut t);
    t.resolve_all();
    assert!(phased_out(&t, priest));
    assert!(t.in_exile("Hill Giant"));
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

#[test]
fn out_of_time_as_a_creature_phases_itself_out_forever() {
    cr!("702.26d", "702.63a", "702.26a", "702.26b");
    ruling!(
        "Out of Time",
        "If Out of Time happens to be a creature when its enter the battlefield trigger resolves, it will phase out along with all other creatures."
    );
    supported("Opalescence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opalescence");
    let a = t.battlefield(P1, "Grizzly Bears");
    let oot = out_of_time(&mut t);
    assert!(t.obj_now(oot).is(CardType::Creature));
    t.resolve_all();
    assert!(phased_out(&t, oot) && phased_out(&t, a));
    // It phased out before the counters are put: a phased-out permanent can't be
    // affected (CR 702.26b), so it gets none.
    let counters = t.counters(oot, "time");
    assert_eq!(counters, 0);
    for _ in 0..3 {
        next_p0_upkeep(&mut t);
        t.resolve_all();
    }
    // Vanishing never removes a counter from the phased-out enchantment.
    assert!(t.on_battlefield(oot));
    assert_eq!(t.counters(oot, "time"), counters);
    assert!(phased_out(&t, oot) && phased_out(&t, a));
}

#[test]
fn if_it_leaves_before_its_trigger_resolves_creatures_untap_but_stay() {
    cr!("610.3c", "603.4");
    ruling!(
        "Out of Time",
        "If Out of Time leaves the battlefield before its enter the battlefield trigger resolves, creatures will untap, but they won't phase out."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, a);
    let oot = out_of_time(&mut t);
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, oot);
    t.resolve_all();
    assert!(!tapped(&t, a));
    assert!(!phased_out(&t, a));
}
