//! The worked example of CR 305 (lands), played out as a test.

use super::r205_examples::{is_mountain_land, type_line_examples};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn basic_land_mountain_is_a_land_with_the_subtype_mountain() {
    cr!("305.5");
    cr_example!("305.5", "Basic Land — Mountain");
    let mut t = TestGame::new(2);
    let (mountain, _, _) = type_line_examples(&mut t);
    assert!(is_mountain_land(&t, mountain));
}
