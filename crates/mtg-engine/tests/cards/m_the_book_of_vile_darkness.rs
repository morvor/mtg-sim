//! The Book of Vile Darkness (hand-written, `src/cards/the_book_of_vile_darkness.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn vecna_gets_the_triggered_abilities_of_the_exiled_cards() {
    cr!("111.1", "603.6a");
    ruling!("The Book of Vile Darkness", "will gain the triggered ability printed on Hand of Vecna and the two triggered abilities");
    ruling!("The Book of Vile Darkness", "any enters-the-battlefield triggered abilities it gains");
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "The Book of Vile Darkness");
    t.battlefield(P0, "Eye of Vecna");
    t.battlefield(P0, "Hand of Vecna");
    t.library_top(P0, "Island");
    t.activate(P0, book, 0, &[]).unwrap();
    t.resolve_all();
    let vecna = t.named_on_battlefield("Vecna")[0];
    assert_eq!(t.pt(vecna), (8, 8));
    assert!(t.in_exile("Eye of Vecna") && t.in_exile("Hand of Vecna"));
    assert!(t.in_exile("The Book of Vile Darkness"));
    let triggered = t
        .obj(vecna)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, ability::AbilityKind::Triggered(_)))
        .count();
    // The Book's, the Eye's two and the Hand's.
    assert_eq!(triggered, 4);
    // The Eye's enters trigger: draw a card and lose 2 life.
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn needs_both_artifacts() {
    cr!("602.2b");
    let mut t = TestGame::new(2);
    let book = t.battlefield(P0, "The Book of Vile Darkness");
    t.battlefield(P0, "Eye of Vecna");
    assert!(t.activate(P0, book, 0, &[]).is_err());
}
