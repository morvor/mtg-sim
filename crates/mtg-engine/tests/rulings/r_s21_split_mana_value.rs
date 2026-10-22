//! Rulings batch S21 — the mana value of a split card (aftermath included) not on the
//! stack is based on the combined mana costs of its halves (CR 202.3, 709.4).

use crate::r_s01_common::*;
use crate::r_s04_common::next_upkeep;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn twilight_prophet_counts_both_halves_of_an_aftermath_card() {
    cr!("202.3", "709.4", "709.4b");
    ruling!(
        "Twilight Prophet",
        "The mana value of a split card, such as cards with aftermath from the Amonkhet block, is based on the combined mana cost of its two halves."
    );
    supported("Twilight Prophet");
    supported("Destined // Lead");
    let mut t = TestGame::new(2);
    // "Ascend. At the beginning of your upkeep, if you have the city's blessing, reveal
    // the top card of your library and put it into your hand. Each opponent loses X life
    // and you gain X life, where X is that card's mana value."
    t.battlefield(P0, "Twilight Prophet");
    t.lands(P0, "Swamp", 9);
    // Destined {1}{B} // Lead {3}{G} (aftermath): mana value 2 + 4 = 6.
    t.library_top(P0, "Destined // Lead");
    next_upkeep(&mut t, P0);
    assert!(t.g.player(P0).has_citys_blessing);
    t.resolve_all();
    assert!(t.in_hand(P0, "Destined // Lead"));
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 26);
}
