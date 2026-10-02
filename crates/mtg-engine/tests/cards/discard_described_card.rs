//! "you may discard a land card. If you do, ...", "you may discard a nonland card. When you
//! do, ...": discarding a card of a described kind (CR 701.9a); without one in hand,
//! nothing is discarded and the "if you do" part doesn't happen.

use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

fn wolves(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Wolf"))
        .count()
}

#[test]
fn pack_guardian_discards_a_land_card_for_a_wolf() {
    cr!("701.9a", "603.12");
    compiles("Pack Guardian");
    // "When this creature enters, you may discard a land card. If you do, create a 2/2
    // green Wolf creature token."
    let mut t = TestGame::new(2);
    t.hand(P0, "Grizzly Bears");
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.enter(P0, "Pack Guardian");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Forest"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(wolves(&t), 1);
    // With no land card in hand, nothing is discarded and there's no Wolf.
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Pack Guardian");
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(wolves(&t), 0);
}

#[test]
fn hypothesizzle_discards_a_nonland_card_to_deal_damage() {
    cr!("701.9a", "603.12");
    compiles("Hypothesizzle");
    // "Draw two cards. Then you may discard a nonland card. When you do, Hypothesizzle
    // deals 4 damage to target creature."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Hypothesizzle");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
}
