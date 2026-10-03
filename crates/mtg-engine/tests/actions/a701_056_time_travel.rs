//! CR 701.56: time travel.

use crate::a701_028_071_common::*;
use mtg_engine::ability::*;
use mtg_engine::kwa::time_travel::candidates;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn time_travel_adds_or_removes_a_time_counter_on_each_chosen_object() {
    cr!("701.56a");
    ruling!(
        "The Wedding of River Song",
        "For each of them, you choose whether you want to put a time counter on that card or permanent, remove a time counter from it, or do neither."
    );
    supported("Wibbly-wobbly, Timey-wimey");
    let mut t = TestGame::new(2);
    // Permanents you control with time counters.
    let a = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(a), "time", 2, None);
    let b = t.battlefield(P0, "Hill Giant");
    t.g.add_counters(Entity::Object(b), "time", 2, None);
    // A suspended card you own in exile.
    let bolt = t.exile(P0, "Rift Bolt");
    t.g.add_counters(Entity::Object(bolt), "time", 3, None);
    // Not candidates: a permanent without time counters, an opponent's permanent, an
    // exiled card without suspend, and another player's suspended card.
    let none = t.battlefield(P0, "Llanowar Elves");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(theirs), "time", 2, None);
    let exiled = t.exile(P0, "Lightning Bolt");
    t.g.add_counters(Entity::Object(exiled), "time", 2, None);
    let their_bolt = t.exile(P1, "Rift Bolt");
    t.g.add_counters(Entity::Object(their_bolt), "time", 2, None);
    t.g.recompute();
    let c = candidates(&t.g, P0);
    assert_eq!(c, vec![a, b, bolt]);
    let _ = none;
    // Put one on the Bears, remove one from the Giant, do neither for Rift Bolt.
    option(&mut t, P0, 0);
    option(&mut t, P0, 1);
    option(&mut t, P0, 2);
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Wibbly-wobbly, Timey-wimey");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.counters(a, "time"), 3);
    assert_eq!(t.counters(b, "time"), 1);
    assert_eq!(t.counters(bolt, "time"), 3);
    assert_eq!(t.counters(theirs, "time"), 2);
    assert_eq!(t.counters(exiled, "time"), 2);
    assert_eq!(t.counters(their_bolt, "time"), 2);
    assert_eq!(t.zone(bolt), Zone::Exile);
}
