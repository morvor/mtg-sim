//! Rulings batch P226 — Corruption of Towashi: "Whenever a permanent you control
//! transforms or a permanent you control enters transformed, you may draw a card. Do
//! this only once each turn." (CR 701.27, 712.14a), and incubate (CR 701.53).

use crate::r_s01_common::*;
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::{enter_transformed, name_of, transform};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const CAPTIVE: &str = "Wolfbitten Captive // Krallenhorde Killer";

#[test]
fn corruption_of_towashi_draws_when_a_permanent_transforms_either_way_once_a_turn() {
    cr!("701.27a", "712.14a", "701.53a", "603.2");
    ruling!(
        "Corruption of Towashi",
        "The last ability of Corruption of Towashi will trigger if a permanent you control transforms in either direction, going from front face up to back face up or vice versa."
    );
    supported("Corruption of Towashi");
    // "When this enchantment enters, incubate 4."
    let mut t = TestGame::new(2);
    t.enter(P0, "Corruption of Towashi");
    t.resolve_all();
    let incubator = with_subtype(&t, P0, "Incubator")[0];
    assert_eq!(t.counters(incubator, counters::PLUS1), 4);
    // Front to back: the Incubator's "{2}: Transform this token."
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    activate_containing(&mut t, P0, incubator, "Transform").unwrap();
    t.resolve_all();
    assert!(t.obj_now(incubator).chars.has_subtype("Phyrexian"));
    assert_eq!(t.hand_size(P0), 1);
    // Only once each turn.
    let captive = t.battlefield(P0, CAPTIVE);
    transform(&mut t, captive);
    assert_eq!(t.stack_len(), 0);
    // Next turn: Krallenhorde Killer back to Wolfbitten Captive (back to front). An
    // opponent's permanent doesn't count.
    t.advance_to(P1, Step::Upkeep);
    let theirs = t.battlefield(P1, CAPTIVE);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let hand = t.hand_size(P0);
    transform(&mut t, theirs);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(name_of(&t, captive), "Krallenhorde Killer");
    t.answer_yes(P0, true);
    transform(&mut t, captive);
    assert_eq!(name_of(&t, captive), "Wolfbitten Captive");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn corruption_of_towashi_draws_when_a_permanent_enters_transformed() {
    cr!("712.14a", "701.27g", "603.6a");
    // A double-faced card entering back face up triggers it; declining to draw doesn't
    // use up the turn's draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Corruption of Towashi");
    t.answer_yes(P0, false);
    enter_transformed(&mut t, P0, CAPTIVE);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    t.answer_yes(P0, true);
    enter_transformed(&mut t, P0, CAPTIVE);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // Entering front face up: no trigger.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    t.enter(P0, CAPTIVE);
    t.settle();
    assert_eq!(t.stack_len(), 0);
}
