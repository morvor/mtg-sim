//! CR 107.3a, 107.3b, 702.34a: "X can't be 0" doesn't stop a spell from being cast for an
//! alternative cost whose X is in a part other than its mana ("Flashback—{R}, Pay X
//! life."; Light Up the Night, not supported yet, has "Remove X loyalty counters from
//! among planeswalkers you control"): X is chosen for that cost. Only an alternative cost
//! with no X at all leaves 0 as the only legal value of X.

use crate::r703_common::oracle_card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn a_flashback_cost_with_x_in_its_life_payment_can_be_paid() {
    cr!("107.3a", "702.34a");
    let mut t = TestGame::new(2);
    let blast = t.custom(
        P0,
        oracle_card(
            "Test Blast",
            "Instant",
            "{X}{R}",
            None,
            "Test Blast deals X damage to any target. X can't be 0.\nFlashback—{R}, Pay X life.",
        ),
        Zone::Graveyard(P0),
    );
    t.lands(P0, "Mountain", 1);
    // It's offered as a legal action, and it can be cast with X = 2.
    let flashback = CastMethod::Keyword(KeywordKind::Flashback);
    assert!(t.g.legal_actions(P0).iter().any(
        |a| matches!(a, Action::Cast { card, method } if *card == blast && *method == flashback)
    ));
    t.cast(P0, blast)
        .method(flashback)
        .x(2)
        .target(Entity::Player(P1))
        .go();
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_exile("Test Blast"));
}
