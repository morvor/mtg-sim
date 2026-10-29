//! Rulings batch S34 — Disciple of Deceit ("Inspired — Whenever this creature becomes
//! untapped, you may discard a nonland card. If you do, search your library for a card
//! with the same mana value as that card, reveal it, put it into your hand, then
//! shuffle."): a split card's mana value is the combined mana value of its halves off the
//! stack (CR 709.4b), and {X} is 0 off the stack (CR 202.3e, 107.3g).

use crate::r_s01_common::*;
use crate::r_s04_common::next_upkeep;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's tapped Disciple of Deceit untaps in P0's next untap step; P0 discards `discard`
/// and searches for `find`. Returns the cards P0 could find.
fn disciple(t: &mut TestGame, discard: ObjectId, find: ObjectId) -> Vec<Entity> {
    supported("Disciple of Deceit");
    let disciple = t.battlefield(P0, "Disciple of Deceit");
    t.g.tap(disciple);
    next_upkeep(t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(discard)]);
    t.answer_choose(P0, &[Entity::Object(find)]);
    let from = t.asked().len();
    t.resolve_all();
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .last()
        .expect("a search")
}

#[test]
fn disciple_of_deceit_finds_the_combined_mana_value_of_a_split_card() {
    cr!("709.4b", "202.3d", "701.23a");
    ruling!(
        "Disciple of Deceit",
        "The mana value of a split card is determined by the combined mana cost of its two halves."
    );
    // Fire // Ice ({1}{R} // {1}{U}) has mana value 4: Hill Giant (4) can be found,
    // Grizzly Bears (2) can't.
    let mut t = TestGame::new(2);
    let fire_ice = t.hand(P0, "Fire // Ice");
    let giant = t.library_top(P0, "Hill Giant");
    let bears = t.library_top(P0, "Grizzly Bears");
    let found = disciple(&mut t, fire_ice, giant);
    assert!(found.contains(&Entity::Object(giant)));
    assert!(!found.contains(&Entity::Object(bears)));
    assert!(t.in_graveyard(P0, "Fire // Ice"));
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn disciple_of_deceit_treats_x_as_0_for_both_cards() {
    cr!("202.3e", "107.3g", "701.23a");
    ruling!(
        "Disciple of Deceit",
        "If there's an {X} in the mana cost of the card you discarded or the card you wish to search for, X is 0."
    );
    // Discarding Blaze ({X}{R}, mana value 1) finds Ingenious Prodigy ({X}{U}, mana value
    // 1), not Grizzly Bears (2).
    let mut t = TestGame::new(2);
    let blaze = t.hand(P0, "Blaze");
    let prodigy = t.library_top(P0, "Ingenious Prodigy");
    let bears = t.library_top(P0, "Grizzly Bears");
    let found = disciple(&mut t, blaze, prodigy);
    assert!(found.contains(&Entity::Object(prodigy)));
    assert!(!found.contains(&Entity::Object(bears)));
    assert!(t.in_hand(P0, "Ingenious Prodigy"));
    // A land can't be discarded to it ("a nonland card"): nothing is discarded or found.
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    let bears = t.library_top(P0, "Grizzly Bears");
    let disciple = t.battlefield(P0, "Disciple of Deceit");
    t.g.tap(disciple);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Forest"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    assert!(t.g.is_live(bears));
}
