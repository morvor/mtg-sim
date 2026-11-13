//! Become the Pilot (hand-written, `src/cards/become_the_pilot.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_stolen_creature_cant_be_blocked_unless_attacking_its_owner() {
    cr!("509.1b");
    let mut t = TestGame::new(3);
    // P0 controls P1's Grizzly Bears with the Aura.
    let bear = t.battlefield(P1, "Grizzly Bears");
    let pilot = t.battlefield(P0, "Become the Pilot");
    t.g.attach(pilot, Entity::Object(bear));
    t.g.recompute();
    assert_eq!(t.obj_now(bear).controller, P0);
    // Controlled continuously since P0's next turn began.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.pt(bear), (4, 4));
    let wall = t.battlefield(P2, "Wall of Stone");
    t.attack(&[(bear, Entity::Player(P2))], &[(wall, bear)]);
    assert_eq!(t.life(P2), 16);
}

#[test]
fn it_can_be_blocked_attacking_its_owner() {
    cr!("509.1b");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let pilot = t.battlefield(P0, "Become the Pilot");
    t.g.attach(pilot, Entity::Object(bear));
    t.g.recompute();
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let wall = t.battlefield(P1, "Wall of Stone");
    t.attack(&[(bear, Entity::Player(P1))], &[(wall, bear)]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.obj_now(wall).damage, 4);
}
