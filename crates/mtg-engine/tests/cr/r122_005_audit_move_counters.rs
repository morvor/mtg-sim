//! CR 122.5 (found with the CR 702.26 phasing audit): moving a counter removes it from one
//! object and puts it onto another; if either can't be done — the second object can't have
//! counters put onto it — no counter is removed or put anywhere.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn no_counter_moves_onto_a_creature_that_cant_have_counters_put_on_it() {
    cr!("122.5", "702.58a");
    ruling!(
        "Melira's Keepers",
        "Melira's Keepers can't have any kind of counters placed on it, not just -1/-1 counters."
    );
    for name in ["Simic Initiate", "Melira's Keepers"] {
        assert!(card(name).is_fully_supported(), "{name}");
    }
    let mut t = TestGame::new(2);
    // Graft 1: a 0/0 that enters with a +1/+1 counter, and "whenever another creature
    // enters, you may move a +1/+1 counter from this creature onto it".
    let initiate = t.enter(P0, "Simic Initiate");
    t.resolve_all();
    assert_eq!(t.counters(initiate, counters::PLUS1), 1);
    // "This creature can't have counters put on it."
    t.answer_yes(P0, true);
    let keepers = t.enter(P0, "Melira's Keepers");
    t.resolve_all();
    assert_eq!(t.counters(keepers, counters::PLUS1), 0);
    assert!(t.on_battlefield(initiate), "the Initiate keeps its counter and lives");
    assert_eq!(t.counters(initiate, counters::PLUS1), 1);
}
