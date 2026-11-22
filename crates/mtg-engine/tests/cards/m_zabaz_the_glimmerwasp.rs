//! Zabaz, the Glimmerwasp (hand-written, `src/cards/zabaz_the_glimmerwasp.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn modular_puts_one_more_counter() {
    cr!("702.43a", "614.1a");
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zabaz, the Glimmerwasp");
    t.g.add_counters(Entity::Object(z), "+1/+1", 1, None);
    let worker = t.battlefield(P0, "Arcbound Worker");
    t.g.add_counters(Entity::Object(worker), "+1/+1", 1, None);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    t.answer_yes(P0, true);
    t.g.destroy(worker, None);
    t.resolve_all();
    assert_eq!(t.counters(thopter, "+1/+1"), 2);
}

#[test]
fn other_counters_are_unchanged() {
    cr!("614.1a");
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zabaz, the Glimmerwasp");
    t.g.add_counters(Entity::Object(z), "+1/+1", 1, None);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.g.add_counters(Entity::Object(thopter), "+1/+1", 1, None);
    assert_eq!(t.counters(thopter, "+1/+1"), 1);
}

#[test]
fn its_own_modular_isnt_affected() {
    cr!("702.43a");
    ruling!("Zabaz, the Glimmerwasp", "it won't increase the counters given by its own modular ability");
    let mut t = TestGame::new(2);
    let z = t.battlefield(P0, "Zabaz, the Glimmerwasp");
    t.g.add_counters(Entity::Object(z), "+1/+1", 1, None);
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    t.answer_yes(P0, true);
    t.g.destroy(z, None);
    t.resolve_all();
    assert_eq!(t.counters(thopter, "+1/+1"), 1);
}
