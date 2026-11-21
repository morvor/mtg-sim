//! Trigger limits (`trigger_limits.rs`): "This ability triggers only once." (Acrobatic
//! Cheerleader, the Mox Collectors) and "This ability triggers only twice each turn."
//! (Nadu, Winged Wisdom).

use crate::combat2_common::*;
use crate::td_next_turn_durations::mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn flying_counters(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(t.g.current(id), "flying")
}

/// Taps the Cheerleader and moves to P0's next second main phase, where its survival
/// ability would trigger.
fn tapped_into_second_main(t: &mut TestGame, cheer: ObjectId) {
    if t.g.turn.active == P0 && t.g.turn.step >= Step::PostcombatMain {
        t.advance_to(P1, Step::Upkeep);
    }
    t.advance_to(P0, Step::PrecombatMain);
    let c = t.g.current(cheer);
    t.g.objects[c.0 as usize].tapped = true;
    t.advance_to(P0, Step::PostcombatMain);
    // Triggered abilities go on the stack as P0 would receive priority.
    t.settle();
}

#[test]
fn acrobatic_cheerleader_triggers_only_once() {
    cr!("603.2");
    ruling!(
        "Acrobatic Cheerleader",
        "it won't trigger again as long as it remains on the battlefield"
    );
    supported("Acrobatic Cheerleader");
    let mut t = TestGame::new(2);
    let cheer = t.battlefield(P0, "Acrobatic Cheerleader");
    tapped_into_second_main(&mut t, cheer);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(flying_counters(&t, cheer), 1);
    // The flying counter is removed; tapped again in a later second main phase, it
    // doesn't trigger.
    t.g.objects[cheer.0 as usize].counters.remove("flying");
    tapped_into_second_main(&mut t, cheer);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(flying_counters(&t, cheer), 0);
}

#[test]
fn acrobatic_cheerleader_countered_trigger_still_counts_and_a_new_object_starts_afresh() {
    cr!("603.2", "400.7");
    ruling!("Acrobatic Cheerleader", "even if the triggered ability is countered");
    ruling!(
        "Acrobatic Cheerleader",
        "it's a new object with no memory of its previous existence"
    );
    let mut t = TestGame::new(2);
    let cheer = t.battlefield(P0, "Acrobatic Cheerleader");
    tapped_into_second_main(&mut t, cheer);
    assert_eq!(t.stack_len(), 1);
    let trigger = *t.g.stack.last().unwrap();
    let stifle = t.hand(P1, "Stifle");
    mana(&mut t, P1);
    t.cast_with(P1, stifle, &[Entity::Object(trigger)]).unwrap();
    t.resolve_all();
    assert_eq!(flying_counters(&t, cheer), 0);
    tapped_into_second_main(&mut t, cheer);
    assert_eq!(t.stack_len(), 0, "it already triggered once");
    // It leaves and returns: a new object, whose ability can trigger.
    let shift = t.hand(P0, "Cloudshift");
    mana(&mut t, P0);
    t.cast_with(P0, shift, &[Entity::Object(cheer)]).unwrap();
    t.resolve_all();
    let new = t.g.current(cheer);
    assert_ne!(new, cheer);
    tapped_into_second_main(&mut t, new);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(flying_counters(&t, new), 1);
}

#[test]
fn ruby_collector_conjures_only_the_first_time() {
    cr!("603.2", "508.3c");
    supported("Ruby Collector");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruby Collector");
    let a: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let decl: Vec<(ObjectId, Entity)> = a.iter().map(|c| (*c, Entity::Player(P1))).collect();
    declare(&mut t, P0, &decl);
    t.resolve_all();
    assert!(t.in_hand(P0, "Mox Ruby"));
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    for c in &a {
        t.g.objects[c.0 as usize].tapped = false;
    }
    let hand2 = t.hand_size(P0);
    declare(&mut t, P0, &decl);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand2);
    assert!(hand2 >= hand);
}

/// P0 casts Giant Growth on `target`.
fn grow(t: &mut TestGame, target: ObjectId) {
    let g = t.hand(P0, "Giant Growth");
    mana(t, P0);
    t.cast_with(P0, g, &[Entity::Object(target)]).unwrap();
}

/// The triggered abilities from Nadu's grant waiting on the stack.
fn nadu_triggers(t: &TestGame) -> usize {
    t.g.stack
        .iter()
        .filter(|s| {
            t.g.obj(**s).stack.as_ref().is_some_and(|i| {
                matches!(&i.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                    if ability.text.contains("twice"))
            })
        })
        .count()
}

#[test]
fn nadu_ability_triggers_only_twice_each_turn_for_each_creature() {
    cr!("603.2");
    ruling!(
        "Nadu, Winged Wisdom",
        "the ability can trigger twice each turn for each creature you control"
    );
    supported("Nadu, Winged Wisdom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nadu, Winged Wisdom");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    for n in 1..=2 {
        grow(&mut t, bears);
        t.settle();
        assert_eq!(nadu_triggers(&t), 1, "targeting {n}");
        t.resolve_all();
    }
    grow(&mut t, bears);
    t.settle();
    assert_eq!(nadu_triggers(&t), 0, "a third time this turn");
    t.resolve_all();
    // Another creature has its own count.
    grow(&mut t, elves);
    t.settle();
    assert_eq!(nadu_triggers(&t), 1);
    t.resolve_all();
    // A new turn: twice again.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    grow(&mut t, bears);
    t.settle();
    assert_eq!(nadu_triggers(&t), 1);
    t.resolve_all();
}

#[test]
fn a_new_nadu_grants_a_new_ability_with_its_own_count() {
    cr!("603.2", "400.7");
    ruling!(
        "Nadu, Winged Wisdom",
        "the triggered ability granted by the new one is different than the triggered ability granted by the old one"
    );
    let mut t = TestGame::new(2);
    let nadu = t.battlefield(P0, "Nadu, Winged Wisdom");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        grow(&mut t, bears);
        t.settle();
        assert_eq!(nadu_triggers(&t), 1);
        t.resolve_all();
    }
    // Nadu leaves and returns this turn (targeting it triggers its own ability).
    let shift = t.hand(P0, "Cloudshift");
    mana(&mut t, P0);
    t.cast_with(P0, shift, &[Entity::Object(nadu)]).unwrap();
    t.resolve_all();
    assert_ne!(t.g.current(nadu), nadu);
    grow(&mut t, bears);
    t.settle();
    assert_eq!(nadu_triggers(&t), 1, "the new Nadu's ability triggers");
    t.resolve_all();
}

#[test]
fn sapphire_collector_conjures_on_the_second_noncreature_spell_in_a_turn_once() {
    cr!("603.2");
    supported("Sapphire Collector");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sapphire Collector");
    let cast_growth = |t: &mut TestGame| {
        let bears = t.battlefield(P0, "Grizzly Bears");
        grow(t, bears);
        t.resolve_all();
    };
    cast_growth(&mut t);
    assert!(!t.in_hand(P0, "Mox Sapphire"));
    cast_growth(&mut t);
    assert!(t.in_hand(P0, "Mox Sapphire"));
    // The third noncreature spell this turn isn't the second.
    cast_growth(&mut t);
    let moxes = |t: &TestGame| {
        t.g.players[0]
            .hand
            .iter()
            .filter(|c| t.g.obj(**c).chars.name.as_str() == "Mox Sapphire")
            .count()
    };
    assert_eq!(moxes(&t), 1);
    // Next turn, the second noncreature spell: only once.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    cast_growth(&mut t);
    cast_growth(&mut t);
    assert_eq!(moxes(&t), 1);
}
