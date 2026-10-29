//! Next-time delayed triggers over an event list or with a qualifier (cards4-triggers-
//! durations): "When you next cast an instant spell, cast a sorcery spell, or activate a
//! loyalty ability this turn" (Repeated Reverberation), "copy the next loyalty ability you
//! activate this turn when you activate it" (Jaya's Phoenix), "When you next activate an
//! ability that isn't a mana ability this turn by spending four or more mana to activate
//! it" (Dynaheir, Invoker Adept).

use crate::combat2_common::{declare, supported, to_end_of_combat};
use crate::td_next_turn_durations::mana;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn ready(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].tapped = false;
    t.g.objects[id.0 as usize].summoning_sick = false;
}

fn cast_reverberation(t: &mut TestGame) {
    let rr = t.hand(P0, "Repeated Reverberation");
    mana(t, P0);
    t.cast_with(P0, rr, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn repeated_reverberation_copies_the_next_instant_twice() {
    cr!("603.7b", "603.7c", "707.10");
    ruling!(
        "Repeated Reverberation",
        "The copies made by Repeated Reverberation’s ability are created on the stack"
    );
    supported("Repeated Reverberation");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    cast_reverberation(&mut t);
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 11, "the Bolt and two copies");
    // Only the next one.
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 8);
}

#[test]
fn repeated_reverberation_copies_a_loyalty_ability_twice() {
    cr!("603.7b", "707.10", "606.3");
    ruling!(
        "Repeated Reverberation",
        "Copying a loyalty ability doesn’t add or remove loyalty counters from any object."
    );
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    t.advance_to(P0, Step::PrecombatMain);
    cast_reverberation(&mut t);
    let loyalty = t.counters(ajani, "loyalty");
    t.activate(P0, ajani, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 26, "+1: gain 2 life, and two copies");
    assert_eq!(t.counters(ajani, "loyalty"), loyalty + 1);
}

#[test]
fn repeated_reverberation_ends_with_the_turn() {
    cr!("603.7b");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    cast_reverberation(&mut t);
    t.advance_to(P1, Step::PrecombatMain);
    let bolt = t.hand(P0, "Lightning Bolt");
    mana(&mut t, P0);
    t.cast_with(P0, bolt, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn jayas_phoenix_copies_the_next_loyalty_ability_after_combat_damage() {
    cr!("603.7b", "603.7c", "707.10");
    ruling!(
        "Jaya's Phoenix",
        "Copying a loyalty ability doesn't add or remove loyalty counters from any object."
    );
    supported("Jaya's Phoenix");
    let mut t = TestGame::new(2);
    let phoenix = t.battlefield(P0, "Jaya's Phoenix");
    ready(&mut t, phoenix);
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    declare(&mut t, P0, &[(phoenix, Entity::Player(P1))]);
    to_end_of_combat(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::PostcombatMain);
    t.activate(P0, ajani, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 24, "+1: gain 2 life, and a copy");
}

#[test]
fn dynaheir_copies_an_ability_activated_with_four_mana() {
    cr!("603.7b", "603.7c", "707.10", "602.2b");
    supported("Dynaheir, Invoker Adept");
    let mut t = TestGame::new(2);
    let dyn_ = t.battlefield(P0, "Dynaheir, Invoker Adept");
    ready(&mut t, dyn_);
    let stone = t.battlefield(P0, "Mind Stone");
    let tome = t.battlefield(P0, "Jayemdae Tome");
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, dyn_, 0, &[]).unwrap();
    t.resolve_all();
    // One mana: not copied, and the delayed ability still waits.
    let hand = t.hand_size(P0);
    mana(&mut t, P0);
    t.activate(P0, stone, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Four mana: copied.
    mana(&mut t, P0);
    t.activate(P0, tome, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}
