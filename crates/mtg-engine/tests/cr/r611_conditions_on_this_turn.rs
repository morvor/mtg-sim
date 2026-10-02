//! CR 611.3a: a continuous effect from a static ability applies at any given moment to
//! whatever its text indicates — including a condition about what happened this turn,
//! which becomes true as soon as it happens.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_bonus_for_spells_cast_this_turn_applies_as_soon_as_the_second_is_cast() {
    cr!("611.3a");
    let mut t = TestGame::new(2);
    let zealot = t.battlefield(P0, "Brightspear Zealot");
    t.lands(P0, "Mountain", 2);
    let first = t.hand(P0, "Shock");
    t.cast(P0, first).target(P1).go();
    t.resolve();
    t.settle();
    assert_eq!(t.pt(zealot), (2, 4));
    let second = t.hand(P0, "Shock");
    t.cast(P0, second).target(P1).go();
    // The second spell is on the stack: "you've cast two or more spells this turn".
    t.settle();
    assert_eq!(t.pt(zealot), (4, 4));
}
