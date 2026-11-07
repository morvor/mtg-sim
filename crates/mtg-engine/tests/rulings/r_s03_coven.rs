//! Rulings batch S03 — coven (an ability word): "Coven — ... if you control three or more
//! creatures with different powers, ..."

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FARMHAND: &str = "Ambitious Farmhand // Seasoned Cathar";

/// Whether P0 could activate Ambitious Farmhand's "Coven — {1}{W}{W}: Transform this
/// creature. Activate only if you control three or more creatures with different powers."
/// with `others` (creatures) beside it (the Farmhand is a 1/2).
fn farmhand_can_transform(others: &[&str]) -> bool {
    let mut t = TestGame::new(2);
    let farmhand = t.battlefield(P0, FARMHAND);
    for o in others {
        t.battlefield(P0, o);
    }
    t.lands(P0, "Plains", 3);
    can_activate(&mut t, P0, farmhand)
}

#[test]
fn three_creatures_with_different_powers_need_three_different_powers() {
    cr!("207.2c", "602.5b");
    ruling!(
        "Ambitious Farmhand // Seasoned Cathar",
        "For three creatures to have different powers from one another, each of their powers needs to be different. A 1/1 creature, a 2/1 creature, and another 2/1 creature aren't three creatures with different powers, even though both 2/1 creatures have different power than the 1/1 creature."
    );
    supported(FARMHAND);
    // Powers 1 (the Farmhand), 2 and 2: no.
    assert!(!farmhand_can_transform(&["Grizzly Bears", "Grizzly Bears"]));
    // Powers 1, 2, 2 and 3: yes.
    assert!(farmhand_can_transform(&[
        "Grizzly Bears",
        "Grizzly Bears",
        "Hill Giant"
    ]));
    // A creature an opponent controls doesn't count.
    let mut t = TestGame::new(2);
    let farmhand = t.battlefield(P0, FARMHAND);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 3);
    assert!(!can_activate(&mut t, P0, farmhand));
    // With three different powers, it transforms.
    let mut t = TestGame::new(2);
    let farmhand = t.battlefield(P0, FARMHAND);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Plains", 3);
    t.activate(P0, farmhand, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj(farmhand).chars.name, "Seasoned Cathar");
}

#[test]
fn creatures_have_different_powers_if_their_powers_are_different_numbers() {
    cr!("207.2c", "208.1");
    ruling!(
        "Ambitious Farmhand // Seasoned Cathar",
        "A creature has different power from another if their powers are different numbers. For example, a 1/1 creature and a 2/1 creature have different powers."
    );
    // Powers 0 (Ornithopter), 1 (the Farmhand) and 2 (the Bears) are three different
    // numbers.
    assert!(farmhand_can_transform(&["Ornithopter", "Grizzly Bears"]));
    // It's the creatures' current powers that count: Giant Growth makes one of two Bears
    // a 5/5.
    let mut t = TestGame::new(2);
    let farmhand = t.battlefield(P0, FARMHAND);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    assert!(!can_activate(&mut t, P0, farmhand));
    let growth = t.hand(P0, "Giant Growth");
    t.lands(P0, "Forest", 1);
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert!(can_activate(&mut t, P0, farmhand));
}

#[test]
fn a_coven_trigger_checks_on_triggering_and_resolving_not_necessarily_the_same_creatures() {
    cr!("603.4", "207.2c");
    ruling!(
        "Stalwart Pathlighter",
        "Many coven abilities, such as that of Dawnhart Wardens above, are triggered abilities with intervening if clauses. You must control three or more creatures with different powers at the time the ability triggers and at the time the ability tries to resolve. They do not, however, need to be the same set of creatures in both cases."
    );
    supported("Stalwart Pathlighter");
    // Stalwart Pathlighter (3/1): "Coven — At the beginning of combat on your turn, if you
    // control three or more creatures with different powers, creatures you control gain
    // indestructible until end of turn."
    let setup = || {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Stalwart Pathlighter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let elves = t.battlefield(P0, "Llanowar Elves");
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "indestructible"), 1);
        (t, bears, elves)
    };
    // The Elves (power 1) leave before it resolves: nothing happens.
    let (mut t, bears, elves) = setup();
    destroy(&mut t, elves);
    t.resolve_all();
    assert!(!t.obj(bears).has_keyword(KeywordKind::Indestructible));
    // The Elves leave, but an Ornithopter (power 0) arrives: three different powers
    // again, with another set of creatures.
    let (mut t, bears, elves) = setup();
    destroy(&mut t, elves);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.resolve_all();
    assert!(t.obj(bears).has_keyword(KeywordKind::Indestructible));
    assert!(t.obj(thopter).has_keyword(KeywordKind::Indestructible));
    // With only two different powers as combat begins, it doesn't trigger at all, even if
    // there are three later.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stalwart Pathlighter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "indestructible"), 0);
    t.battlefield(P0, "Llanowar Elves");
    t.resolve_all();
    assert!(!t.obj(bears).has_keyword(KeywordKind::Indestructible));
}
