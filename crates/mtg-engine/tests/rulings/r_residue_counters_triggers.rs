//! Item `rulings-residue-counters` — triggered abilities that check or count counters:
//! intervening "if" clauses checked as the ability triggers and again as it resolves
//! (CR 603.4: Pelt Collector, Prairie Dog, Sarulf, ED-E), last known information
//! (CR 603.10a, 608.2h), choices made on resolution (CR 608.2c), triggers on counters
//! being put, including the counters a permanent enters with (CR 122.6, 603.6a), and
//! "this turn" counts that look back before the source existed (Botanical Brawler,
//! Iridescent Hornbeetle).

use crate::r_residue_counters_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;


// --- Pelt Collector ----------------------------------------------------------------------

#[test]
fn pelt_collector_rechecks_its_power_as_each_trigger_resolves() {
    cr!("603.4");
    ruling!(
        "Pelt Collector",
        "if Pelt Collector is still a 1/1 creature and two 2/2 creatures die, it only gets one +1/+1 counter"
    );
    supported("Pelt Collector");
    let mut t = TestGame::new(2);
    let pelt = t.battlefield(P0, "Pelt Collector");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, a);
    destroy(&mut t, b);
    assert_eq!(triggers_on_stack(&t, "greater than"), 2);
    t.resolve_all();
    assert_eq!(plus1(&t, pelt), 1, "a 2/2 by the second resolution");
}

#[test]
fn pelt_collector_doesnt_trigger_for_a_creature_no_bigger_than_it() {
    cr!("603.4");
    ruling!(
        "Pelt Collector",
        "If a creature with power less than or equal to Pelt Collector’s power enters the battlefield or dies, Pelt Collector’s first ability doesn’t trigger at all."
    );
    let mut t = TestGame::new(2);
    let pelt = t.battlefield(P0, "Pelt Collector");
    let elves = t.enter(P0, "Llanowar Elves");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "greater than"), 0);
    // Pumping the Elves afterwards gets nothing either.
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(elves)]);
    assert_eq!(plus1(&t, pelt), 0);
}

#[test]
fn pelt_collector_uses_last_known_power_of_an_entering_creature_that_left() {
    cr!("603.4", "608.2h");
    ruling!(
        "Pelt Collector",
        "If the entering creature leaves the battlefield while Pelt Collector’s triggered ability is on the stack, use its power as it last existed on the battlefield"
    );
    let mut t = TestGame::new(2);
    let pelt = t.battlefield(P0, "Pelt Collector");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "greater than"), 1);
    exile_now(&mut t, bears);
    t.resolve_all();
    assert_eq!(plus1(&t, pelt), 1, "a 2/2 as it last existed");
}

#[test]
fn pelt_collector_dies_with_a_larger_creature_dealt_lethal_damage_at_once() {
    cr!("704.3", "704.5g");
    ruling!(
        "Pelt Collector",
        "If Pelt Collector is dealt lethal damage at the same time as a larger creature you control, they’re destroyed at the same time."
    );
    supported("Pyroclasm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pelt Collector");
    t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Pyroclasm", &[]);
    assert!(t.in_graveyard(P0, "Pelt Collector"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn pelt_collector_with_negative_power_triggers_for_a_zero_power_creature() {
    cr!("603.4", "107.1b");
    ruling!(
        "Pelt Collector",
        "If Pelt Collector’s power becomes less than 0, a creature with 0 power"
    );
    supported("Shrink");
    let mut t = TestGame::new(2);
    let pelt = t.battlefield(P0, "Pelt Collector");
    cast_resolve(&mut t, P1, "Shrink", &[obj(pelt)]);
    assert_eq!(t.pt(pelt), (-4, 1));
    t.enter(P0, "Ornithopter");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "greater than"), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, pelt), 1);
}

// --- Prairie Dog -------------------------------------------------------------------------

#[test]
fn prairie_dog_checks_for_spells_cast_from_hand_on_triggering_and_resolving() {
    cr!("603.4");
    ruling!(
        "Prairie Dog",
        "If you’ve cast a spell from your hand since the ability triggered, the ability won’t do anything."
    );
    supported("Prairie Dog");
    supported("Opt");
    // A spell cast earlier in the turn: no trigger.
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Prairie Dog");
    t.set_step(P0, Step::PrecombatMain);
    cast_resolve(&mut t, P0, "Opt", &[]);
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "cast a spell"), 0);
    t.resolve_all();
    assert_eq!(plus1(&t, dog), 0);
    // A spell cast in response: it does nothing.
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Prairie Dog");
    to_end_step(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "cast a spell"), 1);
    cast_resolve(&mut t, P0, "Opt", &[]);
    assert_eq!(plus1(&t, dog), 0);
    // Neither: a counter.
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P0, "Prairie Dog");
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(plus1(&t, dog), 1);
}

// --- Sarulf, Realm Eater -----------------------------------------------------------------

#[test]
fn sarulf_upkeep_ability_needs_counters_on_triggering_and_resolving() {
    cr!("603.4");
    ruling!(
        "Sarulf, Realm Eater",
        "If Sarulf doesn't have any +1/+1 counters on it as your upkeep begins, its last ability won't trigger at all."
    );
    supported("Sarulf, Realm Eater");
    // No counters: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sarulf, Realm Eater");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "remove all"), 0);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Counters removed in response: nothing is exiled.
    let mut t = TestGame::new(2);
    let sarulf = with_counters(&mut t, P0, "Sarulf, Realm Eater", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "remove all"), 1);
    remove_plus1(&mut t, sarulf, 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn sarulf_that_left_the_battlefield_exiles_nothing() {
    cr!("603.4", "400.7");
    ruling!(
        "Sarulf, Realm Eater",
        "If Sarulf isn't on the battlefield as its last ability starts to resolve, you can't remove any +1/+1 counters from it"
    );
    let mut t = TestGame::new(2);
    let sarulf = with_counters(&mut t, P0, "Sarulf, Realm Eater", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_upkeep(&mut t);
    assert_eq!(triggers_on_stack(&t, "remove all"), 1);
    exile_now(&mut t, sarulf);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}


#[test]
fn sarulf_may_remove_counters_it_needs_to_survive() {
    cr!("704.5g", "704.3");
    ruling!(
        "Sarulf, Realm Eater",
        "You may choose to remove Sarulf's +1/+1 counters even if that would cause lethal damage to be marked on Sarulf."
    );
    let mut t = TestGame::new(2);
    let sarulf = with_counters(&mut t, P0, "Sarulf, Realm Eater", 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_upkeep(&mut t);
    damage(&mut t, sarulf, 4);
    assert!(t.on_battlefield(sarulf), "a 5/5 with 4 damage");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "the ability finishes first");
    assert!(t.in_graveyard(P0, "Sarulf, Realm Eater"));
}


#[test]
fn sarulf_counts_an_opponents_permanent_going_to_any_graveyard() {
    cr!("603.10a", "400.3");
    ruling!(
        "Sarulf, Realm Eater",
        "It doesn't matter which graveyard the permanent is put into as long as an opponent controlled it"
    );
    supported("Act of Treason");
    // P0's own Grizzly Bears, controlled by P1, dies into P0's graveyard.
    let mut t = TestGame::new(2);
    let sarulf = t.battlefield(P0, "Sarulf, Realm Eater");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Act of Treason", &[obj(bears)]);
    assert_eq!(controller(&mut t, bears), P1);
    destroy(&mut t, bears);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(plus1(&t, sarulf), 1);
}

// --- ED-E, Lonesome Eyebot ---------------------------------------------------------------

#[test]
fn ed_e_checks_the_attackers_against_its_quest_counters_twice() {
    cr!("603.4");
    ruling!(
        "ED-E, Lonesome Eyebot",
        "If the number of attacking creatures isn't greater than the number of quest counters on ED-E when you attack, its second ability won't trigger at all."
    );
    supported("ED-E, Lonesome Eyebot");
    // One quest counter, one attacker: no trigger.
    let mut t = TestGame::new(2);
    let ede = t.battlefield(P0, "ED-E, Lonesome Eyebot");
    put(&mut t, ede, "quest", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "quest"), 0);
    // Two attackers: it triggers; a counter put in response makes it do nothing.
    let mut t = TestGame::new(2);
    let ede = t.battlefield(P0, "ED-E, Lonesome Eyebot");
    put(&mut t, ede, "quest", 1);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "quest"), 1);
    put(&mut t, ede, "quest", 1);
    t.resolve_all();
    assert_eq!(t.counters(ede, "quest"), 2);
}

// --- Botanical Brawler -------------------------------------------------------------------

#[test]
fn botanical_brawler_counts_counters_put_before_it_was_on_the_battlefield() {
    cr!("603.4", "603.2");
    ruling!(
        "Botanical Brawler",
        "even if Botanical Brawler wasn't on the battlefield at that time"
    );
    supported("Botanical Brawler");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 1);
    let brawler = with_counters(&mut t, P0, "Botanical Brawler", 2);
    let before = plus1(&t, brawler);
    give_plus1(&mut t, bears, 1);
    assert_eq!(triggers_on_stack(&t, "first time"), 0);
    t.resolve_all();
    assert_eq!(plus1(&t, brawler), before);
    // A creature that had none this turn: it triggers.
    let giant = t.battlefield(P0, "Hill Giant");
    give_plus1(&mut t, giant, 1);
    assert_eq!(triggers_on_stack(&t, "first time"), 1);
}

#[test]
fn botanical_brawler_triggers_for_a_permanent_entering_with_counters() {
    cr!("122.6", "603.6a");
    ruling!(
        "Botanical Brawler",
        "Whenever another permanent enters the battlefield under your control with one or more +1/+1 counters on it, Botanical Brawler's last ability will trigger."
    );
    let mut t = TestGame::new(2);
    let brawler = with_counters(&mut t, P0, "Botanical Brawler", 2);
    let before = plus1(&t, brawler);
    enter_with(&mut t, P0, "Grizzly Bears", 2, false);
    assert_eq!(triggers_on_stack(&t, "first time"), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, brawler), before + 1);
}

// --- Iridescent Hornbeetle ---------------------------------------------------------------

#[test]
fn iridescent_hornbeetle_counts_counters_put_before_it_and_on_creatures_since_gone() {
    cr!("122.6", "608.2h");
    ruling!(
        "Iridescent Hornbeetle",
        "even if that happened before Iridescent Hornbeetle was on the battlefield, and even if those creatures are no longer on the battlefield or under your control"
    );
    supported("Iridescent Hornbeetle");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_plus1(&mut t, bears, 2);
    destroy(&mut t, bears);
    t.battlefield(P0, "Iridescent Hornbeetle");
    end_step(&mut t, P0);
    assert_eq!(tokens_named(&t, P0, "Insect"), 2);
}

#[test]
fn iridescent_hornbeetle_counts_the_counters_a_creature_enters_with() {
    cr!("122.6");
    ruling!(
        "Iridescent Hornbeetle",
        "If a creature enters the battlefield under your control with +1/+1 counters on it, Iridescent Hornbeetle’s ability counts those counters."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Iridescent Hornbeetle");
    enter_with(&mut t, P0, "Grizzly Bears", 3, false);
    end_step(&mut t, P0);
    assert_eq!(tokens_named(&t, P0, "Insect"), 3);
}


// --- Cursed Wombat, Omarthis -------------------------------------------------------------

#[test]
fn cursed_wombat_granted_ability_triggers_for_counters_a_permanent_enters_with() {
    cr!("122.6", "603.6a");
    ruling!(
        "Cursed Wombat",
        "the ability granted to that permanent by Cursed Wombat will trigger when that permanent enters the battlefield"
    );
    supported("Cursed Wombat");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cursed Wombat");
    let bears = enter_with(&mut t, P0, "Grizzly Bears", 1, false);
    assert_eq!(triggers_on_stack(&t, "additional"), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
}

#[test]
fn omarthis_triggers_for_a_colorless_creature_entering_with_counters() {
    cr!("122.6", "603.6a");
    ruling!(
        "Omarthis, Ghostfire Initiate",
        "a colorless creature you control entering the battlefield with counters"
    );
    supported("Omarthis, Ghostfire Initiate");
    let mut t = TestGame::new(2);
    let om = with_counters(&mut t, P0, "Omarthis, Ghostfire Initiate", 1);
    // A colorless creature entering with counters, then counters put by an effect.
    let walker = enter_with(&mut t, P0, "Ornithopter", 1, false);
    assert_eq!(triggers_on_stack(&t, "colorless"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(plus1(&t, om), 2);
    give_plus1(&mut t, walker, 2);
    assert_eq!(triggers_on_stack(&t, "colorless"), 1, "one or more: once");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(plus1(&t, om), 3);
    // Not for a colored creature.
    enter_with(&mut t, P0, "Grizzly Bears", 1, false);
    assert_eq!(triggers_on_stack(&t, "colorless"), 0);
}

// --- Elrond, Exemplar of Light, Berta ----------------------------------------------------

#[test]
fn elrond_triggers_once_for_each_creature_a_spell_targets() {
    cr!("603.2", "603.2c");
    ruling!(
        "Elrond, Master of Healing",
        "Elrond, Master of Healing's last ability will trigger twice"
    );
    supported("Elrond, Master of Healing");
    supported("Chandra's Pyrohelix");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elrond, Master of Healing");
    let a = with_counters(&mut t, P0, "Hill Giant", 1);
    let b = with_counters(&mut t, P0, "Hill Giant", 1);
    t.set_step(P1, Step::PrecombatMain);
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    cast_slots(&mut t, P1, "Chandra's Pyrohelix", &[&[obj(a), obj(b)]]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 2);
}

#[test]
fn exemplar_of_light_triggers_once_for_life_gained_for_each() {
    cr!("119.9", "603.2");
    ruling!(
        "Exemplar of Light",
        "that life is gained as one event and Exemplar of Light's second ability triggers only once"
    );
    supported("Exemplar of Light");
    supported("Feed the Clan");
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Exemplar of Light");
    // Feed the Clan: 5 life, as one event.
    cast_new(&mut t, P0, "Feed the Clan", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "gain life"), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, ex), 1);
}

#[test]
fn exemplar_of_light_ignores_a_teammates_life_gain() {
    cr!("810.9", "119.9");
    ruling!(
        "Exemplar of Light",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Exemplar of Light's second ability to trigger"
    );
    let mut t = two_headed_giant();
    let ex = t.battlefield(P0, "Exemplar of Light");
    gain_life(&mut t, P1, 3);
    assert_eq!(triggers_on_stack(&t, "gain life"), 0);
    gain_life(&mut t, P0, 3);
    assert_eq!(triggers_on_stack(&t, "gain life"), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, ex), 1);
}


// --- Protean Hydra -----------------------------------------------------------------------

#[test]
fn protean_hydra_triggers_once_per_counter_removed_at_once() {
    cr!("603.2c", "603.7a");
    ruling!(
        "Protean Hydra",
        "If multiple +1/+1 counters are removed from Protean Hydra at once, its last ability will trigger that many times."
    );
    let mut t = TestGame::new(2);
    let hydra = with_counters(&mut t, P0, "Protean Hydra", 4);
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(hydra)]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "removed"), 3);
}

#[test]
fn protean_hydra_counters_removed_in_the_end_step_return_at_the_next_one() {
    cr!("603.7a", "603.7b");
    ruling!(
        "Protean Hydra",
        "If any +1/+1 counters are removed from Protean Hydra during the end step"
    );
    let mut t = TestGame::new(2);
    let hydra = with_counters(&mut t, P0, "Protean Hydra", 3);
    to_end_step(&mut t, P0);
    t.resolve_all();
    remove_plus1(&mut t, hydra, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, hydra), 2);
    // Not this end step: the next one, at the end of P1's turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(plus1(&t, hydra), 2);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(plus1(&t, hydra), 4);
}
