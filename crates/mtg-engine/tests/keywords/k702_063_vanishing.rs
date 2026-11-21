//! CR 702.63 Vanishing.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_027_037::next_upkeep;
use crate::common_k702_052_066::*;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const VANISHING: &str = "Vanishing";
const TIME: &str = "time";

#[test]
fn a_permanent_with_vanishing_enters_with_time_counters() {
    cr!("702.63", "702.63a");
    assert_supported("Keldon Marauders");
    assert_supported("Calciderm");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let marauders = t.hand(P0, "Keldon Marauders");
    t.cast(P0, marauders).go();
    t.resolve();
    assert!(t.on_battlefield(marauders));
    assert_eq!(t.counters(marauders, TIME), 2);
    // Put onto the battlefield by an effect, it still enters with them.
    let calciderm = t.enter(P0, "Calciderm");
    assert_eq!(t.counters(calciderm, TIME), 4);
}

#[test]
fn a_time_counter_is_removed_each_upkeep_and_the_last_one_sacrifices_it() {
    cr!("702.63a");
    assert_supported("Waning Wurm");
    let mut t = TestGame::new(2);
    let wurm = t.enter(P0, "Waning Wurm");
    assert_eq!(t.counters(wurm, TIME), 2);
    next_upkeep(&mut t, P0);
    assert_eq!(stack_triggers(&t, VANISHING).len(), 1);
    t.resolve();
    assert_eq!(t.counters(wurm, TIME), 1);
    assert!(t.on_battlefield(wurm));
    // The opponent's upkeep doesn't remove one.
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert!(stack_triggers(&t, VANISHING).is_empty());
    next_upkeep(&mut t, P0);
    t.resolve();
    // The last counter was removed: "sacrifice it" triggers.
    assert_eq!(t.counters(wurm, TIME), 0);
    assert!(t.on_battlefield(wurm));
    assert_eq!(stack_triggers(&t, VANISHING).len(), 1);
    t.resolve();
    assert!(!t.on_battlefield(wurm));
    assert!(t.in_graveyard(P0, "Waning Wurm"));
}

#[test]
fn removing_the_last_time_counter_any_other_way_also_sacrifices_it() {
    cr!("702.63a");
    let mut t = TestGame::new(2);
    let calciderm = t.enter(P0, "Calciderm");
    remove_counters(&mut t, calciderm, TIME, 3);
    t.settle();
    assert!(stack_triggers(&t, VANISHING).is_empty());
    remove_counters(&mut t, calciderm, TIME, 1);
    t.settle();
    assert_eq!(stack_triggers(&t, VANISHING).len(), 1);
    t.resolve();
    assert!(!t.on_battlefield(calciderm));
}

#[test]
fn with_no_time_counter_the_upkeep_ability_does_not_trigger() {
    cr!("702.63a");
    let mut t = TestGame::new(2);
    // Put directly onto the battlefield without entering: no time counters.
    let wurm = t.battlefield(P0, "Waning Wurm");
    assert_eq!(t.counters(wurm, TIME), 0);
    next_upkeep(&mut t, P0);
    assert!(stack_triggers(&t, VANISHING).is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
}

#[test]
fn a_counter_added_later_is_removed_by_vanishing_too() {
    cr!("702.63a");
    assert_supported("Soultether Golem");
    let mut t = TestGame::new(2);
    let golem = t.enter(P0, "Soultether Golem");
    assert_eq!(t.counters(golem, TIME), 1);
    // "Whenever another creature you control enters, put a time counter on this creature."
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.counters(golem, TIME), 2);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(golem));
    assert_eq!(t.counters(golem, TIME), 1);
}

#[test]
fn vanishing_without_a_number_adds_no_counters() {
    cr!("702.63b");
    ruling!(
        "Tidewalker",
        "when the last time counter is removed, it will be put into the graveyard as a state-based action"
    );
    assert_supported("Tidewalker");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    // Its own ability puts a time counter on it for each Island; vanishing adds none.
    let walker = t.enter(P0, "Tidewalker");
    assert_eq!(t.counters(walker, TIME), 2);
    assert_eq!(t.pt(walker), (2, 2));
    next_upkeep(&mut t, P0);
    t.resolve();
    assert_eq!(t.pt(walker), (1, 1));
    next_upkeep(&mut t, P0);
    t.resolve();
    // At 0/0 it's put into the graveyard before the sacrifice trigger resolves.
    assert!(!t.on_battlefield(walker));
    assert!(t.in_graveyard(P0, "Tidewalker"));
}

#[test]
fn vanishing_without_a_number_and_no_counters_does_nothing() {
    cr!("702.63b");
    ruling!(
        "Tidewalker",
        "its vanishing ability has no effect"
    );
    let mut t = TestGame::new(2);
    // Something boosting its toughness keeps it alive with no counters.
    assert_supported("Glorious Anthem");
    t.battlefield(P0, "Glorious Anthem");
    let tide = t.enter(P0, "Tidewalker");
    assert_eq!(t.pt(tide), (1, 1));
    assert_eq!(t.counters(tide, TIME), 0);
    next_upkeep(&mut t, P0);
    assert!(stack_triggers(&t, VANISHING).is_empty());
    next_upkeep(&mut t, P0);
    assert!(t.on_battlefield(tide));
}

#[test]
fn each_instance_of_vanishing_works_separately() {
    cr!("702.63c");
    let mut t = TestGame::new(2);
    let def = custom_card(
        "Twice-Vanishing Wisp",
        "Creature — Spirit",
        Some((1, 1)),
        "Vanishing 2\nVanishing 1",
    );
    let wisp = enter_def(&mut t, P0, def);
    // Each instance adds its own counters.
    assert_eq!(t.counters(wisp, TIME), 3);
    // Each instance removes one.
    next_upkeep(&mut t, P0);
    assert_eq!(stack_triggers(&t, VANISHING).len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(wisp, TIME), 1);
    assert!(t.on_battlefield(wisp));
    // The first removes the last counter; the second then doesn't resolve (no counter
    // left), and each instance's "sacrifice it" ability triggers.
    next_upkeep(&mut t, P0);
    t.resolve();
    assert_eq!(t.counters(wisp, TIME), 0);
    assert_eq!(stack_triggers(&t, VANISHING).len(), 3);
    t.resolve_all();
    assert!(!t.on_battlefield(wisp));
}

#[test]
fn a_permanent_that_dies_with_no_time_counters_had_its_last_one_removed() {
    cr!("702.63a");
    ruling!(
        "Deadly Grub",
        "it could have been put into the graveyard some other way (say, while the sacrifice ability of vanishing is on the stack)"
    );
    assert_supported("Deadly Grub");
    let mut t = TestGame::new(2);
    let grub = t.enter(P0, "Deadly Grub");
    assert_eq!(t.counters(grub, TIME), 3);
    remove_counters(&mut t, grub, TIME, 3);
    t.settle();
    assert_eq!(stack_triggers(&t, VANISHING).len(), 1);
    // Destroyed in response to the sacrifice trigger: it had no time counters.
    destroy(&mut t, grub);
    t.resolve_all();
    let insects: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Insect"))
        .map(|o| (o.power(), o.toughness()))
        .collect();
    assert_eq!(insects, vec![(6, 1)]);
    // One that dies with a time counter on it doesn't.
    let grub = t.enter(P0, "Deadly Grub");
    destroy(&mut t, grub);
    t.resolve_all();
    assert!(!t.on_battlefield(grub));
    assert_eq!(t.g.permanents().filter(|o| o.is_token()).count(), 1);
}
