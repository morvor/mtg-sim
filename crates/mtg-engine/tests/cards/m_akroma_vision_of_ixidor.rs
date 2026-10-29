//! Akroma, Vision of Ixidor (hand-written, `src/cards/akroma_vision_of_ixidor.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn each_listed_keyword_gives_plus_one() {
    cr!("608.2h", "611.2c");
    ruling!("Akroma, Vision of Ixidor", "A creature with more than one of the listed keyword gets +1/+1 for each");
    let mut t = TestGame::new(2);
    let akroma = t.battlefield(P0, "Akroma, Vision of Ixidor");
    // Flying and vigilance.
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Serra Angel");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(angel), (6, 6));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(theirs), (4, 4));
    assert_eq!(t.pt(akroma), (6, 6));
}

#[test]
fn bonus_is_locked_in_on_resolution() {
    cr!("608.2h");
    ruling!("Akroma, Vision of Ixidor", "Gaining or losing keywords after that time won't cause a creature to grow or shrink");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akroma, Vision of Ixidor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    // Gaining flying afterwards doesn't add to it.
    t.lands(P0, "Island", 1);
    let wings = t.hand(P0, "Jump");
    t.cast(P0, wings).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (2, 2));
}
