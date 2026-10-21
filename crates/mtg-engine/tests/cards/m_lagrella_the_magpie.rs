//! Lagrella, the Magpie (hand-written, `src/cards/lagrella_the_magpie.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn returning_cards_under_your_control_get_two_counters() {
    cr!("603.7", "610.3");
    ruling!("Lagrella, the Magpie", "\"This way\" refers to cards entering the battlefield as a result of Lagrella leaving the battlefield");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(mine), Entity::Object(theirs)]);
    let lag = t.enter(P0, "Lagrella, the Magpie");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_exile("Hill Giant"));
    t.g.destroy(lag, None);
    t.resolve_all();
    let bear = t.named_on_battlefield("Grizzly Bears")[0];
    let giant = t.named_on_battlefield("Hill Giant")[0];
    assert_eq!(t.counters(bear, "+1/+1"), 2);
    // The Giant returned under its owner's (an opponent's) control.
    assert_eq!(t.counters(giant, "+1/+1"), 0);
}

#[test]
fn counters_also_when_lagrella_leaves_on_a_later_turn() {
    cr!("603.7");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(mine)]);
    let lag = t.enter(P0, "Lagrella, the Magpie");
    t.resolve_all();
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.g.destroy(lag, None);
    t.resolve_all();
    let bear = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(bear, "+1/+1"), 2);
}
