//! Delayed and "this turn" triggered abilities created by effects
//! (cards3-triggers-durations; CR 603.7).

use crate::r_s01_common::supported;
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn suppress_returns_the_cards_at_the_end_step_of_that_players_next_turn() {
    cr!("603.7a", "603.7c");
    supported("Suppress");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Island");
    t.library_top(P1, "Forest");
    cast_new(&mut t, P0, "Suppress", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    // Not at P0's end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
    // At the end step of P1's next turn (P1 drew one card for the turn).
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Island"));
}
