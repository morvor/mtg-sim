//! Rulings batch S07 — fabricate (CR 702.123): "When this permanent enters, you may put N
//! +1/+1 counters on it. If you don't, create N 1/1 colorless Servo artifact creature
//! tokens."

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn is_yes_no(d: &Decision) -> bool {
    matches!(d, Decision::YesNo { .. })
}

/// (Counters on Visionary Augmenter, Servo tokens) at the time of a decision.
fn augmenter_state(g: &Game) -> (u32, usize) {
    let counters = g
        .find_in_zone(Zone::Battlefield, "Visionary Augmenter")
        .first()
        .map_or(0, |a| g.obj(*a).counter(counters::PLUS1));
    let servos = g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Servo"))
        .count();
    (counters, servos)
}

#[test]
fn the_fabricate_choice_is_made_as_it_resolves() {
    cr!("702.123a", "608.2");
    ruling!(
        "Visionary Augmenter",
        "You choose whether to put +1/+1 counters on the creature or create Servo tokens as the fabricate ability is resolving. No player may take actions between the time you choose and the time that counters are added or tokens are created."
    );
    supported("Visionary Augmenter");
    // Visionary Augmenter: 2/1, fabricate 2.
    let mut t = TestGame::new(2);
    let seen = watch(&mut t, P0, is_yes_no, augmenter_state);
    let from = t.asked().len();
    let va = enter(&mut t, P0, "Visionary Augmenter");
    assert_eq!(t.stack_len(), 1);
    // The ability is on the stack; nothing has been chosen yet.
    assert!(seen.lock().unwrap().is_empty());
    t.answer_yes(P0, true);
    t.resolve();
    // Chosen as it resolved, with neither counters nor tokens yet; right after, the
    // counters are on it, with no priority in between.
    assert_eq!(*seen.lock().unwrap(), vec![(0, 0)]);
    assert_eq!(count_asked(&t, from, is_priority), 0);
    assert_eq!(t.counters(va, counters::PLUS1), 2);
    assert_eq!(t.pt(va), (4, 3));
    // Choosing tokens instead.
    let mut t = TestGame::new(2);
    let va = enter(&mut t, P0, "Visionary Augmenter");
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.counters(va, counters::PLUS1), 0);
    assert_eq!(tokens_with_subtype(&t, P0, "Servo").len(), 2);
}

#[test]
fn a_servo_is_created_if_the_counter_cant_be_put_on_it() {
    cr!("702.123a");
    ruling!(
        "Marionette Apprentice",
        "If you can't put a +1/+1 counter on the creature for any reason as fabricate resolves (for instance, if it's no longer on the battlefield), you just create a Servo token."
    );
    supported("Marionette Apprentice");
    // Marionette Apprentice: 1/2, fabricate 1. It's destroyed with fabricate on the stack.
    let mut t = TestGame::new(2);
    let ma = enter(&mut t, P0, "Marionette Apprentice");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, ma);
    assert!(!t.on_battlefield(ma));
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(count_asked(&t, from, is_yes_no), 0);
    let servos = tokens_with_subtype(&t, P0, "Servo");
    assert_eq!(servos.len(), 1);
    assert_eq!(t.pt(servos[0]), (1, 1));
}
