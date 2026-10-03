//! Rulings batch S29 — counters on players (CR 122.1): energy, experience, poison, rad,
//! and ticket counters; "Each opponent loses all counters." (Final Act); modes performed
//! in the order written (CR 700.2, 608.2c).

use crate::r_s01_common::supported;
use crate::r_s29_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const KINDS: [&str; 5] = [
    counters::ENERGY,
    counters::EXPERIENCE,
    counters::POISON,
    counters::RAD,
    counters::TICKET,
];

#[test]
fn each_opponent_loses_every_kind_of_counter_players_can_have() {
    cr!("122.1", "700.2");
    ruling!(
        "Final Act",
        "Counters that players could have include energy counters, experience counters, poison counters, rad counters, and ticket counters."
    );
    supported("Final Act");
    // Final Act, fifth mode: "Each opponent loses all counters."
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        for (i, k) in KINDS.iter().enumerate() {
            t.g.add_counters(Entity::Player(p), k, i as u32 + 1, None);
        }
    }
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, counters::PLUS1, 1);
    choose_modes(&mut t, P0, &[4]);
    cast_and_resolve(&mut t, P0, "Final Act", &[]);
    for p in [P1, P2] {
        for k in KINDS {
            assert_eq!(t.player(p).counter(k), 0, "{p} {k}");
        }
    }
    // Not P0's, nor counters on permanents.
    for (i, k) in KINDS.iter().enumerate() {
        assert_eq!(t.player(P0).counter(k), i as u32 + 1);
    }
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn final_acts_modes_are_performed_in_the_order_written() {
    cr!("700.2", "608.2c");
    ruling!(
        "Final Act",
        "If you choose more than one mode for Final Act, you perform the actions in the order written."
    );
    // "Destroy all creatures." then "Exile all graveyards.": the destroyed creatures are
    // exiled with the graveyards.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.graveyard(P1, "Lightning Bolt");
    choose_modes(&mut t, P0, &[0, 3]);
    cast_and_resolve(&mut t, P0, "Final Act", &[]);
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Exile);
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Exile);
    assert!(t.in_exile("Lightning Bolt"));
    // Final Act itself goes to the graveyard after it has resolved.
    assert!(t.in_graveyard(P0, "Final Act"));
    assert_eq!(t.graveyard_size(P1), 0);
}
