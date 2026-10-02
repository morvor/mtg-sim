//! Dina's Guidance (hand-written, `src/cards/dinas_guidance.rs`).

use mtg_engine::decision::Answer;
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
    }
}
