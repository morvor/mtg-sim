//! CR 509.3a, 603.2: "Whenever this creature blocks or becomes blocked, it gets ..." triggers
//! when blockers are declared, for the attacking creature as well as for a blocker, and
//! "it" is that creature.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn raging_gorilla_gets_plus_two_minus_two_when_it_becomes_blocked() {
    cr!("509.3a", "509.3c", "603.2");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // A 2/3: "Whenever this creature blocks or becomes blocked, it gets +2/-2 until end of
    // turn."
    let gorilla = t.battlefield(P0, "Raging Gorilla");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(gorilla, Entity::Player(P1))], &[(bears, gorilla)]);
    // As a 4/1 it kills the Bears and dies to their 2 damage.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Raging Gorilla"));
}

#[test]
fn raging_gorilla_gets_plus_two_minus_two_when_it_blocks() {
    cr!("509.3a", "509.3c", "603.2");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let gorilla = t.battlefield(P0, "Raging Gorilla");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(bears, Entity::Player(P0))], &[(gorilla, bears)]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Raging Gorilla"));
}
