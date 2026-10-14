//! Dina's Guidance: "Search your library for a creature card, reveal it, put it into your
//! hand or graveyard, then shuffle." The searcher chooses where the found card goes
//! (`SearchDest::or`, `oracle/patterns/search_grammar.rs`).

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn shuffles(t: &TestGame) -> usize {
    t.g.turn_events
        .iter()
        .filter(|e| matches!(e, Event::Shuffled { player } if *player == P0))
        .count()
}

#[test]
fn the_found_creature_card_goes_into_your_hand_or_graveyard() {
    cr!("701.23a", "701.24a");
    let c = card("Dina's Guidance");
    assert!(c.unsupported_text().is_empty(), "{:?}", c.unsupported_text());
    for (choice, zone) in [(0, Zone::Hand(P0)), (1, Zone::Graveyard(P0))] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        t.lands(P0, "Swamp", 1);
        let bears = t.library_top(P0, "Grizzly Bears");
        let bolt = t.library_top(P0, "Lightning Bolt");
        let spell = t.hand(P0, "Dina's Guidance");
        t.answer_choose(P0, &[Entity::Object(bears)]);
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        t.cast(P0, spell).go();
        t.resolve_all();
        let now = t.g.current(bears);
        assert_eq!(t.zone(now), zone);
        assert_eq!(t.zone(bolt), Zone::Library(P0));
        assert_eq!(shuffles(&t), 1);
        // The searcher was offered both places.
        let offered = t.asked().into_iter().any(|(p, d)| {
            p == P0 && matches!(d, Decision::ChooseOption { options, .. } if options.len() == 2)
        });
        assert!(offered);
    }
}

#[test]
fn a_player_who_finds_nothing_has_no_card_to_put_anywhere() {
    cr!("701.23b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Swamp", 1);
    let bears = t.library_top(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Dina's Guidance");
    t.answer_choose(P0, &[]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(!t
        .asked()
        .into_iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
    assert_eq!(t.zone(bears), Zone::Library(P0));
    assert_eq!(shuffles(&t), 1);
}
