//! Raging River (hand-written, `src/cards/raging_river.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn an_attacker_can_be_blocked_only_from_its_side() {
    cr!("509.1b", "700.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging River");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    let giant = t.battlefield(P1, "Hill Giant");
    // P1: the Wall left, the Giant right. P0: the Bears left.
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.attack(&[(bear, Entity::Player(P1))], &[(giant, bear)]);
    // The Giant (right) couldn't block it.
    assert_eq!(t.life(P1), 18);
    let _ = wall;
}

#[test]
fn a_creature_on_the_same_side_can_block() {
    cr!("509.1b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raging River");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.attack(&[(bear, Entity::Player(P1))], &[(giant, bear)]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}
