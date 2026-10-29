//! Booby Trap (hand-written, `src/cards/booby_trap.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn trap(t: &mut TestGame) -> ObjectId {
    let trap = t.battlefield(P0, "Booby Trap");
    t.g.objects[trap.0 as usize].choices.player = Some(P1);
    t.g.objects[trap.0 as usize].choices.card_name = Some("Lightning Bolt".into());
    t.g.recompute();
    trap
}

#[test]
fn drawing_the_chosen_card_sets_it_off() {
    cr!("603.2");
    ruling!("Booby Trap", "it has to be on the battlefield when the trigger resolves");
    let mut t = TestGame::new(2);
    let trap = trap(&mut t);
    t.library_top(P1, "Lightning Bolt");
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert!(!t.on_battlefield(trap));
    assert_eq!(t.life(P1), 10);
}

#[test]
fn other_cards_and_players_dont() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    let trap = trap(&mut t);
    t.library_top(P1, "Shock");
    t.g.draw_cards(P1, 1);
    t.library_top(P0, "Lightning Bolt");
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.on_battlefield(trap));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn the_chosen_player_reveals_each_card_drawn() {
    cr!("701.20a");
    let mut t = TestGame::new(2);
    trap(&mut t);
    t.library_top(P1, "Shock");
    t.g.draw_cards(P1, 1);
    assert!(t.dump_log().contains("reveals Shock"));
    // The other player doesn't.
    t.library_top(P0, "Shock");
    t.g.draw_cards(P0, 1);
    assert_eq!(t.dump_log().matches("reveals Shock").count(), 1);
}
