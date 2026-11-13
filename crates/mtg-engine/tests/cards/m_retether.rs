//! Retether (hand-written, `src/cards/retether.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn auras_return_attached_to_creatures() {
    cr!("303.4f", "303.4g");
    ruling!("Retether", "All the Auras return to the battlefield simultaneously");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Holy Strength");
    // Wild Growth has enchant land: it can't enchant a creature, so it stays.
    t.graveyard(P0, "Wild Growth");
    t.battlefield(P0, "Forest");
    t.lands(P0, "Plains", 4);
    let s = t.hand(P0, "Retether");
    t.cast(P0, s).go();
    t.resolve();
    let hs = t.named_on_battlefield("Holy Strength");
    assert_eq!(hs.len(), 1);
    assert_eq!(t.g.obj(hs[0]).attached_to, Some(Entity::Object(bear)));
    assert!(t.in_graveyard(P0, "Wild Growth"));
}
