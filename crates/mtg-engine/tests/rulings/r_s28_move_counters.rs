//! Rulings batch S28 — moving counters (CR 122.5): "{T}: Move a +1/+1 counter from this
//! artifact onto target creature" (Explorer's Cache, Weapon Rack). The counter is removed
//! from one permanent and put on the other; if either can't happen, nothing moves.

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use mtg_engine::testing::*;
use mtg_engine::*;

const PLUS1: &str = "+1/+1";

#[test]
fn moving_a_counter_puts_it_on_the_creature_for_effects_that_care() {
    cr!("122.5", "614.1a");
    ruling!(
        "Explorer's Cache",
        "To move a counter from one permanent to another, the counter is removed from the first permanent and put on the second. Any abilities that care about a counter being removed from or put onto a permanent will apply."
    );
    supported("Explorer's Cache");
    // "This artifact enters with two +1/+1 counters on it." Hardened Scales: "If one or
    // more +1/+1 counters would be put on a creature you control, that many plus one
    // +1/+1 counters are put on it instead."
    let mut t = TestGame::new(2);
    let cache = t.enter(P0, "Explorer's Cache");
    assert_eq!(t.counters(cache, PLUS1), 2);
    t.battlefield(P0, "Hardened Scales");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, cache, 0, &[Entity::Object(bears)])
        .expect("move a counter");
    t.resolve_all();
    assert_eq!(t.counters(cache, PLUS1), 1);
    assert_eq!(t.counters(bears, PLUS1), 2);
}

#[test]
fn nothing_moves_if_the_cache_has_left_the_battlefield() {
    cr!("122.5", "113.7a");
    ruling!(
        "Explorer's Cache",
        "If Explorer's Cache isn't on the battlefield as its activated ability resolves, you can't remove a +1/+1 counter from it, so you won't put a +1/+1 counter on the target creature."
    );
    let mut t = TestGame::new(2);
    let cache = t.enter(P0, "Explorer's Cache");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, cache, 0, &[Entity::Object(bears)])
        .expect("move a counter");
    destroy(&mut t, cache);
    t.resolve_all();
    assert_eq!(t.counters(bears, PLUS1), 0);
}

#[test]
fn weapon_racks_counters_dont_affect_it_and_it_stays_when_empty() {
    cr!("122.5");
    ruling!(
        "Weapon Rack",
        "Once Weapon Rack runs out of +1/+1 counters, it remains on the battlefield. You can activate its last ability, but it won't do anything."
    );
    supported("Weapon Rack");
    // "This artifact enters with three +1/+1 counters on it. {T}: Move a +1/+1 counter
    // from this artifact onto target creature. Activate only as a sorcery."
    let mut t = TestGame::new(2);
    let rack = t.enter(P0, "Weapon Rack");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.remove_counters(Entity::Object(rack), PLUS1, 3);
    t.activate(P0, rack, 0, &[Entity::Object(bears)])
        .expect("it can still be activated");
    t.resolve_all();
    assert!(t.on_battlefield(rack));
    assert_eq!(t.counters(bears, PLUS1), 0);
}

#[test]
fn weapon_rack_keeps_its_counter_if_the_target_is_gone() {
    cr!("122.5", "608.2b");
    ruling!(
        "Weapon Rack",
        "If the creature becomes an illegal target or can't have a +1/+1 counter put onto it for some other reason, you won't remove a +1/+1 counter from Weapon Rack."
    );
    let mut t = TestGame::new(2);
    let rack = t.enter(P0, "Weapon Rack");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, rack, 0, &[Entity::Object(bears)])
        .expect("move a counter");
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(rack, PLUS1), 3);
}
