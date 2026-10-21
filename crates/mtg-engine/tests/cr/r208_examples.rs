//! The worked examples of CR 208 (power/toughness), played out as tests. The 208.2a
//! example is cited by a test in `r208_power_toughness.rs`.

use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn veteran_motorists_bonus_applies_once_the_vehicle_becomes_a_creature() {
    cr!("208.3a", "702.122b", "611.2c");
    cr_example!("208.3a", "Veteran Motorist");
    let mut t = TestGame::new(2);
    // Smuggler's Copter: a 3/3 Vehicle with crew 1.
    let copter = t.battlefield(P0, "Smuggler's Copter");
    // "Whenever this creature crews a Vehicle, that Vehicle gets +1/+1 until end of turn."
    let motorist = t.battlefield(P0, "Veteran Motorist");
    t.answer_choose(P0, &[Entity::Object(motorist)]);
    t.activate(P0, copter, 0, &[]).unwrap();
    t.clear_answers();
    assert!(t.obj_now(motorist).tapped);
    t.settle();
    // The trigger resolves first, while the Vehicle isn't a creature.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    assert!(!t.obj_now(copter).is(CardType::Creature));
    // The crew ability resolves: the effect applies to it.
    t.resolve();
    assert!(t.obj_now(copter).is(CardType::Creature));
    assert_eq!(t.pt(copter), (4, 4));
}
