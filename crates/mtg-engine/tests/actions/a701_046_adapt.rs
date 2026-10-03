//! CR 701.46: adapt.

use crate::a701_028_071_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn adapt_puts_counters_only_on_a_permanent_without_plus_one_counters() {
    cr!("701.46a");
    ruling!(
        "Aeromunculus",
        "As that ability resolves, if the creature has a +1/+1 counter on it for any reason, you simply won’t put any +1/+1 counters on it."
    );
    supported("Aeromunculus");
    // "{2}{G}{U}: Adapt 1."
    let mut t = TestGame::new(2);
    let aero = t.battlefield(P0, "Aeromunculus");
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Island", 4);
    // Activated twice; the second resolves after the first put a counter on it.
    t.activate(P0, aero, 0, &[]).unwrap();
    t.activate(P0, aero, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(aero, "+1/+1"), 1);
    assert_eq!(t.pt(aero), (3, 4));
    // Other kinds of counters don't matter.
    t.g.add_counters(Entity::Object(aero), "charge", 1, None);
    t.g.remove_counters(Entity::Object(aero), "+1/+1", 1);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 2);
    t.activate(P0, aero, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(aero, "+1/+1"), 1);
}

#[test]
fn a_creature_that_lost_its_counters_can_adapt_again() {
    cr!("701.46a");
    ruling!(
        "Aeromunculus",
        "If a creature somehow loses all of its +1/+1 counters, it can adapt again and get more +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let aero = t.battlefield(P0, "Aeromunculus");
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Island", 4);
    t.activate(P0, aero, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(aero, "+1/+1"), 1);
    t.g.remove_counters(Entity::Object(aero), "+1/+1", 1);
    t.activate(P0, aero, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(aero, "+1/+1"), 1);
}
