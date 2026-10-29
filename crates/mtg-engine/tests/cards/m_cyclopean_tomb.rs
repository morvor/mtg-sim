//! Cyclopean Tomb (hand-written, `src/cards/cyclopean_tomb.rs`): once it's put into a
//! graveyard, at the beginning of each of its controller's upkeeps for the rest of the
//! game, all mire counters are removed from one land the Tomb put a mire counter on.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn is_swamp(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.subtypes.iter().any(|s| s.as_str() == "Swamp")
}

#[test]
fn after_it_dies_each_upkeep_one_of_its_lands_loses_its_mire_counters() {
    cr!("603.7", "603.7a");
    ruling!("Cyclopean Tomb", "triggers during each of your upkeeps for the rest of the game");
    ruling!("Cyclopean Tomb", "This effect is not tied to the Tomb remaining on the battlefield.");
    let mut t = TestGame::new(2);
    let tomb = t.battlefield(P0, "Cyclopean Tomb");
    let a = t.battlefield(P1, "Forest");
    let b = t.battlefield(P1, "Island");
    let other = t.battlefield(P1, "Mountain");
    t.set_step(P0, Step::Upkeep);
    t.lands(P0, "Plains", 4);
    t.activate(P0, tomb, 0, &[Entity::Object(a)]).unwrap();
    t.resolve_all();
    t.g.untap(tomb);
    t.activate(P0, tomb, 0, &[Entity::Object(b)]).unwrap();
    t.resolve_all();
    // A mire counter from elsewhere isn't the Tomb's.
    t.g.add_counters(Entity::Object(other), "mire", 1, None);
    t.g.recompute();
    assert!(is_swamp(&t, a) && is_swamp(&t, b) && !is_swamp(&t, other));
    t.g.destroy(tomb, None);
    t.settle();
    assert!(is_swamp(&t, a));
    // Next upkeep: one of the two (the first, by the default choice).
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    let left = [a, b].iter().filter(|l| t.counters(**l, "mire") > 0).count();
    assert_eq!(left, 1);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.counters(a, "mire"), 0);
    assert_eq!(t.counters(b, "mire"), 0);
    assert!(!is_swamp(&t, a) && !is_swamp(&t, b));
    // The other land keeps its counter.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.counters(other, "mire"), 1);
}
