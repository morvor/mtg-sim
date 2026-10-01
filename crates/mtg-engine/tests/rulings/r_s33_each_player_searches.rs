//! Rulings batch S33 — "each player may search their library ...": the player whose turn
//! it is searches (or chooses not to), then each other player in turn order; then those
//! who searched shuffle, in the same order (CR 101.4, 701.23i).

use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// The players' searches and shuffles since index `from` of this turn's events, in order
/// ("search P1", "shuffle P0", ...).
fn searches_and_shuffles(t: &TestGame, from: usize) -> Vec<String> {
    t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Searched { player } => Some(format!("search {player}")),
            Event::Shuffled { player } => Some(format!("shuffle {player}")),
            _ => None,
        })
        .collect()
}

/// The basic lands `p` controls on the battlefield.
fn basic_lands(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.is(CardType::Land))
        .count()
}

#[test]
fn veteran_explorer_players_search_in_turn_order_then_shuffle() {
    cr!("101.4", "701.23i", "701.24a");
    ruling!(
        "Veteran Explorer",
        "The player whose turn it is searches their library (or chooses not to), then each other player in turn order does the same. Then the player whose turn it is shuffles their library (if that player searched), then each other player in turn order does the same."
    );
    supported("Veteran Explorer");
    // "When this creature dies, each player may search their library for up to two basic
    // land cards, put them onto the battlefield, then shuffle." It's P1's turn (of three):
    // P1 and P0 search, P2 doesn't.
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        t.library_top(p, "Forest");
        t.library_top(p, "Island");
    }
    t.set_step(P1, Step::PrecombatMain);
    let explorer = t.battlefield(P0, "Veteran Explorer");
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    t.answer_yes(P0, true);
    let from = t.g.turn_events.len();
    destroy(&mut t, explorer);
    t.resolve_all();
    assert_eq!(
        searches_and_shuffles(&t, from),
        vec!["search P1", "search P0", "shuffle P1", "shuffle P0"]
    );
    assert_eq!(
        (basic_lands(&t, P0), basic_lands(&t, P1), basic_lands(&t, P2)),
        (2, 2, 0)
    );
    // The choices were asked in turn order: P1, P2, P0.
    let askers: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
        .map(|(p, _)| p)
        .collect();
    assert_eq!(askers, vec![P1, P2, P0]);
}

#[test]
fn noble_benefactor_players_search_in_turn_order_then_shuffle() {
    cr!("101.4", "701.23i");
    ruling!(
        "Noble Benefactor",
        "The player whose turn it is searches their library (or chooses not to), then each other player in turn order does the same. Then the player whose turn it is shuffles their library (if that player searched), then each other player in turn order does the same."
    );
    supported("Noble Benefactor");
    // "When this creature dies, each player may search their library for a card and put
    // that card into their hand. Then each player who searched their library this way
    // shuffles." P0's turn, two players: P0 declines, P1 searches.
    let mut t = TestGame::new(2);
    let benefactor = t.battlefield(P0, "Noble Benefactor");
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    let hands = (t.hand_size(P0), t.hand_size(P1));
    let from = t.g.turn_events.len();
    destroy(&mut t, benefactor);
    t.resolve_all();
    assert_eq!(
        searches_and_shuffles(&t, from),
        vec!["search P1", "shuffle P1"]
    );
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (hands.0, hands.1 + 1));
}
