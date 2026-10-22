//! Keen-Eared Sentry (hand-written, `src/cards/keen_eared_sentry.rs`).

use mtg_engine::dungeons;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The room `p`'s venture marker is in.
fn room(t: &TestGame, p: PlayerId) -> Option<usize> {
    dungeons::marker(&t.g, p).map(|(_, r)| r)
}

#[test]
fn an_opponent_ventures_only_once_each_turn() {
    cr!("701.49a", "101.2");
    interp!("venture-limit-looks-at-whole-turn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keen-Eared Sentry");
    t.enter(P1, "Veteran Dungeoneer");
    t.resolve_all();
    assert_eq!(room(&t, P1), Some(0));
    // A second venture this turn does nothing.
    t.enter(P1, "Veteran Dungeoneer");
    t.resolve_all();
    assert_eq!(room(&t, P1), Some(0));
}

#[test]
fn its_controller_ventures_freely() {
    cr!("701.49b");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Keen-Eared Sentry");
    t.enter(P0, "Veteran Dungeoneer");
    t.resolve_all();
    assert_eq!(room(&t, P0), Some(0));
    t.enter(P0, "Veteran Dungeoneer");
    t.resolve_all();
    assert!(room(&t, P0).is_some_and(|r| r > 0));
}

#[test]
fn ventures_before_it_entered_count_and_without_it_there_is_no_limit() {
    cr!("701.49b");
    interp!("venture-limit-looks-at-whole-turn");
    let mut t = TestGame::new(2);
    t.enter(P1, "Veteran Dungeoneer");
    t.resolve_all();
    t.enter(P1, "Veteran Dungeoneer");
    t.resolve_all();
    let r = room(&t, P1).expect("in a dungeon");
    assert!(r > 0);
    // P1 ventured this turn before the Sentry arrived: no more ventures this turn.
    t.battlefield(P0, "Keen-Eared Sentry");
    t.enter(P1, "Veteran Dungeoneer");
    t.resolve_all();
    assert_eq!(room(&t, P1), Some(r));
}
