//! The worked examples of CR 118 (costs), played out as tests.

use super::r105_util::{card_from_text, put};
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_cost_whose_actions_were_modified_has_still_been_paid() {
    cr!("118.11", "702.24a");
    cr_example!("118.11", "The cumulative upkeep cost has still been paid.");
    let mut t = TestGame::new(2);
    // Psychic Vortex: "Cumulative upkeep—Draw a card." Obstinate Familiar: "If you would
    // draw a card, you may skip that draw instead."
    let vortex = t.battlefield(P0, "Psychic Vortex");
    t.battlefield(P0, "Obstinate Familiar");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    if t.g.turn.active == P0 {
        t.advance_to(P1, Step::Upkeep);
    }
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let (hand, library) = (t.hand_size(P0), t.library_size(P0));
    // P0 pays the cost (draw a card) and skips the draw.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.counters(vortex, "age"), 1);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), library);
    // The cost was paid: Psychic Vortex isn't sacrificed.
    assert!(t.on_battlefield(vortex));
}

#[test]
fn a_cost_paid_on_resolution_that_cant_be_paid_means_the_effect_doesnt_happen() {
    cr!("118.12");
    cr_example!("118.12", "No player will draw cards.");
    let mut t = TestGame::new(2);
    // Standstill: "When a player casts a spell, sacrifice this enchantment. If you do,
    // each of that player's opponents draws three cards."
    let standstill = t.battlefield(P0, "Standstill");
    for _ in 0..5 {
        t.library_top(P0, "Island");
        t.library_top(P1, "Island");
    }
    let banisher = put(
        &mut t,
        P1,
        card_from_text(
            "Banisher",
            "{0}",
            "Artifact",
            None,
            "{T}: Exile target enchantment.",
        ),
    );
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(P0).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // In response, an ability exiles Standstill.
    t.activate(P1, banisher, 0, &[Entity::Object(standstill)])
        .unwrap();
    t.resolve();
    assert!(t.in_exile("Standstill"));
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    // Standstill's ability resolves: the sacrifice can't be paid, so no one draws.
    t.resolve();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0, h1));
}

#[test]
fn a_cost_paid_on_resolution_counts_as_paid_even_if_its_action_was_replaced() {
    cr!("118.12", "614.12");
    cr_example!("118.12", "Dermoplasm is still returned to its owner's hand");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // The opponent has cast Gather Specimens: "If a creature would enter the battlefield
    // under an opponent's control this turn, it enters under your control instead."
    t.lands(P1, "Island", 6);
    let specimens = t.hand(P1, "Gather Specimens");
    t.g.turn.priority = Some(P1);
    t.cast(P1, specimens).go();
    t.resolve();
    // P0 controls a face-down Dermoplasm ("When this creature is turned face up, you may
    // put a creature card with a morph ability from your hand onto the battlefield face
    // up. If you do, return this creature to its owner's hand.").
    let plasm = t.battlefield(P0, "Dermoplasm");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, plasm));
    t.g.recompute();
    let phoenix = t.hand(P0, "Ashcloud Phoenix");
    t.lands(P0, "Island", 4);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        Action::Special(SpecialAction::TurnFaceUp { obj: plasm }),
    )
    .unwrap();
    assert!(!t.obj_now(plasm).face_down);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(phoenix)]);
    t.resolve_all();
    // The Phoenix enters under the opponent's control...
    let phoenix = t.g.current(phoenix);
    assert!(t.on_battlefield(phoenix));
    assert_eq!(t.obj_now(phoenix).controller, P1);
    // ...but P0 chose to pay the cost, so Dermoplasm is returned to its owner's hand.
    assert!(t.in_hand(P0, "Dermoplasm"));
}
