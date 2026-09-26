//! CR 702.100 Evolve.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_018_026::triggers_on_stack;
use crate::common_k702_052_066::{destroy, enter_def, run_effect};
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts real cards onto the battlefield under `p`'s control at the same time.
fn enter_together(t: &mut TestGame, p: PlayerId, names: &[&str]) -> Vec<ObjectId> {
    let moves = names
        .iter()
        .map(|n| MoveEv {
            obj: t.g.create_card_object(card(n), p, Zone::Nowhere),
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(p),
            etb: EtbInfo {
                controller: Some(p),
                ..Default::default()
            },
            source: None,
        })
        .collect();
    t.g.move_objects(moves).into_iter().flatten().collect()
}

#[test]
fn evolve_puts_a_counter_when_a_greater_creature_enters() {
    cr!("702.100", "702.100a");
    ruling!(
        "Cloudfin Raptor",
        "Whenever a creature enters the battlefield under your control, check its power and toughness against the power and toughness of the creature with evolve."
    );
    assert_supported("Cloudfin Raptor");
    let mut t = TestGame::new(2);
    // Cloudfin Raptor: 0/1 flying, evolve.
    let raptor = t.battlefield(P0, "Cloudfin Raptor");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    t.resolve_all();
    assert_eq!(t.counters(raptor, counters::PLUS1), 1);
    assert_eq!(t.pt(raptor), (1, 2));
    // A creature an opponent controls doesn't.
    t.enter(P1, "Hill Giant");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 0);
}

#[test]
fn evolve_doesnt_trigger_unless_a_stat_is_greater() {
    cr!("702.100a");
    ruling!(
        "Cloudfin Raptor",
        "If neither stat of the new creature is greater, evolve won’t trigger at all."
    );
    ruling!(
        "Cloudfin Raptor",
        "When comparing the stats of the two creatures for evolve, you always compare power to power and toughness to toughness."
    );
    let mut t = TestGame::new(2);
    // Shambleshark: 2/1. Memnite (1/1) is neither stronger nor tougher.
    let shark = t.battlefield(P0, "Shambleshark");
    t.enter(P0, "Memnite");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 0);
    // Wall of Omens (0/4) has a greater toughness only: it triggers.
    t.enter(P0, "Wall of Omens");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    t.resolve_all();
    assert_eq!(t.pt(shark), (3, 2));
    // Adaptive Snapjaw (6/2): a 3/3 has a greater toughness, though a lesser power.
    let snapjaw = t.battlefield(P0, "Adaptive Snapjaw");
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.pt(snapjaw), (7, 3));
}

#[test]
fn counters_the_creature_enters_with_count() {
    cr!("702.100a");
    ruling!(
        "Cloudfin Raptor",
        "If a creature enters the battlefield with +1/+1 counters on it, consider those counters when determining if evolve will trigger."
    );
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Cloudfin Raptor");
    // Arcbound Worker: a 0/0 that enters with a +1/+1 counter.
    t.enter(P0, "Arcbound Worker");
    t.resolve_all();
    assert_eq!(t.counters(raptor, counters::PLUS1), 1);
}

#[test]
fn the_comparison_is_made_again_as_evolve_resolves() {
    cr!("702.100a");
    ruling!(
        "Cloudfin Raptor",
        "If evolve triggers, the stat comparison will happen again when the ability tries to resolve. If neither stat of the new creature is greater, the ability will do nothing."
    );
    let mut t = TestGame::new(2);
    // Experiment One: 1/1, evolve.
    let one = t.battlefield(P0, "Experiment One");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    // In response, the Bears get -1/-1: 1/1 isn't greater.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(-1), Value::c(-1))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    t.resolve_all();
    assert_eq!(t.counters(one, counters::PLUS1), 0);
}

#[test]
fn the_stat_that_is_greater_may_change_before_evolve_resolves() {
    cr!("702.100a");
    ruling!(
        "Cloudfin Raptor",
        "When comparing the stats as the evolve ability resolves, it’s possible that the stat that’s greater changes from power to toughness or vice versa."
    );
    let mut t = TestGame::new(2);
    // A 2/2 with evolve; a 1/3 enters (its toughness is greater), then gets +2/-2.
    let evolver = enter_def(
        &mut t,
        P0,
        custom_card("Evolving Ooze", "Creature — Ooze", Some((2, 2)), "Evolve"),
    );
    t.resolve_all();
    let wall = enter_def(
        &mut t,
        P0,
        custom_card("Squat Wall", "Creature — Wall", Some((1, 3)), ""),
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(-2))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(wall)],
    );
    t.resolve_all();
    assert_eq!(t.counters(evolver, counters::PLUS1), 1);
}

#[test]
fn creatures_entering_together_are_compared_one_at_a_time() {
    cr!("702.100a");
    ruling!(
        "Cloudfin Raptor",
        "If multiple creatures enter the battlefield at the same time, evolve may trigger multiple times, although the stat comparison will take place each time one of those abilities tries to resolve."
    );
    let mut t = TestGame::new(2);
    let one = t.battlefield(P0, "Experiment One");
    enter_together(&mut t, P0, &["Grizzly Bears", "Grizzly Bears"]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 2);
    t.resolve_all();
    // The first makes it 2/2: the second Bears is no longer greater.
    assert_eq!(t.counters(one, counters::PLUS1), 1);
}

#[test]
fn a_creature_that_left_is_compared_as_it_last_existed() {
    cr!("702.100a");
    ruling!(
        "Cloudfin Raptor",
        "If the creature that entered the battlefield leaves the battlefield before evolve tries to resolve, use its last known power and toughness to compare the stats."
    );
    let mut t = TestGame::new(2);
    let one = t.battlefield(P0, "Experiment One");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    destroy(&mut t, bears);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.counters(one, counters::PLUS1), 1);
}

#[test]
fn a_creature_evolves_when_its_evolve_ability_puts_counters_on_it() {
    cr!("702.100b");
    assert_supported("Renegade Krasis");
    let mut t = TestGame::new(2);
    // Renegade Krasis: 3/2, evolve, "Whenever this creature evolves, put a +1/+1 counter on
    // each other creature you control with a +1/+1 counter on it."
    let krasis = t.battlefield(P0, "Renegade Krasis");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, None);
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.counters(krasis, counters::PLUS1), 1);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(elves, counters::PLUS1), 0);
    // +1/+1 counters put on it otherwise don't make it evolve.
    t.g.add_counters(Entity::Object(krasis), counters::PLUS1, 1, None);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn a_creature_evolves_once_however_many_counters_are_put_on_it() {
    cr!("702.100b");
    let mut t = TestGame::new(2);
    // Hardened Scales: one more +1/+1 counter.
    t.battlefield(P0, "Hardened Scales");
    let krasis = t.battlefield(P0, "Renegade Krasis");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, None);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.counters(krasis, counters::PLUS1), 2);
    // Krasis evolved once: the Bears got one (+1 from Hardened Scales) more.
    assert_eq!(t.counters(bears, counters::PLUS1), 4);
}

#[test]
fn a_noncreature_permanent_is_never_greater() {
    cr!("702.100c");
    let mut t = TestGame::new(2);
    let one = t.battlefield(P0, "Experiment One");
    let bears = t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    // The Bears stop being a creature before evolve resolves.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveTypes(vec![types::CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(bears)],
    );
    t.resolve_all();
    assert_eq!(t.counters(one, counters::PLUS1), 0);
}

#[test]
fn each_instance_of_evolve_triggers_separately() {
    cr!("702.100d");
    let def = || {
        custom_card(
            "Doubly Evolving Ooze",
            "Creature — Ooze",
            Some((1, 1)),
            "Evolve\nEvolve",
        )
    };
    let mut t = TestGame::new(2);
    let ooze = t.custom(P0, def(), Zone::Battlefield);
    t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 2);
    t.resolve_all();
    assert_eq!(t.pt(ooze), (3, 3));
    // Each checks again as it resolves: after one counter, a 2/2 isn't greater.
    let mut t = TestGame::new(2);
    let ooze = t.custom(P0, def(), Zone::Battlefield);
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 2);
    t.resolve_all();
    assert_eq!(t.pt(ooze), (2, 2));
}
