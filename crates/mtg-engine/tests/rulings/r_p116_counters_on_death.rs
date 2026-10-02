//! Rulings batch P116 — Buzzard-Wasp Colony: "Whenever another creature you control dies,
//! if it had counters on it, put its counters on this creature." (CR 603.10a: leaves-the-
//! battlefield abilities look back in time; CR 122.1: counters of each kind.)

use crate::r_p116_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// A Buzzard-Wasp Colony for P0 (no enters trigger).
fn colony(t: &mut TestGame) -> ObjectId {
    supported("Buzzard-Wasp Colony");
    t.battlefield(P0, "Buzzard-Wasp Colony")
}

#[test]
fn colony_gets_the_same_counters_rather_than_moving_them() {
    cr!("603.10a", "122.1", "700.4");
    ruling!(
        "Buzzard-Wasp Colony",
        "doesn't cause you to move counters from the creature that died onto it. Rather, you put the same number of each kind of counter"
    );
    ruling!(
        "Buzzard-Wasp Colony",
        "puts all counters that were on the creature that died onto Buzzard-Wasp Colony, not just its +1/+1 counters."
    );
    let mut t = TestGame::new(2);
    let c = colony(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 2);
    put_counters(&mut t, bears, "oil", 3);
    put_counters(&mut t, bears, "stun", 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(c, "+1/+1"), 2);
    assert_eq!(t.counters(c, "oil"), 3);
    assert_eq!(t.counters(c, "stun"), 1);
    assert_eq!(t.pt(c), (4, 4));
    // A creature without counters: nothing.
    let mut t = TestGame::new(2);
    let c = colony(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 0);
    assert!(t.obj_now(c).counters.is_empty());
}

#[test]
fn colony_gets_minus_counters_too_and_may_die() {
    cr!("603.10a", "704.5f");
    ruling!(
        "Buzzard-Wasp Colony",
        "If another creature you control has -1/-1 counters on it when it dies, Buzzard-Wasp Colony's ability will include those as well. This may result in Buzzard-Wasp Colony dying."
    );
    let mut t = TestGame::new(2);
    let c = colony(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "-1/-1", 2);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.on_battlefield(c));
    assert!(t.in_graveyard(P0, "Buzzard-Wasp Colony"));
}

#[test]
fn colony_sees_plus_and_minus_counters_put_at_once() {
    cr!("603.10a", "704.5f", "704.5q");
    ruling!(
        "Buzzard-Wasp Colony",
        "Buzzard-Wasp Colony's last ability will see all of the +1/+1 counters it had when it died as well as the -1/-1 counters it had"
    );
    let mut t = TestGame::new(2);
    let c = colony(&mut t);
    put_counters(&mut t, c, "+1/+1", 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 1);
    // Three -1/-1 counters at once: 0/0, it dies before the counters annihilate.
    put_counters(&mut t, bears, "-1/-1", 3);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.stack_len(), 1);
    t.g.resolve_top();
    // Before state-based actions: one +1/+1 counter and three -1/-1 counters were added.
    assert_eq!(t.counters(c, "+1/+1"), 6);
    assert_eq!(t.counters(c, "-1/-1"), 3);
    t.settle();
    assert_eq!(t.counters(c, "+1/+1"), 3);
    assert_eq!(t.counters(c, "-1/-1"), 0);
    assert_eq!(t.pt(c), (5, 5));
}

#[test]
fn two_colonies_each_get_the_counters() {
    cr!("603.10a", "603.2");
    ruling!(
        "Buzzard-Wasp Colony",
        "if you control two Buzzard-Wasp Colonies when a creature dies, you'll put the appropriate number of each kind of counter onto both of them."
    );
    let mut t = TestGame::new(2);
    let a = colony(&mut t);
    let b = colony(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 2);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(a, "+1/+1"), 2);
    assert_eq!(t.counters(b, "+1/+1"), 2);
}
