//! A permanent put onto the battlefield with additional types: "That permanent is an
//! enchantment in addition to its other types." (CR 611.2e; pattern in
//! `src/oracle/patterns/zone_move_grammar.rs`).

use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn arbiter_of_the_ideal_puts_an_enchantment_with_a_manifestation_counter() {
    cr!("611.2e", "122.1");
    ruling!(
        "Arbiter of the Ideal",
        "The permanent will continue to be an enchantment in addition to its other types even if that counter is removed."
    );
    let u = card("Arbiter of the Ideal").unsupported_text().join(" | ");
    assert!(u.is_empty(), "unsupported: {u}");
    let mut t = TestGame::new(2);
    let arbiter = t.battlefield(P0, "Arbiter of the Ideal");
    let bears = t.library_top(P0, "Grizzly Bears");
    assert!(t.g.tap(arbiter));
    t.answer_yes(P0, true);
    assert!(t.g.untap(arbiter));
    t.g.flush_events();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.counters(bears, "manifestation"), 1);
    assert!(t.obj_now(bears).chars.card_types.contains(CardType::Enchantment));
    assert!(t.obj_now(bears).chars.card_types.contains(CardType::Creature));
    // Removing the counter doesn't change that.
    t.g.remove_counters(Entity::Object(bears), "manifestation", 1);
    t.g.recompute();
    assert!(t.obj_now(bears).chars.card_types.contains(CardType::Enchantment));
}

#[test]
fn arbiter_of_the_ideal_leaves_other_cards_on_top() {
    cr!("701.20a");
    ruling!(
        "Arbiter of the Ideal",
        "or if the card isn’t one of the listed types, it will remain on top of your library."
    );
    let mut t = TestGame::new(2);
    let arbiter = t.battlefield(P0, "Arbiter of the Ideal");
    let bolt = t.library_top(P0, "Lightning Bolt");
    assert!(t.g.tap(arbiter));
    t.answer_yes(P0, true);
    assert!(t.g.untap(arbiter));
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.g.player(P0).library.last().copied(), Some(bolt));
}
