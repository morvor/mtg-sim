//! "As long as ~ is on the stack, [static ability]" (CR 113.6: an instant's or sorcery's
//! static ability that functions on the stack; `oracle/patterns/on_stack_static.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn kaerveks_torch_makes_spells_that_target_it_cost_more() {
    cr!("113.6", "601.2f");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let torch = t.hand(P0, "Kaervek's Torch");
    let spell = t.cast(P0, torch).x(1).target(Entity::Player(P1)).go();
    // Counterspell costs {U}{U} plus {2}: two Islands aren't enough.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    assert!(t.cast(P1, cs).target(spell).try_go().is_err());
    t.lands(P1, "Island", 2);
    t.cast(P1, cs).target(spell).go();
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn kaerveks_torch_doesnt_raise_the_cost_of_spells_targeting_something_else() {
    cr!("113.6", "601.2f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let torch = t.hand(P0, "Kaervek's Torch");
    t.cast(P0, torch).x(1).target(Entity::Player(P1)).go();
    // With the Torch on the stack, a Lightning Bolt aimed at the Bears costs just {R}.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 19);
}
