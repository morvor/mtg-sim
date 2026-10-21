//! Jared Carthalion, True Heir (hand-written, `src/cards/jared_carthalion_true_heir.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn opponent_becomes_monarch_and_you_cant_this_turn() {
    cr!("725.1");
    ruling!("Jared Carthalion, True Heir", "the current monarch stays the monarch");
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Jared Carthalion, True Heir");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P1));
    // "When this creature enters, you become the monarch."
    t.enter(P0, "Canal Courier");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P1));
    // Next turn, it can.
    t.advance_to(P1, Step::Upkeep);
    t.enter(P0, "Canal Courier");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
}
