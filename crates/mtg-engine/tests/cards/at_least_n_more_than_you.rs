//! "a player who controls at least N more [objects] than you" (the player-clause grammar
//! in `oracle/patterns/value_grammar.rs`): Avatar of Might's cost reduction.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn an_opponent_with_at_least_four_more_creatures_reduces_the_cost() {
    cr!("601.2f");
    assert!(mtg_engine::card("Avatar of Might").unsupported_text().is_empty());
    for (bears, castable) in [(4, true), (3, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 2);
        for _ in 0..bears {
            t.battlefield(P1, "Grizzly Bears");
        }
        let avatar = t.hand(P0, "Avatar of Might");
        assert_eq!(t.cast(P0, avatar).try_go().is_ok(), castable, "{bears} bears");
    }
    // Four more than you: one creature of yours needs five.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.battlefield(P0, "Grizzly Bears");
    for _ in 0..4 {
        t.battlefield(P1, "Grizzly Bears");
    }
    let avatar = t.hand(P0, "Avatar of Might");
    assert!(t.cast(P0, avatar).try_go().is_err());
}
