//! Sight Beyond Sight ("Look at the top two cards of your library. Put one of them into
//! your hand and the other on the bottom of your library.") while its controller may look
//! at the top card of their library (Skill Borrower). Taking the top card uncovers the
//! other one, which the top-card effect sees again and makes a new object (CR 401.6); it
//! must still be the card put on the bottom, and the library must list only live objects.
//! (Found by the every-card sweep: Maelstrom Nexus game 0, `--from 15350`.)

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_card_left_over_goes_to_the_bottom_even_after_becoming_a_new_object() {
    cr!("401.6");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    t.battlefield(P0, "Skill Borrower");
    let other = t.library_top(P0, "Sight Beyond Sight");
    let forest = t.library_top(P0, "Forest");
    let spell = t.hand(P0, "Sight Beyond Sight");
    let size = t.library_size(P0);
    t.cast(P0, spell).go();
    t.resolve();
    // The Forest (the top card) went to the hand, the other card to the bottom.
    assert_eq!(t.zone(t.g.current(forest)), Zone::Hand(P0));
    assert_eq!(t.library_size(P0), size - 1);
    let lib = t.g.player(P0).library.clone();
    for id in &lib {
        assert!(t.g.is_live(*id), "{id:?} is listed but isn't a live object");
        assert_eq!(t.g.obj(*id).zone, Zone::Library(P0));
    }
    let bottom = lib[0];
    assert_eq!(bottom, t.g.current(other));
    assert_eq!(t.g.obj(bottom).chars.name, "Sight Beyond Sight");
    assert_ne!(
        t.g.obj(*lib.last().unwrap()).chars.name,
        "Sight Beyond Sight",
        "the card left over isn't left on top"
    );
}
