//! Don't Blink (hand-written, `src/cards/dont_blink.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn creatures_entering_from_exile_are_shuffled_into_their_libraries() {
    cr!("614.1c");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Don't Blink");
    t.cast(P0, s).go();
    t.resolve();
    let lib = t.library_size(P1);
    t.lands(P0, "Plains", 1);
    let cs = t.hand(P0, "Cloudshift");
    // Cloudshift targets a creature you control: flicker P0's own.
    let mine = t.battlefield(P0, "Hill Giant");
    t.cast(P0, cs).target(mine).go();
    t.resolve();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert_eq!(t.library_size(P0), 31);
    assert_eq!(t.library_size(P1), lib);
    assert!(t.on_battlefield(bear));
}

#[test]
fn creature_spells_cast_from_exile_too() {
    cr!("614.1c");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Don't Blink");
    t.cast(P0, s).go();
    t.resolve();
    // A creature cast normally enters.
    t.lands(P0, "Forest", 2);
    let b = t.hand(P0, "Grizzly Bears");
    t.cast(P0, b).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_creature_spell_cast_from_exile_is_shuffled_in() {
    cr!("614.1c", "608.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let s = t.hand(P0, "Don't Blink");
    t.cast(P0, s).go();
    t.resolve();
    // Misthollow Griffin: "You may cast this card from exile."
    t.lands(P0, "Island", 4);
    let g = t.exile(P0, "Misthollow Griffin");
    t.cast(P0, g).go();
    t.resolve();
    assert!(t.named_on_battlefield("Misthollow Griffin").is_empty());
    assert_eq!(t.library_size(P0), 31);
}
