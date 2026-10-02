//! CR 108.2b, 111.7: a token that has left the battlefield isn't a card, though it stays
//! in its new zone until the next state-based action check. Counts of "cards in your
//! graveyard" made in the meantime (in the middle of a resolving spell) don't count it.

use crate::r703_common::oracle_card;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls a creature token and has `cards` cards in their graveyard; casts `spell`
/// (a sorcery costing {B}) and resolves it. Returns P0's hand size before and after.
fn run(cards: usize, text: &str) -> (TestGame, usize, usize) {
    let mut t = TestGame::new(2);
    for _ in 0..cards {
        t.graveyard(P0, "Grizzly Bears");
    }
    let token = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[token.0 as usize].kind = ObjKind::Token;
    t.lands(P0, "Swamp", 1);
    let def = oracle_card("Token Test", "Sorcery", "{B}", None, text);
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, spell).go();
    let before = t.hand_size(P0);
    t.resolve();
    let after = t.hand_size(P0);
    (t, before, after)
}

#[test]
fn a_token_in_a_graveyard_isnt_counted_as_a_card() {
    cr!("108.2b", "111.7");
    let text = "Sacrifice a creature token. Then if you have seven or more cards in your graveyard, draw a card.";
    // Six cards and the sacrificed token: not seven cards.
    let (t, before, after) = run(6, text);
    assert_eq!(after, before, "{}", t.dump_log());
    // Seven cards: the condition holds.
    let (_, before, after) = run(7, text);
    assert_eq!(after, before + 1);
}

#[test]
fn the_number_of_cards_in_a_graveyard_doesnt_include_tokens() {
    cr!("108.2b", "111.7");
    let text = "Sacrifice a creature token. Then you gain life equal to the number of cards in your graveyard.";
    let (t, _, _) = run(3, text);
    assert_eq!(t.life(P0), 23, "{}", t.dump_log());
}
