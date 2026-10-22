//! Rulings batch S31 — an Equipment that becomes a creature (CR 301.5c, 704.5n).

use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn an_equipment_that_becomes_a_creature_becomes_unattached() {
    cr!("301.5c", "704.5n", "205.1b");
    ruling!(
        "Ensoul Artifact",
        "The artifact retains any types, subtypes, or supertypes it has. Notably, if an Equipment becomes an artifact creature, it usually can't be attached to another creature. If it was attached to a creature, it becomes unattached."
    );
    supported("Ensoul Artifact");
    supported("Bonesplitter");
    // Ensoul Artifact: "Enchanted artifact is a creature with base power and toughness 5/5
    // in addition to its other types."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", bears);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    attach_new(&mut t, P0, "Ensoul Artifact", splitter);
    t.settle();
    t.g.recompute();
    // A 5/5 artifact creature that's still an Equipment, no longer attached.
    let s = t.obj_now(splitter);
    assert!(s.is(CardType::Creature) && s.is(CardType::Artifact));
    assert!(s.chars.has_subtype("Equipment"));
    assert_eq!(t.pt(splitter), (5, 5));
    assert!(t.on_battlefield(splitter));
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(bears), (2, 2));
    // It can't be attached to a creature again: activating its equip ability does nothing.
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let _ = activate_containing(&mut t, P0, splitter, "Equip");
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(bears), (2, 2));
}
