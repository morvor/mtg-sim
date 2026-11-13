//! Aeon Engine (hand-written, `src/cards/aeon_engine.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn reverse(t: &mut TestGame) {
    let engine = t.battlefield(P0, "Aeon Engine");
    t.g.untap(engine);
    t.activate(P0, engine, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Aeon Engine"));
}

#[test]
fn reversing_turn_order_changes_the_next_player_and_apnap_order() {
    cr!("101.4", "500.1");
    ruling!("Aeon Engine", "anything that cares about this order (such as determining in what order choices are made) will use the new order");
    let mut t = TestGame::new(4);
    assert_eq!(t.g.next_player(P0), P1);
    assert_eq!(t.g.apnap(), vec![P0, P1, P2, P3]);
    reverse(&mut t);
    // Play now proceeds to the right: P3 is next, then P2, then P1.
    assert_eq!(t.g.next_player(P0), P3);
    assert_eq!(t.g.apnap(), vec![P0, P3, P2, P1]);
    // Seats don't move: the player to P0's left is still P1.
    assert_eq!(mtg_engine::seats::neighbor(&t.g, P0, true), Some(P1));
    // The next turn is P3's, then P2's.
    t.advance_to(P3, Step::Upkeep);
    assert_eq!(t.g.turn.number, 2);
    assert_eq!(t.g.apnap(), vec![P3, P2, P1, P0]);
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(t.g.turn.number, 3);
}

#[test]
fn reversing_it_twice_restores_the_original_order() {
    cr!("101.4");
    ruling!("Aeon Engine", "it will return to the default order used at the beginning of the game");
    let mut t = TestGame::new(3);
    reverse(&mut t);
    assert_eq!(t.g.next_player(P0), P2);
    reverse(&mut t);
    assert_eq!(t.g.next_player(P0), P1);
    assert_eq!(t.g.apnap(), vec![P0, P1, P2]);
}

#[test]
fn it_enters_tapped_and_exiling_it_is_part_of_the_cost() {
    cr!("602.2");
    let mut t = TestGame::new(3);
    let engine = t.hand(P0, "Aeon Engine");
    t.lands(P0, "Wastes", 5);
    t.cast(P0, engine).go();
    t.resolve_all();
    let engine = t.named_on_battlefield("Aeon Engine")[0];
    assert!(t.obj_now(engine).tapped);
    assert!(t.activate(P0, engine, 0, &[]).is_err());
    assert!(!t.g.turn_order_reversed);
}

#[test]
fn with_two_players_it_has_no_noticeable_effect() {
    cr!("101.4");
    ruling!("Aeon Engine", "it has no noticeable effect on the game");
    let mut t = TestGame::new(2);
    reverse(&mut t);
    assert!(t.g.turn_order_reversed);
    assert_eq!(t.g.next_player(P0), P1);
    assert_eq!(t.g.apnap(), vec![P0, P1]);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.turn.number, 2);
}

#[test]
fn reversed_during_an_extra_turn_play_resumes_from_the_last_normal_turn() {
    cr!("101.4", "500.7");
    ruling!("Aeon Engine", "the next non-extra turn will be taken by the next player in turn order from the last player who took a non-extra turn");
    let mut t = TestGame::new(4);
    // P1 takes an extra turn after P0's turn 1.
    t.g.extra_turns.push(P1);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.turn.extra);
    t.advance_to_step(Step::PrecombatMain);
    reverse(&mut t);
    // The next normal turn follows P0's (the last normal one) to the right: P3's.
    t.advance_to(P3, Step::Upkeep);
    assert_eq!(t.g.turn.number, 3);
    assert!(!t.g.turn.extra);
}
