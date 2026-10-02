//! In-game tests for compiler misreads found by the Oracle round trip (follow-up item
//! `roundtrip-tail-1`, cards A–D): each shows the behavior the corrected compilation has
//! and the old one didn't.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn two_life_conditions_joined_by_and_are_both_checked() {
    cr!("611.3a");
    // Blood Baron of Vizkopa: "As long as you have 30 or more life and an opponent has 10
    // or less life, this creature gets +6/+6 and has flying." It was read as one condition
    // "you have 30 or less life", so the bonus applied at 20 life.
    supported("Blood Baron of Vizkopa");
    let mut t = TestGame::new(2);
    let baron = t.battlefield(P0, "Blood Baron of Vizkopa");
    t.settle();
    assert_eq!(t.pt(baron), (4, 4));
    t.g.player_mut(P0).life = 30;
    t.g.player_mut(P1).life = 10;
    t.g.dirty = true;
    t.settle();
    assert_eq!(t.pt(baron), (10, 10));
    t.g.player_mut(P0).life = 29;
    t.g.dirty = true;
    t.settle();
    assert_eq!(t.pt(baron), (4, 4));
}
