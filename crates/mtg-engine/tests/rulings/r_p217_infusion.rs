//! Rulings batch P217 — infusion (an ability word, CR 207.2c): "if you gained life this
//! turn" is checked as the ability resolves, not when it triggers.

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s05_common::enter;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// P0 gains `n` life (as a resolving effect would) and settles.
fn gain(t: &mut TestGame, n: u32) {
    t.g.gain_life(P0, n);
    t.g.flush_events();
    t.settle();
}

#[test]
fn old_growth_educator_triggers_regardless_and_checks_on_resolution() {
    cr!("603.4", "608.2c", "207.2c");
    ruling!(
        "Old-Growth Educator",
        "The infusion ability will trigger when Old-Growth Educator enters regardless of whether you've gained life this turn. As that ability resolves, it will check to see if you've gained life this turn. If you have, you'll put two +1/+1 counters on Old-Growth Educator."
    );
    supported("Old-Growth Educator");
    // No life gained: it triggers, and resolves doing nothing.
    let mut t = TestGame::new(2);
    let edu = enter(&mut t, P0, "Old-Growth Educator");
    assert_eq!(triggers_on_stack(&t, "gained life"), 1);
    t.resolve_all();
    assert_eq!(t.counters(edu, counters::PLUS1), 0);
    // Life gained in response: two +1/+1 counters.
    let mut t = TestGame::new(2);
    let edu = enter(&mut t, P0, "Old-Growth Educator");
    assert_eq!(triggers_on_stack(&t, "gained life"), 1);
    gain(&mut t, 1);
    t.resolve_all();
    assert_eq!(t.counters(edu, counters::PLUS1), 2);
}

#[test]
fn poisoners_apprentice_triggers_and_targets_regardless_and_checks_on_resolution() {
    cr!("603.4", "608.2c", "603.3d");
    ruling!(
        "Poisoner's Apprentice",
        "The infusion ability will trigger when Poisoner's Apprentice enters regardless of whether you've gained life this turn, and you'll choose a target. As that ability resolves, it will check to see if you've gained life this turn. If you have, the target creature will get -4/-4 until end of turn."
    );
    supported("Poisoner's Apprentice");
    // No life gained: the target is still chosen, and nothing happens.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    enter(&mut t, P0, "Poisoner's Apprentice");
    assert_eq!(triggers_on_stack(&t, "gained life"), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(
        crate::r_s25_common::targets_of(&t, top),
        vec![Entity::Object(giant)]
    );
    t.resolve_all();
    assert_eq!(t.pt(giant), (6, 4));
    // Life gained in response: -4/-4.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    enter(&mut t, P0, "Poisoner's Apprentice");
    gain(&mut t, 1);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Graveyard(P1));
}

#[test]
fn tragedy_feaster_triggers_regardless_and_checks_on_resolution() {
    cr!("603.4", "608.2c", "513.1");
    ruling!(
        "Tragedy Feaster",
        "Tragedy Feaster's infusion ability will trigger regardless of whether you've gained life this turn. As the ability resolves, it will check to see if you've gained life this turn. If you have, you won't sacrifice a permanent."
    );
    supported("Tragedy Feaster");
    // Life gained in response to the trigger: no sacrifice.
    let mut t = TestGame::new(2);
    let feaster = t.battlefield(P0, "Tragedy Feaster");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "gained life"), 1);
    gain(&mut t, 1);
    t.resolve_all();
    assert!(t.on_battlefield(feaster));
    // No life gained: P0 sacrifices a permanent (the Feaster is the only one).
    let mut t = TestGame::new(2);
    let feaster = t.battlefield(P0, "Tragedy Feaster");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "gained life"), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(feaster));
}
