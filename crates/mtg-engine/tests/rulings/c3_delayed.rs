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

#[test]
fn reincarnation_returns_a_creature_card_from_its_owners_graveyard() {
    cr!("603.7a", "603.7c", "603.7b");
    ruling!(
        "Reincarnation",
        "The creature card that will be returned to the battlefield isn't chosen until the delayed triggered ability resolves."
    );
    supported("Reincarnation");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    t.graveyard(P0, "Serra Angel");
    cast_new(&mut t, P0, "Reincarnation", &[Entity::Object(bears)]);
    t.resolve_all();
    // The Bears die: P0 chooses the Hill Giant from P1's graveyard; it enters under P1's
    // control (its owner's).
    t.answer_choose(P0, &[Entity::Object(giant)]);
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj_now(g[0]).controller, P1);
    assert!(t.in_graveyard(P0, "Serra Angel"));
}
