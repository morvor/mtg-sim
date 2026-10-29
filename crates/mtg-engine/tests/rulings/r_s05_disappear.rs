//! Rulings batch S05 — disappear (an ability word): "if a permanent left the battlefield
//! under your control this turn".

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_disappear_trigger_with_an_intervening_if_doesnt_trigger_unless_a_permanent_left() {
    cr!("603.4");
    ruling!(
        "Foot Mystic",
        "Some disappear abilities, such as those of the above cards, are triggered abilities with intervening if clauses. If a permanent hasn't left the battlefield under your control that turn by the time the disappear ability would trigger, it won't trigger at all."
    );
    supported("Foot Mystic");
    supported("Lord Dregg, Insect Invader");
    // Foot Mystic: "When this creature enters, if a permanent left the battlefield under
    // your control this turn, create a 1/1 black Ninja creature token."
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Foot Mystic");
    assert_eq!(t.stack_len(), 0, "it didn't trigger at all");
    // An opponent's permanent leaving doesn't count.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, theirs);
    enter(&mut t, P0, "Foot Mystic");
    assert_eq!(t.stack_len(), 0);
    // Once one of P0's permanents has left, it triggers.
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    enter(&mut t, P0, "Foot Mystic");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Ninja").len(), 1);

    // Lord Dregg: "At the beginning of your end step, if a permanent left the battlefield
    // under your control this turn, create a 1/1 black Insect Warrior creature token with
    // flying." Nothing left: no trigger in the end step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lord Dregg, Insect Invader");
    t.advance_to(P0, Step::End);
    assert_eq!(t.stack_len(), 0);
    assert!(tokens_with_subtype(&t, P0, "Insect").is_empty());
    // The next turn, a permanent leaves first.
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Insect").len(), 1);
}
