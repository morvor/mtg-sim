//! Rulings batch S25 — a permanent that enters as a copy of another makes its own "as
//! [this] enters" choices (CR 707.2, 614.12c): the anchor-word abilities of a copied
//! Siege depend on the copy's choice (CR 607.2m).

use crate::r_s01_common::supported;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_copy_of_a_siege_makes_its_own_choice() {
    cr!("707.2", "614.12c", "607.2m");
    ruling!(
        "Citadel Siege",
        "If a permanent enters the battlefield as a copy of one of the Sieges, its controller will make a new choice for that Siege. Which ability the copy has won't depend on the choice made for the original permanent."
    );
    supported("Citadel Siege");
    supported("Copy Enchantment");
    // "As Citadel Siege enters, choose Khans or Dragons. • Khans — At the beginning of
    // combat on your turn, put two +1/+1 counters on target creature you control.
    // • Dragons — At the beginning of combat on each opponent's turn, tap target creature
    // that player controls."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let siege = t.enter(P0, "Citadel Siege");
    // Copy Enchantment enters as a copy of it; its controller chooses Dragons.
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    let copy = t.hand(P0, "Copy Enchantment");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(siege)]);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, copy).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Citadel Siege").len(), 2);
    // On P0's turn only the original (Khans) puts counters on the Bears.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // On P1's turn only the copy (Dragons) taps P1's creature.
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.advance_to(P1, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}
