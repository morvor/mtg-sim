//! Rulings batch P226 — planeswalkers that transform (CR 606.3, 712.18): the loyalty
//! ability limit is per permanent, so a planeswalker that transforms after activating
//! a loyalty ability can't activate one of its other face's that turn.

use crate::r_s01_common::with_subtype;
use crate::r_s02_common::can_activate;
use crate::r_s06_common::activate_containing;
use crate::r_s17_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const ARLINN: &str = "Arlinn Kord // Arlinn, Embraced by the Moon";

#[test]
fn arlinn_keeps_her_loyalty_and_cant_activate_again_after_transforming() {
    cr!("606.3", "712.18", "701.27a");
    ruling!(
        "Arlinn Kord // Arlinn, Embraced by the Moon",
        "When the ability that transforms Arlinn Kord into Arlinn, Embraced by the Moon (or vice versa) resolves, the number of loyalty counters on her doesn’t change."
    );
    ruling!(
        "Arlinn Kord // Arlinn, Embraced by the Moon",
        "You can’t activate a loyalty ability of Arlinn Kord and later that turn after she transforms activate a loyalty ability of Arlinn, Embraced by the Moon (or vice versa)."
    );
    // (Arlinn, Embraced by the Moon's −6 doesn't compile; the abilities used here do.)
    // Arlinn Kord (loyalty 3) "0: Create a 2/2 green Wolf creature token. Transform
    // Arlinn Kord."
    let mut t = TestGame::new(2);
    let arlinn = t.battlefield(P0, ARLINN);
    activate_containing(&mut t, P0, arlinn, "Wolf").unwrap();
    t.resolve_all();
    assert_eq!(name_of(&t, arlinn), "Arlinn, Embraced by the Moon");
    assert_eq!(t.counters(arlinn, counters::LOYALTY), 3);
    assert_eq!(with_subtype(&t, P0, "Wolf").len(), 1);
    // No loyalty ability of Arlinn, Embraced by the Moon this turn.
    assert!(!can_activate(&mut t, P0, arlinn));
    assert!(activate_containing(&mut t, P0, arlinn, "+1").is_err());
    // Next turn: "−1: Arlinn deals 3 damage to any target. Transform Arlinn." Then
    // Arlinn Kord can't activate a loyalty ability that turn either.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, arlinn, "3 damage").unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(name_of(&t, arlinn), "Arlinn Kord");
    assert_eq!(t.counters(arlinn, counters::LOYALTY), 2);
    assert!(!can_activate(&mut t, P0, arlinn));
}
