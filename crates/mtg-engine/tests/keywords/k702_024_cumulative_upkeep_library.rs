//! CR 702.24a: a cumulative upkeep cost that moves cards from a library is paid for every
//! age counter at once, or not at all.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to P0's next upkeep and puts the upkeep triggers on the stack.
fn p0_upkeep(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
}

#[test]
fn exiling_more_cards_than_the_library_has_cant_be_paid() {
    cr!("702.24a", "118.3");
    ruling!(
        "Arctic Wolves",
        "Partial payments of the total cumulative upkeep cost can’t be made"
    );
    ruling!(
        "Thought Lash",
        "If you don’t pay Thought Lash’s cumulative upkeep, you’ll sacrifice Thought Lash as normal"
    );
    let mut t = TestGame::new(2);
    // "Cumulative upkeep—Exile the top card of your library."
    let lash = t.battlefield(P0, "Thought Lash");
    t.g.add_counters(Entity::Object(lash), "age", 2, None);
    p0_upkeep(&mut t);
    // With a third age counter the cost is exiling the top three cards; two are left.
    let n = t.library_size(P0);
    t.g.mill(P0, n as u32 - 2);
    assert_eq!(t.library_size(P0), 2);
    let exiled = t.g.exile.len();
    t.answer_yes(P0, true);
    t.resolve();
    // The cost can't be paid: no card was exiled for it, and Thought Lash is sacrificed.
    assert!(!t.on_battlefield(lash));
    assert!(t.in_graveyard(P0, "Thought Lash"));
    assert_eq!(t.g.exile.len(), exiled);
    assert_eq!(t.library_size(P0), 2);
    // "When a player doesn't pay ~'s cumulative upkeep, that player exiles all cards from
    // their library."
    t.settle();
    t.resolve();
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.g.exile.len(), exiled + 2);
}

#[test]
fn exiling_as_many_cards_as_the_library_has_is_paid() {
    cr!("702.24a");
    let mut t = TestGame::new(2);
    let lash = t.battlefield(P0, "Thought Lash");
    t.g.add_counters(Entity::Object(lash), "age", 1, None);
    p0_upkeep(&mut t);
    let n = t.library_size(P0);
    t.g.mill(P0, n as u32 - 2);
    let exiled = t.g.exile.len();
    t.answer_yes(P0, true);
    t.resolve();
    // Two age counters, two cards: both are exiled and Thought Lash stays.
    assert!(t.on_battlefield(lash));
    assert_eq!(t.library_size(P0), 0);
    assert_eq!(t.g.exile.len(), exiled + 2);
    t.settle();
    assert!(t.g.stack.is_empty());
}
