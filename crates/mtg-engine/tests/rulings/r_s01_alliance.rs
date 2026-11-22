//! Rulings batch S01 — alliance (an ability word, CR 207.2c): "Whenever another creature
//! you control enters, [effect]."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn simultaneous_entries_choose_different_modes_and_only_three_are_chosen() {
    cr!("700.2b", "603.3c");
    ruling!(
        "Gala Greeters",
        "If multiple creatures enter the battlefield simultaneously, you must still choose different modes for each instance of the triggered ability that's put onto the stack. If more than three creatures enter the battlefield simultaneously, that choice is made only for the first three."
    );
    supported("Gala Greeters");
    supported("Secure the Wastes");
    let mut t = TestGame::new(2);
    // "Whenever another creature you control enters, choose one that hasn't been chosen
    // this turn — • Put a +1/+1 counter on this creature. • Create a tapped Treasure
    // token. • You gain 2 life."
    let greeters = t.battlefield(P0, "Gala Greeters");
    t.lands(P0, "Plains", 5);
    // "Create X 1/1 white Warrior creature tokens." Four Warriors enter at once.
    let wastes = t.hand(P0, "Secure the Wastes");
    t.cast(P0, wastes).x(4).go();
    t.resolve();
    // Four abilities triggered; only three could be put on the stack, each with a mode
    // that hadn't been chosen.
    assert_eq!(t.stack_len(), 3);
    let chosen: Vec<Vec<usize>> = t
        .g
        .stack
        .iter()
        .map(|s| {
            t.g.obj(*s)
                .stack
                .as_ref()
                .unwrap()
                .chosen
                .iter()
                .filter_map(|c| c.mode)
                .collect()
        })
        .collect();
    let mut all: Vec<usize> = chosen.iter().flatten().copied().collect();
    all.sort();
    assert_eq!(all, vec![0, 1, 2]);
    t.resolve_all();
    assert_eq!(t.counters(greeters, "+1/+1"), 1);
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
    assert_eq!(t.life(P0), 22);
}
