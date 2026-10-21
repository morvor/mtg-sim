//! Cunning Giant (hand-written, `src/cards/cunning_giant.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn unblocked_it_may_assign_its_damage_to_a_creature() {
    cr!("510.1a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Cunning Giant");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(ogre)]);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Gray Ogre"));
}

#[test]
fn or_to_the_player_as_usual() {
    cr!("510.1a");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Cunning Giant");
    t.battlefield(P1, "Gray Ogre");
    t.answer_yes(P0, false);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
}
