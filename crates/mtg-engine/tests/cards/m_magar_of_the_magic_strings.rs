//! Magar of the Magic Strings (hand-written, `src/cards/magar_of_the_magic_strings.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn setup(t: &mut TestGame) -> ObjectId {
    let magar = t.battlefield(P0, "Magar of the Magic Strings");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.activate(P0, magar, 0, &[Entity::Object(bolt)]).unwrap();
    t.resolve_all();
    t.g.current(bolt)
}

#[test]
fn the_card_becomes_a_face_down_3_3_that_casts_copies_of_itself() {
    cr!("707.14", "708.2");
    ruling!("Magar of the Magic Strings", "It has no name, no mana cost, no creature types");
    let mut t = TestGame::new(2);
    let fd = setup(&mut t);
    assert!(t.on_battlefield(fd) && t.obj(fd).face_down);
    assert_eq!(t.pt(fd), (3, 3));
    assert!(t.obj(fd).chars.name.is_empty());
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    // Create the copy, cast it, and aim it at P1.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.attack(&[(fd, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20 - 3 - 3);
    // The card itself is still on the battlefield face down.
    assert!(t.on_battlefield(fd));
}

#[test]
fn leaving_the_battlefield_exiles_it() {
    cr!("614.1a");
    ruling!("Magar of the Magic Strings", "the card is exiled and remains in exile");
    let mut t = TestGame::new(2);
    let fd = setup(&mut t);
    t.g.destroy(fd, None);
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
}
