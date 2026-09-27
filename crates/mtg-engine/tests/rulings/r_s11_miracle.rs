//! Rulings batch S11 — miracle (CR 702.94): "You may reveal this card from your hand as you
//! draw it if it's the first card you've drawn this turn. When you reveal this card this
//! way, you may cast it by paying [cost] rather than its mana cost." Terminus: "Put all
//! creatures on the bottom of their owners' libraries. Miracle {W}".

use crate::r_s01_common::*;
use crate::r_s07_common::cast_methods;
use crate::r_s11_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const MIRACLE: CastMethod = CastMethod::Keyword(KeywordKind::Miracle);

#[test]
fn a_card_put_into_your_hand_without_drawing_it_wasnt_drawn() {
    cr!("702.94a", "121.1", "121.2");
    ruling!(
        "Terminus",
        "If an effect puts a card into your hand without using the word \"draw,\" the card wasn't drawn."
    );
    supported("Terminus");
    supported("Dark Confidant");
    // Dark Confidant: "At the beginning of your upkeep, reveal the top card of your
    // library and put that card into your hand. You lose life equal to its mana value."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dark Confidant");
    let second = t.library_top(P0, "Terminus");
    let first = t.library_top(P0, "Terminus");
    t.lands(P0, "Plains", 1);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let first = t.g.current(first);
    assert_eq!(t.zone(first), Zone::Hand(P0));
    assert_eq!(t.life(P0), 14);
    // It wasn't drawn: no miracle trigger, and it can't be cast for its miracle cost.
    assert_eq!(triggered_from(&t, first), 0);
    assert!(!cast_methods(&mut t, P0, first).contains(&MIRACLE));
    // The card drawn in the draw step is the first card drawn this turn.
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, true); // cast
    t.advance_to(P0, Step::Draw);
    t.resolve_all();
    let second = t.g.current(second);
    assert_eq!(t.zone(second), Zone::Graveyard(P0));
    assert!(t.named_on_battlefield("Dark Confidant").is_empty());
    assert_eq!(t.zone(first), Zone::Hand(P0));
}

#[test]
fn a_miracle_card_can_be_cast_for_its_miracle_cost_only_as_the_trigger_resolves() {
    cr!("702.94a", "603.11");
    ruling!(
        "Terminus",
        "You can cast a card for its miracle cost only as the miracle triggered ability resolves. If you don't want to cast it at that time (or you can't cast it, perhaps because there are no legal targets available), you won't be able to cast it later for the miracle cost."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = t.library_top(P0, "Terminus");
    t.lands(P0, "Plains", 6);
    t.answer_yes(P0, true); // reveal
    t.answer_yes(P0, false); // don't cast now
    t.g.draw_cards(P0, 1);
    t.settle();
    let card = t.g.current(card);
    assert_eq!(triggered_from(&t, card), 1);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Hand(P0));
    assert!(t.on_battlefield(bears));
    // Later that turn, only its mana cost is available.
    let methods = cast_methods(&mut t, P0, card);
    assert!(methods.contains(&CastMethod::Normal));
    assert!(!methods.contains(&MIRACLE));
    // Tapping the six Plains for its mana cost works.
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Library(P1));
    assert_eq!(tapped_lands(&t, P0), 6);
}
