//! The worked examples of CR 113 (abilities), played out as tests. Most are cited by the
//! tests in `r113_abilities.rs`.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn absolver_thrulls_trigger_conditions_function_from_the_battlefield_and_from_exile() {
    cr!("113.6k", "702.55c");
    cr_example!("113.6k", "Absolver Thrull");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let moon = t.battlefield(P1, "Bad Moon");
    // "When this creature enters or the creature it haunts dies, destroy target
    // enchantment." The first condition functions from the battlefield.
    t.answer_targets(P0, &[Entity::Object(anthem)]);
    let thrull = t.enter(P0, "Absolver Thrull");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    // It dies and haunts a creature.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.g.destroy(thrull, None);
    t.resolve_all();
    assert!(t.in_exile("Absolver Thrull"));
    // The second condition functions from exile.
    t.answer_targets(P0, &[Entity::Object(moon)]);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Bad Moon"));
}
