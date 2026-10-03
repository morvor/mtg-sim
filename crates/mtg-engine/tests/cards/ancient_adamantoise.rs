//! Ancient Adamantoise: "Damage isn't removed from this creature during cleanup steps." (an
//! exception to CR 514.2; see `rule_statics::cleanup_damage`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn damage_stays_marked_through_cleanup() {
    cr!("514.2");
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P0, "Ancient Adamantoise");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 2);
    let b1 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b1).target(turtle).go();
    t.resolve();
    let b2 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b2).target(wurm).go();
    t.resolve();
    assert_eq!(t.obj_now(turtle).damage, 3);
    assert_eq!(t.obj_now(wurm).damage, 3);
    t.advance_to(P1, Step::Upkeep);
    // The Adamantoise keeps its damage; another creature's damage is removed.
    assert_eq!(t.obj_now(turtle).damage, 3);
    assert_eq!(t.obj_now(wurm).damage, 0);
}

#[test]
fn damage_is_removed_while_phased_out() {
    cr!("514.2", "702.26b");
    ruling!("Ancient Adamantoise", "is phased out with damage marked on it");
    let mut t = TestGame::new(2);
    let turtle = t.battlefield(P0, "Ancient Adamantoise");
    t.lands(P0, "Mountain", 1);
    let b1 = t.hand(P0, "Lightning Bolt");
    t.cast(P0, b1).target(turtle).go();
    t.resolve();
    t.g.objects[turtle.0 as usize].phased_out = true;
    t.g.recompute();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(turtle).damage, 0);
}
