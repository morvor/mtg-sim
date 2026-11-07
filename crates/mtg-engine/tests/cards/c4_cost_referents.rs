//! cards4-referents-statics, family `cost-referents`: what a cost did, read by the effect
//! (CR 400.7j, 608.2h).

use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// Activates Wonderscape Sage returning `land`; the cards in hand before and after.
fn sage(land: &str) -> (usize, usize, usize) {
    let mut t = TestGame::new(2);
    let sage = t.battlefield(P0, "Wonderscape Sage");
    let l = t.battlefield(P0, land);
    t.hand(P0, "Grizzly Bears");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[Entity::Object(l)]);
    t.activate(P0, sage, 0, &[]).expect("activates");
    t.resolve_all();
    (hand, t.hand_size(P0), t.graveyard_size(P0))
}

#[test]
fn wonderscape_sage_discards_unless_the_returned_land_had_a_nonbasic_land_type() {
    cr!("205.3i", "400.7", "608.2h");
    supported("Wonderscape Sage");
    // A Gate: the land returns to the hand, a card is drawn, nothing discarded.
    let (before, after, gy) = sage("Azorius Guildgate");
    assert_eq!((after, gy), (before + 2, 0));
    // An Island: also discard a card.
    let (before, after, gy) = sage("Island");
    assert_eq!((after, gy), (before + 1, 1));
}
