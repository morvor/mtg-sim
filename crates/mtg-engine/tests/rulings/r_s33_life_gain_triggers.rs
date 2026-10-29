//! Rulings batch S33 — "whenever you gain life" abilities: each life-gain event is one
//! event, however much life is gained, and life gained "for each" of something is gained
//! at once (CR 119.9, 603.2c); "for the first time each turn" counts the life gained
//! earlier in the turn, even before the ability's source was on the battlefield
//! (CR 603.2).

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s06_common::attach_new;
use crate::r_s19_common::{gain_level, level};
use crate::r_s25_common::cast_new;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s33_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls three Grizzly Bears (and whatever else is on the battlefield) and casts
/// Congregate ("Target player gains 2 life for each creature on the battlefield.")
/// targeting themself. Returns the number of triggered abilities whose text contains
/// `trigger` that were put on the stack for it, before they resolve.
fn congregate(t: &mut TestGame, trigger: &str) -> usize {
    for _ in 0..3 {
        t.battlefield(P0, "Grizzly Bears");
    }
    cast_new(t, P0, "Congregate", &[Entity::Player(P0)]);
    t.resolve();
    t.settle();
    triggers_on_stack(t, trigger)
}

/// The +1/+1 counters on each creature `p` controls.
fn counters_on_creatures(t: &TestGame, p: PlayerId) -> Vec<u32> {
    crate::r_s01_common::creatures(t, p)
        .into_iter()
        .map(|c| t.counters(c, "+1/+1"))
        .collect()
}

#[test]
fn bloodbond_vampire_triggers_once_per_life_gain_event_however_much() {
    cr!("119.9", "603.2c");
    ruling!(
        "Bloodbond Vampire",
        "The ability triggers just once for each life-gaining event, whether it’s 1 life from Drana’s Emissary or 7 life from Nissa’s Renewal."
    );
    supported("Bloodbond Vampire");
    // "Whenever you gain life, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let vampire = t.battlefield(P0, "Bloodbond Vampire");
    // Sacred Nectar: "You gain 4 life."
    cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(vampire, "+1/+1"), 1);
    // Drana's Emissary: "At the beginning of your upkeep, each opponent loses 1 life and
    // you gain 1 life."
    t.battlefield(P0, "Drana's Emissary");
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.counters(vampire, "+1/+1"), 2);
}

#[test]
fn scion_of_the_swarm_life_gained_for_each_is_one_event() {
    cr!("119.9", "603.2c");
    ruling!(
        "Scion of the Swarm",
        "If you gain an amount of life \"for each\" of something, that life is gained as one event and the ability will trigger only once."
    );
    ruling!(
        "Scion of the Swarm",
        "An ability that triggers \"whenever you gain life\" triggers just once for each life-gaining event, no matter how much life you gain."
    );
    supported("Scion of the Swarm");
    // "Whenever you gain life, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let scion = t.battlefield(P0, "Scion of the Swarm");
    let from = t.g.turn_events.len();
    assert_eq!(congregate(&mut t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert_eq!(life_gains_since(&t, from, P0), vec![8]);
    assert_eq!(t.counters(scion, "+1/+1"), 1);
}

#[test]
fn trudge_garden_life_gained_for_each_triggers_once() {
    cr!("119.9", "603.2c");
    ruling!(
        "Trudge Garden",
        "If you gain an amount of life \"for each\" of something, that life is gained as one event and the ability will trigger only once."
    );
    supported("Trudge Garden");
    // "Whenever you gain life, you may pay {2}. If you do, create a 4/4 green Fungus
    // Beast creature token with trample."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trudge Garden");
    assert_eq!(congregate(&mut t, "Fungus Beast"), 1);
}

#[test]
fn veinwitch_coven_triggers_once_per_life_gain_event() {
    cr!("119.9", "603.2c");
    ruling!(
        "Veinwitch Coven",
        "An ability that triggers \"whenever you gain life\" triggers just once for each life-gaining event, no matter how much life you gain."
    );
    supported("Veinwitch Coven");
    // "Whenever you gain life, you may pay {B}. If you do, return target creature card
    // from your graveyard to your hand."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Veinwitch Coven");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Craw Wurm");
    assert_eq!(congregate(&mut t, "return target creature card"), 1);
}

#[test]
fn blood_researcher_life_gained_for_each_triggers_once() {
    cr!("119.9", "603.2c");
    ruling!(
        "Blood Researcher",
        "If you gain an amount of life “for each” of something, that life is gained as one event and the ability will trigger only once."
    );
    ruling!(
        "Blood Researcher",
        "An ability that triggers “whenever you gain life” triggers just once for each life gain event, no matter how much life you gain."
    );
    supported("Blood Researcher");
    let mut t = TestGame::new(2);
    let researcher = t.battlefield(P0, "Blood Researcher");
    assert_eq!(congregate(&mut t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(researcher, "+1/+1"), 1);
}

#[test]
fn cleric_of_lifes_bond_life_gained_for_each_triggers_once() {
    cr!("119.9", "603.2c");
    ruling!(
        "Cleric of Life's Bond",
        "An ability that triggers “whenever you gain life” triggers just once for each life-gaining event, no matter how much life you gain."
    );
    supported("Cleric of Life's Bond");
    // "Whenever you gain life for the first time each turn, put a +1/+1 counter on this
    // creature."
    let mut t = TestGame::new(2);
    let cleric = t.battlefield(P0, "Cleric of Life's Bond");
    assert_eq!(congregate(&mut t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(cleric, "+1/+1"), 1);
}

#[test]
fn archangel_of_thune_life_gained_for_each_triggers_once() {
    cr!("119.9", "603.2c");
    ruling!(
        "Archangel of Thune",
        "If you gain an amount of life “for each” of something, that life is gained as one event and the ability triggers only once."
    );
    supported("Archangel of Thune");
    // "Whenever you gain life, put a +1/+1 counter on each creature you control."
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Archangel of Thune");
    assert_eq!(congregate(&mut t, "+1/+1 counter on each"), 1);
    t.resolve_all();
    assert_eq!(t.counters(angel, "+1/+1"), 1);
    assert_eq!(counters_on_creatures(&t, P0), vec![1, 1, 1, 1]);
}

#[test]
fn light_of_promise_life_gained_for_each_triggers_once_with_that_many_counters() {
    cr!("119.9", "603.2c");
    ruling!(
        "Light of Promise",
        "If you gain an amount of life “for each” of something, that life is gained as one event and the ability triggers only once."
    );
    supported("Light of Promise");
    // "Enchanted creature has 'Whenever you gain life, put that many +1/+1 counters on
    // this creature.'" Four creatures: 8 life, one trigger, eight counters.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Light of Promise", giant);
    assert_eq!(congregate(&mut t, "that many +1/+1 counters"), 1);
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 8);
}

#[test]
fn cleric_class_the_level_2_ability_triggers_once_for_life_gained_for_each() {
    cr!("119.9", "119.10", "603.2c", "716.2a");
    ruling!(
        "Cleric Class",
        "If you gain an amount of life “for each” of something, that life is gained as one event and the ability will trigger only once."
    );
    ruling!(
        "Cleric Class",
        "An ability that triggers “whenever you gain life” triggers just once for each life gain event, no matter how much life you gain."
    );
    supported("Cleric Class");
    // Level 1: "If you would gain life, you gain that much life plus 1 instead." Level 2:
    // "Whenever you gain life, put a +1/+1 counter on target creature you control."
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Cleric Class");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    assert_eq!(level(&t, class), 2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let from = t.g.turn_events.len();
    // Three Bears, the Giant: 2 × 4 = 8, plus 1.
    assert_eq!(congregate(&mut t, "+1/+1 counter on target"), 1);
    t.resolve_all();
    assert_eq!(life_gains_since(&t, from, P0), vec![9]);
    assert_eq!(t.counters(giant, "+1/+1"), 1);
}

#[test]
fn attended_healer_life_gained_for_each_triggers_once() {
    cr!("119.9", "603.2c");
    ruling!(
        "Attended Healer",
        "If you gain an amount of life “for each” of something, that life is gained as one event and the ability will trigger only once."
    );
    supported("Attended Healer");
    // "Whenever you gain life for the first time each turn, create a 1/1 white Cat
    // creature token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Attended Healer");
    assert_eq!(congregate(&mut t, "Cat creature token"), 1);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

#[test]
fn attended_healer_life_gained_before_it_entered_counts_for_the_first_time() {
    cr!("603.2");
    ruling!(
        "Attended Healer",
        "An ability that triggers whenever you gain life “for the first time each turn” won’t trigger if you gain life during a turn before the permanent with that ability is on the battlefield, even if you gain life again later in the turn."
    );
    let mut t = TestGame::new(2);
    cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
    t.battlefield(P0, "Attended Healer");
    cast_new(&mut t, P0, "Sacred Nectar", &[]);
    t.resolve();
    t.settle();
    assert_eq!(t.life(P0), 28);
    assert_eq!(triggers_on_stack(&t, "Cat creature token"), 0);
    t.resolve_all();
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
    // The next turn, it triggers.
    crate::r_s04_common::next_upkeep(&mut t, P0);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    cast_new(&mut t, P0, "Sacred Nectar", &[]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cat creature token"), 1);
}

#[test]
fn cleric_of_lifes_bond_life_gained_before_it_entered_counts_for_the_first_time() {
    cr!("603.2");
    ruling!(
        "Cleric of Life's Bond",
        "An ability that triggers whenever you gain life “for the first time each turn” won’t trigger if you gain life during a turn before the permanent with that ability is on the battlefield, even if you gain life again later in the turn."
    );
    let mut t = TestGame::new(2);
    cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
    let cleric = t.battlefield(P0, "Cleric of Life's Bond");
    cast_and_resolve(&mut t, P0, "Sacred Nectar", &[]);
    assert_eq!(t.counters(cleric, "+1/+1"), 0);
}
