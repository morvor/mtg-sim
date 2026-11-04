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
