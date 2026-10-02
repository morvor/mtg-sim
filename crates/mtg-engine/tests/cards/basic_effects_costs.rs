//! Additional costs: "behold a Kithkin and exile it" (CR 701.4a, 607.2q) and two costs
//! with their own verbs ("discard a card and sacrifice a creature", CR 601.2b).

use crate::basic_effects_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn champion_of_the_clachan_exiles_the_beheld_kithkin_and_returns_it() {
    cr!("701.4a", "607.2q");
    assert_supported("Champion of the Clachan");
    let mut t = TestGame::new(2);
    let harrier = t.battlefield(P0, "Goldmeadow Harrier");
    t.lands(P0, "Plains", 4);
    let champ = t.hand(P0, "Champion of the Clachan");
    t.answer_choose(P0, &[Entity::Object(harrier)]);
    t.cast(P0, champ).go();
    // The cost exiled the Harrier.
    assert!(t.in_exile("Goldmeadow Harrier"));
    t.resolve();
    let champ = t.named_on_battlefield("Champion of the Clachan")[0];
    t.g.destroy(champ, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Goldmeadow Harrier"));
}

#[test]
fn ruthless_disposal_discards_and_sacrifices() {
    cr!("601.2b", "601.2h");
    assert_supported("Ruthless Disposal");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Craw Wurm");
    t.hand(P0, "Island");
    t.lands(P0, "Swamp", 5);
    let rd = t.hand(P0, "Ruthless Disposal");
    t.cast(P0, rd)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    assert!(!t.on_battlefield(mine));
    assert!(t.in_graveyard(P0, "Island"));
    t.resolve();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
}
