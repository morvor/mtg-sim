//! River Song (hand-written, `src/cards/river_song.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Puts a card named `name` on the bottom of `p`'s library.
fn library_bottom(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.library_top(p, name);
    let lib = &mut t.g.players[p.idx()].library;
    lib.retain(|x| *x != id);
    lib.insert(0, id);
    id
}

#[test]
fn you_draw_from_the_bottom_of_your_library() {
    cr!("121.1");
    ruling!("River Song", "applies to all cards you draw while it is on the battlefield");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "River Song");
    library_bottom(&mut t, P0, "Shock");
    t.library_top(P0, "Island");
    let drawn = t.g.draw_cards(P0, 1);
    assert_eq!(t.obj_now(drawn[0]).chars.name, "Shock");
    // The draw for the turn too.
    library_bottom(&mut t, P0, "Lightning Bolt");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(!t.in_hand(P0, "Island"));
}

#[test]
fn opponents_still_draw_from_the_top() {
    cr!("121.1");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "River Song");
    library_bottom(&mut t, P1, "Shock");
    t.library_top(P1, "Island");
    let drawn = t.g.draw_cards(P1, 1);
    assert_eq!(t.obj_now(drawn[0]).chars.name, "Island");
}

#[test]
fn a_spells_draws_take_the_bottom_cards() {
    cr!("121.2");
    ruling!(
        "River Song",
        "applies to all cards you draw while it is on the battlefield"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "River Song");
    t.set_step(P0, Step::PrecombatMain);
    library_bottom(&mut t, P0, "Shock");
    library_bottom(&mut t, P0, "Giant Growth");
    t.library_top(P0, "Island");
    t.lands(P0, "Island", 3);
    // "Draw two cards."
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.resolve();
    assert!(t.in_hand(P0, "Shock"));
    assert!(t.in_hand(P0, "Giant Growth"));
    assert!(!t.in_hand(P0, "Island"));
}

#[test]
fn drawing_from_the_bottom_is_still_drawing() {
    cr!("121.1");
    ruling!(
        "River Song",
        "Drawing cards from the bottom of your library is exactly the same as drawing from the top"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "River Song");
    // "Whenever an opponent draws a card, Underworld Dreams deals 1 damage to that player."
    t.battlefield(P1, "Underworld Dreams");
    library_bottom(&mut t, P0, "Shock");
    t.library_top(P0, "Island");
    let before = t.g.history.cards_drawn.get(&P0).copied().unwrap_or(0);
    let drawn = t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.obj_now(drawn[0]).chars.name, "Shock");
    assert_eq!(t.g.history.cards_drawn.get(&P0).copied(), Some(before + 1));
    assert_eq!(t.life(P0), 19);
}
