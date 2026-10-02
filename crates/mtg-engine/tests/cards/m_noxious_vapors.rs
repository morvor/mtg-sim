//! Noxious Vapors (hand-written, `src/cards/noxious_vapors.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn keeps_one_card_of_each_color_and_lands() {
    cr!("701.9a", "701.20a");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.hand(P1, "Shock");
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Forest");
    t.lands(P0, "Swamp", 3);
    let s = t.hand(P0, "Noxious Vapors");
    t.cast(P0, s).go();
    t.answer_choose(P1, &[bolt.into()]);
    t.resolve();
    assert!(t.in_hand(P1, "Lightning Bolt"));
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_hand(P1, "Forest"));
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.hand_size(P1), 3);
}
