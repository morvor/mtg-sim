//! "Whenever [enchanted creature | a Sliver] deals damage / is dealt damage, its controller
//! gains / loses that much life.": the player is the controller of the object the
//! trigger is about, not the ability's controller (CR 603.2, 119.3).

use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

#[test]
fn ragged_veins_the_enchanted_creatures_controller_loses_that_much_life() {
    cr!("603.2", "119.3");
    compiles("Ragged Veins");
    // P0's Ragged Veins ("Whenever enchanted creature is dealt damage, its controller
    // loses that much life.") on P1's Hill Giant; P0 Shocks the Giant (2 damage).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let veins = t.battlefield(P0, "Ragged Veins");
    assert!(t.g.attach(veins, Entity::Object(giant)));
    t.g.recompute();
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
}

#[test]
fn visions_of_brutality_the_enchanted_creatures_controller_loses_that_much_life() {
    cr!("603.2", "119.3");
    compiles("Visions of Brutality");
    // P0's Visions of Brutality ("Whenever enchanted creature deals damage, its
    // controller loses that much life.") on P1's Hill Giant (3/3); P0's Grizzly Bears
    // fights it (Prey Upon): the Giant deals 3 damage, so P1 loses 3.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let visions = t.battlefield(P0, "Visions of Brutality");
    assert!(t.g.attach(visions, Entity::Object(giant)));
    t.g.recompute();
    t.lands(P0, "Forest", 1);
    let prey = t.hand(P0, "Prey Upon");
    t.cast(P0, prey).target(bears).target(giant).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
}
