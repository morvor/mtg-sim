//! Common Cause (hand-written, `src/cards/common_cause.rs`): nonartifact creatures get
//! +2/+2 as long as they all share a color.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn all_creatures_sharing_a_color_get_the_bonus() {
    cr!("604.1", "613.4c");
    ruling!("Common Cause", "It really does mean “all creatures”, including your opponent’s.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Common Cause");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    // An artifact creature doesn't count and doesn't get the bonus.
    let golem = t.battlefield(P1, "Ornithopter");
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(b), (3, 3));
    assert_eq!(t.pt(golem), (0, 2));
}

#[test]
fn a_creature_of_another_color_turns_it_off() {
    cr!("604.1");
    ruling!("Common Cause", "If any creature is not of the shared color, the bonus is lost.");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Common Cause");
    let a = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    assert_eq!(t.pt(a), (2, 2));
}

#[test]
fn additional_colors_are_fine() {
    cr!("604.1");
    ruling!("Common Cause", "They can have additional colors beyond the one they share");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Common Cause");
    let a = t.battlefield(P0, "Grizzly Bears");
    // Green and white.
    let b = t.battlefield(P1, "Watchwolf");
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(b), (5, 5));
}
