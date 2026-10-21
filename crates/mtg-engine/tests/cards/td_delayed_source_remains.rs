//! "... you may return that card to the battlefield at the beginning of the next end step
//! if Shirei is still on the battlefield." (Shirei, Shizo's Caretaker).

use crate::combat2_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn shirei_returns_small_creatures_at_the_next_end_step() {
    cr!("603.7a", "603.4");
    ruling!(
        "Shirei, Shizo's Caretaker",
        "only triggers if the creature's power immediately before leaving the battlefield is 1 or less"
    );
    supported("Shirei, Shizo's Caretaker");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Shirei, Shizo's Caretaker");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.destroy(elves, None);
    t.g.destroy(bears, None);
    t.resolve_all();
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    // Only at that end step.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

#[test]
fn shirei_return_is_optional() {
    cr!("603.7a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Shirei, Shizo's Caretaker");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.destroy(elves, None);
    t.resolve_all();
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
}

#[test]
fn shirei_that_left_and_returned_is_a_new_object() {
    cr!("400.7", "603.4");
    ruling!(
        "Shirei, Shizo's Caretaker",
        "If Shirei leaves the battlefield and returns to the battlefield after a creature with power 1 or less is put into your graveyard, Shirei is considered a new object."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let shirei = t.battlefield(P0, "Shirei, Shizo's Caretaker");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.g.destroy(elves, None);
    t.resolve_all();
    // Flicker Shirei.
    t.g.move_object(
        shirei,
        mtg_engine::object::Zone::Exile,
        events::MoveCause::Effect,
        Some(P0),
    );
    let exiled = t.g.current(shirei);
    let back = t
        .g
        .move_object(
            exiled,
            mtg_engine::object::Zone::Battlefield,
            events::MoveCause::Effect,
            Some(P0),
        )
        .expect("returned");
    assert!(t.on_battlefield(back));
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Llanowar Elves").is_empty());
}
