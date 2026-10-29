//! Bludgeon Brawl (hand-written, `src/cards/bludgeon_brawl.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn noncreature_artifacts_are_equipment_with_their_mana_value() {
    cr!("301.5", "702.6a");
    ruling!("Bludgeon Brawl", "The affected artifacts retain their other types and abilities");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bludgeon Brawl");
    // Mind Stone has mana value 2.
    let stone = t.battlefield(P0, "Mind Stone");
    let bear = t.battlefield(P0, "Grizzly Bears");
    assert!(t.obj_now(stone).chars.has_subtype("Equipment"));
    t.lands(P0, "Plains", 2);
    let equip = t
        .g
        .obj(stone)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.starts_with("Equip"))
        .expect("an equip ability");
    t.activate(P0, stone, equip, &[Entity::Object(bear)]).unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(stone).attached_to, Some(Entity::Object(bear)));
    assert_eq!(t.pt(bear), (4, 2));
    // It still has its mana ability.
    assert!(t
        .g
        .obj(stone)
        .chars
        .abilities
        .iter()
        .any(|a| a.text.contains("Add {C}")));
}

#[test]
fn artifact_creatures_and_equipment_are_unaffected() {
    cr!("301.5");
    ruling!("Bludgeon Brawl", "An Equipment that’s also a creature can’t be equipped to anything");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bludgeon Brawl");
    let thopter = t.battlefield(P0, "Ornithopter");
    assert!(!t.obj_now(thopter).chars.has_subtype("Equipment"));
}
