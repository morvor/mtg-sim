//! Reflexive triggered abilities on the results of earlier instructions (CR 603.12,
//! 603.12a).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn seasoned_cryomancer_taps_up_to_that_many_creatures() {
    cr!("603.12", "603.12a");
    supported("Seasoned Cryomancer");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Horned Turtle");
    // The two cards drawn are discarded: two nonland cards (fillers have no types).
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    t.answer_choose(P0, &[Entity::Object(bolt), Entity::Object(shock)]);
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.enter(P0, "Seasoned Cryomancer");
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert!(!t.obj_now(c).tapped);
    assert_eq!(t.counters(a, "stun"), 1);
    assert_eq!(t.counters(b, "stun"), 1);
}

#[test]
fn seasoned_cryomancer_no_trigger_when_only_lands_are_discarded() {
    cr!("603.12");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let f1 = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Island");
    t.answer_choose(P0, &[Entity::Object(f1), Entity::Object(f2)]);
    t.enter(P0, "Seasoned Cryomancer");
    t.g.flush_events();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(!t.obj_now(a).tapped);
}

#[test]
fn kitt_kanto_goads_the_reflexive_target() {
    cr!("603.12", "701.15a");
    supported("Kitt Kanto, Mayhem Diva");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kitt Kanto, Mayhem Diva");
    let x = t.battlefield(P0, "Grizzly Bears");
    let y = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Horned Turtle");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(x), Entity::Object(y)]);
    t.answer_targets(P0, &[Entity::Object(theirs)]);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(x).tapped && t.obj_now(y).tapped);
    assert_eq!(t.pt(theirs), (3, 6));
    assert_eq!(t.obj_now(theirs).goaded_by, vec![P0]);
}
