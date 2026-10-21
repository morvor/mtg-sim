//! Fear of Sleep Paralysis (hand-written, `src/cards/fear_of_sleep_paralysis.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn opponents_stunned_permanents_stay_tapped_and_keep_the_counter() {
    cr!("122.1d");
    ruling!("Fear of Sleep Paralysis", "it doesn't untap and the stun counter isn't removed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fear of Sleep Paralysis");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bear.0 as usize].tapped = true;
    t.g.add_counters(Entity::Object(bear), "stun", 1, None);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bear).tapped);
    assert_eq!(t.counters(bear, "stun"), 1);
}

#[test]
fn your_own_permanents_lose_stun_counters_normally() {
    cr!("122.1d");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fear of Sleep Paralysis");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[bear.0 as usize].tapped = true;
    t.g.add_counters(Entity::Object(bear), "stun", 1, None);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bear).tapped);
    assert_eq!(t.counters(bear, "stun"), 0);
}

#[test]
fn effects_cant_remove_them_either() {
    cr!("122.1d");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fear of Sleep Paralysis");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bear), "stun", 2, None);
    assert_eq!(t.g.remove_counters(Entity::Object(bear), "stun", 2), 0);
    assert_eq!(t.counters(bear, "stun"), 2);
}
