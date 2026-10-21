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
