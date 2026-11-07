//! The worked example of CR 302 (creatures), played out as a test.

use super::r205_examples::{is_goblin_wizard_creature, type_line_examples};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn creature_goblin_wizard_is_a_creature_with_subtypes_goblin_and_wizard() {
    cr!("302.3");
    cr_example!("302.3", "Creature — Goblin Wizard");
    let mut t = TestGame::new(2);
    let (_, goblin, _) = type_line_examples(&mut t);
    assert!(is_goblin_wizard_creature(&t, goblin));
}
