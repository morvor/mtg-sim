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

/// P0's Grizzly Bears enchanted with Gift of the Woods ("Whenever enchanted creature
/// blocks or becomes blocked, it gets +0/+3 until end of turn and you gain 1 life.") in
/// combat with P1's Hill Giant, attacking (`attacks`) or blocking.
fn gift_of_the_woods(attacks: bool) -> TestGame {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gift = t.battlefield(P0, "Gift of the Woods");
    assert!(t.g.attach(gift, Entity::Object(bears)));
    let giant = t.battlefield(P1, "Hill Giant");
    if attacks {
        t.set_step(P0, Step::PrecombatMain);
        t.attack(&[(bears, Entity::Player(P1))], &[(giant, bears)]);
    } else {
        t.set_step(P1, Step::PrecombatMain);
        t.attack(&[(giant, Entity::Player(P0))], &[(bears, giant)]);
    }
    t
}

#[test]
fn an_enchanted_creature_that_becomes_blocked_is_it() {
    cr!("509.3a", "509.3c", "603.2");
    // The 2/5 Bears survive the Giant's 3 damage.
    let t = gift_of_the_woods(true);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn an_enchanted_creature_that_blocks_is_it() {
    cr!("509.3a", "509.3c", "603.2");
    let t = gift_of_the_woods(false);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.life(P0), 21);
}
