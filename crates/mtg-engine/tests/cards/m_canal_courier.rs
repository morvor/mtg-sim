//! Canal Courier (hand-written, `src/cards/canal_courier.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn unblockable_when_it_and_another_attack_different_players() {
    cr!("508.1", "509.1b");
    ruling!("Canal Courier", "Both Canal Courier and the other creature must attack different players");
    let mut t = TestGame::new(3);
    let courier = t.battlefield(P0, "Canal Courier");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.attack(
        &[(courier, Entity::Player(P1)), (bear, Entity::Player(P2))],
        &[(wall, courier)],
    );
    // The block was illegal: the Courier dealt its damage to P1.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn blockable_when_attacking_the_same_player() {
    cr!("509.1b");
    let mut t = TestGame::new(3);
    let courier = t.battlefield(P0, "Canal Courier");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.attack(
        &[(courier, Entity::Player(P1)), (bear, Entity::Player(P1))],
        &[(wall, courier)],
    );
    assert_eq!(t.life(P1), 18);
}
