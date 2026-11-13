//! "If a triggered ability of [objects] triggers, that ability triggers an additional
//! time." (`src/oracle/patterns/triggers_additional_time.rs`, CR 603.2d).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn only_the_equipped_creatures_abilities_trigger_an_additional_time() {
    cr!("603.2d");
    // Wizard's Staff: "Equipped creature has prowess. If a triggered ability of equipped
    // creature triggers, that ability triggers an additional time." "Equipped creature"
    // is the creature it's attached to, not any creature with an Equipment.
    let u = card("Wizard's Staff").unsupported_text().join(" | ");
    assert!(u.is_empty(), "Wizard's Staff has unsupported text: {u}");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let staff = t.battlefield(P0, "Wizard's Staff");
    assert!(t.g.attach(staff, Entity::Object(bears)));
    // Monastery Swiftspear (1/2, prowess) carries another Equipment.
    let spear = t.battlefield(P0, "Monastery Swiftspear");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(splitter, Entity::Object(spear)));
    t.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(spear), (3, 2));
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    // The Bears' prowess triggers twice; the Swiftspear's once.
    assert_eq!(t.stack_len(), 4);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(spear), (4, 3));
}
