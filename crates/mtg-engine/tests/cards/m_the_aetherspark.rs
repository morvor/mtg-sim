//! The Aetherspark (hand-written, `src/cards/the_aetherspark.rs`).

use mtg_engine::combat::attack_targets;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn cant_be_attacked_while_attached_to_a_creature() {
    cr!("508.1b", "506.3c");
    let mut t = TestGame::new(2);
    let spark = t.battlefield(P1, "The Aetherspark");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(attack_targets(&t.g).contains(&Entity::Object(spark)));
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.attach(spark, Entity::Object(bear));
    t.g.recompute();
    assert!(!attack_targets(&t.g).contains(&Entity::Object(spark)));
}

#[test]
fn gains_loyalty_from_the_equipped_creatures_combat_damage() {
    cr!("510.2");
    let mut t = TestGame::new(2);
    let spark = t.battlefield(P0, "The Aetherspark");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(spark, Entity::Object(bear));
    t.g.recompute();
    let before = t.counters(spark, "loyalty");
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.counters(spark, "loyalty"), before + 2);
}
