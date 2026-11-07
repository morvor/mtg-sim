//! Suppressor Skyguard (hand-written, `src/cards/suppressor_skyguard.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn prevents_combat_damage_to_you_if_another_opponent_isnt_attacked() {
    cr!("603.4", "615.1a");
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Suppressor Skyguard");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn no_prevention_if_every_opponent_is_attacked() {
    cr!("603.4");
    ruling!("Suppressor Skyguard", "if any of the attacking player's opponents besides you aren't being attacked");
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Suppressor Skyguard");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P0, "Gray Ogre");
    t.attack(
        &[(bear, Entity::Player(P1)), (ogre, Entity::Player(P2))],
        &[],
    );
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
    let _ = Step::End;
}

#[test]
fn two_players_no_other_opponent() {
    cr!("603.4");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Suppressor Skyguard");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}
