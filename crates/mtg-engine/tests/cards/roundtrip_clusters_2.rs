//! In-game tests for compiler misreads found by the Oracle round trip (follow-up item
//! `roundtrip-clusters-2`): each shows the behavior the corrected compilation has and
//! the old one didn't.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn a_search_for_up_to_x_cards_may_find_fewer() {
    cr!("701.23d");
    // Diabolic Revelation: "Search your library for up to X cards, put those cards into
    // your hand, then shuffle." "Up to" was dropped, and a search for a quantity of cards
    // must find that many (CR 701.23d): the player had to take X cards.
    supported("Diabolic Revelation");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::PrecombatMain);
    let wanted = t.library_top(P0, "Grizzly Bears");
    for _ in 0..4 {
        t.library_top(P0, "Forest");
    }
    let spell = t.hand(P0, "Diabolic Revelation");
    t.lands(P0, "Swamp", 8);
    t.answer_choose(P0, &[Entity::Object(wanted)]);
    t.cast(P0, spell).x(3).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
