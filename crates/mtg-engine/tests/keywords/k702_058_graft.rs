//! CR 702.58 Graft.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_052_066::*;
use mtg_engine::testing::*;
use mtg_engine::*;

const GRAFT: &str = "Graft";
const PLUS1: &str = "+1/+1";

#[test]
fn a_permanent_with_graft_enters_with_plus1_counters() {
    cr!("702.58", "702.58a");
    assert_supported("Simic Initiate");
    assert_supported("Cytospawn Shambler");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let initiate = t.hand(P0, "Simic Initiate");
    t.cast(P0, initiate).go();
    t.resolve();
    assert!(t.on_battlefield(initiate));
    assert_eq!(t.counters(initiate, PLUS1), 1);
    assert_eq!(t.pt(initiate), (1, 1));
    let shambler = t.enter(P0, "Cytospawn Shambler");
    assert_eq!(t.counters(shambler, PLUS1), 6);
    assert_eq!(t.pt(shambler), (6, 6));
}

#[test]
fn a_counter_may_be_moved_onto_another_creature_that_enters() {
    cr!("702.58a");
    assert_supported("Sporeback Troll");
    let mut t = TestGame::new(2);
    let troll = t.enter(P0, "Sporeback Troll");
    assert_eq!(t.pt(troll), (2, 2));
    // Any other creature, including an opponent's.
    let bears = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(stack_triggers(&t, GRAFT).len(), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.pt(troll), (1, 1));
    assert_eq!(t.counters(bears, PLUS1), 1);
    assert_eq!(t.pt(bears), (3, 3));
    // The move is optional.
    let bears2 = t.enter(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.pt(troll), (1, 1));
    assert_eq!(t.counters(bears2, PLUS1), 0);
}

#[test]
fn moving_the_last_counter_off_a_zero_zero_creature_kills_it() {
    cr!("702.58a");
    let mut t = TestGame::new(2);
    let initiate = t.enter(P0, "Simic Initiate");
    let bears = t.enter(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!t.on_battlefield(initiate));
    assert!(t.in_graveyard(P0, "Simic Initiate"));
}

#[test]
fn graft_triggers_only_for_other_creatures_and_only_with_a_counter() {
    cr!("702.58a");
    let mut t = TestGame::new(2);
    // Its own entering doesn't trigger it.
    let troll = t.enter(P0, "Sporeback Troll");
    t.settle();
    assert!(stack_triggers(&t, GRAFT).is_empty());
    // Noncreature permanents don't trigger it.
    t.enter(P0, "Glorious Anthem");
    t.settle();
    assert!(stack_triggers(&t, GRAFT).is_empty());
    // With no +1/+1 counter on it, it doesn't trigger (intervening "if").
    remove_counters(&mut t, troll, PLUS1, 2);
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert!(stack_triggers(&t, GRAFT).is_empty());
}

#[test]
fn graft_does_nothing_if_the_counter_is_gone_on_resolution() {
    cr!("702.58a");
    let mut t = TestGame::new(2);
    let troll = t.enter(P0, "Sporeback Troll");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(stack_triggers(&t, GRAFT).len(), 1);
    remove_counters(&mut t, troll, PLUS1, 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(bears, PLUS1), 0);
}

#[test]
fn a_land_with_graft_moves_its_counter_onto_a_creature() {
    cr!("702.58a");
    ruling!("Llanowar Reborn", "won't affect Llanowar Reborn in any way");
    assert_supported("Llanowar Reborn");
    let mut t = TestGame::new(2);
    let reborn = t.enter(P0, "Llanowar Reborn");
    assert_eq!(t.counters(reborn, PLUS1), 1);
    assert!(t.obj_now(reborn).tapped);
    assert!(!t.obj_now(reborn).is_creature());
    let bears = t.enter(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(reborn, PLUS1), 0);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn each_instance_of_graft_works_separately() {
    cr!("702.58b");
    let mut t = TestGame::new(2);
    let def = custom_card(
        "Doubly Grafted Mutant",
        "Creature — Mutant",
        Some((0, 0)),
        "Graft 2\nGraft 1",
    );
    let mutant = enter_def(&mut t, P0, def);
    // Each instance adds its counters.
    assert_eq!(t.counters(mutant, PLUS1), 3);
    // Each instance triggers and may move a counter.
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(stack_triggers(&t, GRAFT).len(), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.counters(mutant, PLUS1), 1);
    assert_eq!(t.counters(bears, PLUS1), 2);
}
