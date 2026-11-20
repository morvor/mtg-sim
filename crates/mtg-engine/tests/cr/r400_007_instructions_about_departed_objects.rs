//! CR 400.7: "An object that moves from one zone to another becomes a new object with no
//! memory of, or relation to, its previous existence." An instruction about an object
//! that has left does nothing to it, nor to the new object it became. The actions.rs
//! primitives assert (in debug builds) that they're given live ids, so these paths, which
//! the liveness assertions found passing ids of objects that had left, now leave such
//! objects out themselves (`Game::present`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn doubling_the_counters_on_a_creature_that_left_does_nothing() {
    cr!("400.7", "701.10e");
    // Primordial Hydra: "At the beginning of your upkeep, double the number of +1/+1
    // counters on this creature." It's destroyed with the trigger on the stack.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Primordial Hydra");
    t.g.add_counters(Entity::Object(hydra), "+1/+1", 3, None);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(t.g.destroy(hydra, None));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Primordial Hydra"));
    let card = t.g.current(hydra);
    assert_eq!(t.counters(card, "+1/+1"), 0);
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // With the Hydra still there, the counters double.
    let mut t = TestGame::new(2);
    let hydra = t.battlefield(P0, "Primordial Hydra");
    t.g.add_counters(Entity::Object(hydra), "+1/+1", 3, None);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(hydra, "+1/+1"), 6, "{}", t.dump_log());
}
