//! CR 701.70: recruit.

use crate::a701_028_071_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn soldiers(t: &TestGame) -> Vec<ObjectId> {
    t.g.permanents()
        .filter(|o| {
            o.is_token() && o.chars.has_subtype("Human") && o.chars.has_subtype("Soldier")
        })
        .map(|o| o.id)
        .collect()
}

#[test]
fn recruit_draws_discards_and_creates_a_soldier_for_a_nonland_card() {
    cr!("701.70a");
    supported("Esgaroth Garrison");
    // "When this creature enters, recruit." Discarding a nonland card: a 1/1 white Human
    // Soldier creature token. (The card discarded by default is the first in hand.)
    let mut t = TestGame::new(2);
    t.hand(P0, "Lightning Bolt");
    let top = t.library_top(P0, "Forest");
    let _ = top;
    t.enter(P0, "Esgaroth Garrison");
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    let s = soldiers(&t);
    assert_eq!(s.len(), 1);
    assert_eq!(t.pt(s[0]), (1, 1));
    assert_eq!(t.obj(s[0]).chars.colors, ColorSet::single(Color::White));
    // Discarding a land card: no token.
    let mut t = TestGame::new(2);
    t.hand(P0, "Forest");
    t.library_top(P0, "Lightning Bolt");
    t.enter(P0, "Esgaroth Garrison");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(soldiers(&t).is_empty());
}
