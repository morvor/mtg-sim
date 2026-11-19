//! Rulings batch S33 — "each player may search their library ...": the player whose turn
//! it is searches (or chooses not to), then each other player in turn order; then those
//! who searched shuffle, in the same order (CR 101.4, 701.23i).

use crate::r_s01_common::{supported, watch, Seen};
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
            Event::Searched { player, .. } => Some(format!("search {player}")),
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

#[test]
fn veteran_explorer_no_one_searches_if_every_player_declines() {
    cr!("701.23a", "701.24a");
    // Every player declines: no library is searched or shuffled, and no land enters.
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        t.library_top(p, "Forest");
    }
    let explorer = t.battlefield(P0, "Veteran Explorer");
    t.answer_yes(P0, false);
    t.answer_yes(P1, false);
    let from = t.g.turn_events.len();
    destroy(&mut t, explorer);
    t.resolve_all();
    assert!(searches_and_shuffles(&t, from).is_empty());
    assert_eq!((basic_lands(&t, P0), basic_lands(&t, P1)), (0, 0));
}

#[test]
fn veteran_explorer_a_player_who_cant_search_cant_choose_to() {
    cr!("101.2", "701.23a");
    // P0's Stranglehold: "Your opponents can't search libraries." P1 isn't offered the
    // search (and doesn't shuffle); P0 searches as usual.
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        t.library_top(p, "Forest");
    }
    t.battlefield(P0, "Stranglehold");
    let explorer = t.battlefield(P0, "Veteran Explorer");
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    let from = t.g.turn_events.len();
    let asked = t.asked().len();
    destroy(&mut t, explorer);
    t.resolve_all();
    assert_eq!(
        searches_and_shuffles(&t, from),
        vec!["search P0", "shuffle P0"]
    );
    assert_eq!((basic_lands(&t, P0), basic_lands(&t, P1)), (1, 0));
    let askers: Vec<PlayerId> = t.asked()[asked..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(askers, vec![P0]);
}

/// The "takes part" log lines when a player decides (the decisions recorded so far).
fn takers(g: &mtg_engine::game::Game) -> Vec<String> {
    g.log
        .iter()
        .filter(|l| l.text.ends_with("chooses to take part"))
        .map(|l| l.text.clone())
        .collect()
}

fn is_yes_no(d: &mtg_engine::decision::Decision) -> bool {
    matches!(d, mtg_engine::decision::Decision::YesNo { .. })
}

#[test]
fn veteran_explorer_each_player_knows_the_earlier_decisions_to_search() {
    cr!("101.4", "101.4b");
    ruling!(
        "Veteran Explorer",
        "The player whose turn it is searches their library (or chooses not to), then each other player in turn order does the same."
    );
    supported("Veteran Explorer");
    // P1's turn: P1, P2, P0 decide, each knowing the decisions before theirs.
    let mut t = TestGame::new(3);
    for p in [P0, P1, P2] {
        t.library_top(p, "Forest");
    }
    t.set_step(P1, Step::PrecombatMain);
    let explorer = t.battlefield(P0, "Veteran Explorer");
    t.answer_yes(P1, true);
    t.answer_yes(P2, false);
    t.answer_yes(P0, true);
    let seen: Vec<Seen<Vec<String>>> = [P1, P2, P0]
        .iter()
        .map(|p| watch(&mut t, *p, is_yes_no, takers))
        .collect();
    destroy(&mut t, explorer);
    t.resolve_all();
    let p1 = vec!["P1 chooses to take part".to_string()];
    assert_eq!(*seen[0].lock().unwrap(), vec![Vec::<String>::new()]);
    assert_eq!(*seen[1].lock().unwrap(), vec![p1.clone()]);
    assert_eq!(*seen[2].lock().unwrap(), vec![p1]);
    assert_eq!(
        (basic_lands(&t, P0), basic_lands(&t, P1), basic_lands(&t, P2)),
        (1, 1, 0)
    );
}

#[test]
fn borderland_explorer_players_discard_then_search_together() {
    cr!("101.4", "101.4a", "101.4b", "701.23i");
    ruling!(
        "Borderland Explorer",
        "Each player will know if previous players are discarding, but not what they are discarding."
    );
    ruling!(
        "Borderland Explorer",
        "All cards to be discarded are set aside, then revealed and discarded simultaneously. Similarly, all basic land cards searched for are set aside, then revealed simultaneously."
    );
    supported("Borderland Explorer");
    // "When this creature enters, each player may discard a card. Each player who
    // discarded a card this way may search their library for a basic land card, reveal it,
    // put it into their hand, then shuffle." Both players discard and search.
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let plains = t.library_top(P1, "Plains");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P1, "Hill Giant");
    for p in [P0, P1] {
        t.answer_yes(p, true);
        t.answer_yes(p, true);
    }
    t.answer_choose(P0, &[Entity::Object(a)]);
    t.answer_choose(P1, &[Entity::Object(b)]);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.answer_choose(P1, &[Entity::Object(plains)]);
    // At each of P1's choices: whether P0 is taking part, what P1 knows of P0's choice,
    // and where P0's Bears and Forest are.
    let seen = watch(&mut t, P1, |_| true, |g| {
        (
            takers(g),
            g.known_apnap_choices(PlayerId(1)),
            g.player(PlayerId(0)).hand.len(),
            g.player(PlayerId(0)).graveyard.len(),
        )
    });
    let hand0 = t.hand_size(P0);
    let from = t.g.turn_events.len();
    t.enter(P0, "Borderland Explorer");
    t.resolve_all();
    assert_eq!(
        searches_and_shuffles(&t, from),
        vec!["search P0", "search P1", "shuffle P0", "shuffle P1"]
    );
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_hand(P0, "Forest") && t.in_hand(P1, "Plains"));
    let seen = seen.lock().unwrap();
    // P1's decision to discard: P0 has decided to take part.
    assert_eq!(seen[0].0, vec!["P0 chooses to take part".to_string()]);
    // P1's discard: P0's chosen card is known to be chosen, but stays face down (101.4a);
    // it hasn't been discarded yet.
    let discard = seen
        .iter()
        .find(|s| !s.1.is_empty())
        .expect("P1 chooses a card to discard");
    assert_eq!(discard.1, vec![(P0, None)]);
    assert_eq!((discard.2, discard.3), (hand0, 0));
}

#[test]
fn old_growth_dryads_opponents_search_then_shuffle_in_turn_order() {
    cr!("101.4", "608.2f", "701.23i", "701.24a");
    supported("Old-Growth Dryads");
    // "When this creature enters, each opponent may search their library for a basic land
    // card, put it onto the battlefield tapped, then shuffle." Both opponents search, then
    // both shuffle.
    let mut t = TestGame::new(3);
    t.library_top(P1, "Forest");
    t.library_top(P2, "Forest");
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    let from = t.g.turn_events.len();
    t.enter(P0, "Old-Growth Dryads");
    t.resolve_all();
    assert_eq!(
        searches_and_shuffles(&t, from),
        vec!["search P1", "search P2", "shuffle P1", "shuffle P2"]
    );
    assert_eq!((basic_lands(&t, P1), basic_lands(&t, P2)), (1, 1));
}

#[test]
fn old_growth_dryads_an_opponent_who_cant_search_isnt_offered_it_and_doesnt_shuffle() {
    cr!("101.2", "608.2d");
    ruling!(
        "Stranglehold",
        "your opponents can’t choose to search, so they won’t shuffle."
    );
    supported("Old-Growth Dryads");
    supported("Stranglehold");
    // P2's Stranglehold: "Your opponents can't search libraries." When P0's Old-Growth
    // Dryads enters, P1 (P2's opponent) isn't offered the search ("may search ... then
    // shuffle") and doesn't shuffle; P2 searches as usual.
    let mut t = TestGame::new(3);
    t.battlefield(P2, "Stranglehold");
    for p in [P1, P2] {
        t.library_top(p, "Forest");
        t.answer_yes(p, true);
    }
    let from = t.g.turn_events.len();
    let asked = t.asked().len();
    t.enter(P0, "Old-Growth Dryads");
    t.resolve_all();
    assert_eq!(
        searches_and_shuffles(&t, from),
        vec!["search P2", "shuffle P2"]
    );
    assert_eq!((basic_lands(&t, P1), basic_lands(&t, P2)), (0, 1));
    let askers: Vec<PlayerId> = t.asked()[asked..]
        .iter()
        .filter(|(_, d)| is_yes_no(d))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(askers, vec![P2]);
}
