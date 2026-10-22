//! Rulings on time travel (CR 701.56): only time counters are added or removed.

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn time_travel_doesnt_move_a_sagas_lore_counters() {
    cr!("701.56a", "714.3");
    ruling!(
        "Wibbly-wobbly, Timey-wimey",
        "Time counters are usually found on cards with suspend and vanishing, but may be found on other cards as well. Notably, Sagas use lore counters to track their progress, not time counters. You can't move a Saga's chapters forward and backward this way."
    );
    supported("Wibbly-wobbly, Timey-wimey");
    supported("The Girl in the Fireplace");
    let mut t = TestGame::new(2);
    // A Saga with one lore counter; its chapter I created a Human Noble token with
    // vanishing 3 (three time counters). And another permanent with time counters.
    let saga = t.enter(P0, "The Girl in the Fireplace");
    t.resolve_all();
    assert_eq!(t.counters(saga, counters::LORE), 1);
    let noble = with_subtype(&t, P0, "Noble")[0];
    assert_eq!(t.counters(noble, counters::TIME), 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::TIME, 2, None);
    t.g.recompute();
    // Wibbly-wobbly, Timey-wimey: "Time travel. Draw a card." P0 adds a time counter to
    // everything it's offered.
    for _ in 0..3 {
        t.answer(P0, DecisionKind::Option, Answer::Index(0));
    }
    let from = t.asked().len();
    t.lands(P0, "Island", 2);
    let spell = t.hand(P0, "Wibbly-wobbly, Timey-wimey");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::TIME), 3);
    assert_eq!(t.counters(noble, counters::TIME), 4);
    // Only those two were offered: the Saga has no time counter, and it's still on its
    // first chapter.
    let offered = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseOption { prompt, .. } if prompt.starts_with("Time travel")))
        .count();
    assert_eq!(offered, 2);
    assert_eq!(t.counters(saga, counters::LORE), 1);
    assert_eq!(t.counters(saga, counters::TIME), 0);
    assert!(with_subtype(&t, P0, "Horse").is_empty());
}
