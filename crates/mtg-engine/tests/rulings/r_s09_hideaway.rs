//! Rulings batch S09 — hideaway (CR 702.75): "When this permanent enters, look at the top
//! N cards of your library. Exile one of them face down and put the rest on the bottom of
//! your library in a random order."

use crate::r_s01_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::zones;
use mtg_engine::*;

#[test]
fn hideaway_n_looks_at_n_cards_and_exiles_one_face_down() {
    cr!("702.75a");
    ruling!(
        "Widespread Thieving",
        "“Hideaway N” means “When this permanent enters the battlefield, look at the top N cards of your library. Exile one of them face down and put the rest on the bottom of your library in a random order."
    );
    supported("Widespread Thieving");
    let mut t = TestGame::new(2);
    let cards = stack_library(
        &mut t,
        P0,
        &[
            "Lightning Bolt",
            "Llanowar Elves",
            "Counterspell",
            "Grizzly Bears",
            "Hill Giant",
            "Island",
        ],
    );
    let size = t.library_size(P0);
    t.enter(P0, "Widespread Thieving");
    t.settle();
    // Hideaway 5: the fifth card from the top (Hill Giant) may be chosen; exile it.
    t.answer_choose(P0, &[Entity::Object(cards[4])]);
    t.resolve_all();
    let exiled = t.g.current(cards[4]);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(t.g.obj(exiled).face_down);
    // Its controller may look at it; the opponent may not.
    assert!(zones::may_look(&t.g, P0, exiled));
    assert!(!zones::may_look(&t.g, P1, exiled));
    // The other four went to the bottom; the sixth card (Island) is now on top.
    assert_eq!(t.library_size(P0), size - 1);
    let lib = &t.g.player(P0).library;
    assert_eq!(*lib.last().unwrap(), cards[5]);
    let mut bottom4: Vec<ObjectId> = lib[..4].to_vec();
    bottom4.sort();
    let mut expected = vec![cards[0], cards[1], cards[2], cards[3]];
    expected.sort();
    assert_eq!(bottom4, expected);
}

#[test]
fn a_permanent_with_hideaway_doesnt_enter_tapped() {
    cr!("702.75a", "702.75b");
    ruling!(
        "Widespread Thieving",
        "Previously, permanents with hideaway entered the battlefield tapped. This ability has been removed from the definition of hideaway."
    );
    let mut t = TestGame::new(2);
    let wt = t.enter(P0, "Widespread Thieving");
    t.settle();
    assert!(!t.obj_now(wt).tapped);
    t.answer_choose(P0, &[]);
    t.resolve_all();
    assert!(!t.obj_now(wt).tapped);
}
