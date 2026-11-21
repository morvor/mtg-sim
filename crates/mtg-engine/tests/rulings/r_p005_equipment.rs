//! Rulings batch P005 — Dan Lewis ("Noncreature, non-Equipment artifacts you control are
//! Equipment in addition to their other types and have “Equipped creature gets +1/+0” and
//! equip {1}."): a type-changing effect that stops applying (CR 613.8a dependency, 611.3a)
//! and the state-based actions for attached permanents (CR 704.5m, 704.5p), and an Aura
//! that is also an Equipment (CR 301.5a, 303.4b, 303.4j, 701.3b).

use crate::r_p006_common::{in_graveyard, obj};
use crate::r_p044_common::kill;
use crate::r_s01_common::supported;
use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const DAN: &str = "Dan Lewis";

/// Activates the equip ability `equipment` has (from Dan Lewis) targeting `target`, with
/// a Wastes for {1}, and resolves it.
fn equip(t: &mut TestGame, equipment: ObjectId, target: ObjectId) {
    t.lands(P0, "Wastes", 1);
    t.g.recompute();
    let index = t
        .obj_now(equipment)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.to_lowercase().contains("equip"))
        .expect("an equip ability");
    t.activate(P0, equipment, index, &[obj(target)]).unwrap();
    t.resolve_all();
}

fn is_equipment(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).chars.has_subtype("Equipment")
}

#[test]
fn dan_lewis_stops_applying_to_an_artifact_that_becomes_a_creature() {
    cr!("613.8a", "704.5p", "611.3a");
    ruling!(
        "Dan Lewis",
        "If an artifact affected by Dan Lewis later becomes a creature, Dan's effect no longer applies to it. That artifact will stop being an Equipment and become unattached from any creature it was attached to; it will remain on the battlefield. The same thing happens if Dan leaves the battlefield."
    );
    supported(DAN);
    supported("Chimeric Staff");
    for dan_leaves in [false, true] {
        let mut t = TestGame::new(2);
        let dan = t.battlefield(P0, DAN);
        // Chimeric Staff: "{X}: This artifact becomes an X/X Construct artifact creature
        // until end of turn."
        let staff = t.battlefield(P0, "Chimeric Staff");
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert!(is_equipment(&mut t, staff));
        equip(&mut t, staff, bears);
        assert_eq!(t.obj_now(staff).attached_to, Some(obj(bears)));
        assert_eq!(t.pt(bears), (3, 2));
        if dan_leaves {
            kill(&mut t, dan);
        } else {
            t.lands(P0, "Wastes", 2);
            let x = t
                .obj_now(staff)
                .chars
                .abilities
                .iter()
                .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
                .position(|a| a.text.contains("{X}"))
                .unwrap();
            t.answer(P0, DecisionKind::X, mtg_engine::decision::Answer::Number(2));
            t.activate(P0, staff, x, &[]).unwrap();
            t.resolve_all();
            assert!(t.obj_now(staff).is(CardType::Creature));
        }
        assert!(!is_equipment(&mut t, staff), "dan leaves: {dan_leaves}");
        assert!(t.on_battlefield(staff));
        assert_eq!(t.obj_now(staff).attached_to, None);
        assert_eq!(t.pt(bears), (2, 2));
    }
}

#[test]
fn an_aura_that_becomes_an_artifact_is_also_an_equipment() {
    cr!("301.5a", "303.4b", "303.4j", "701.3b", "704.5m");
    ruling!(
        "Dan Lewis",
        "While Dan is on the battlefield, an Aura enchantment that somehow becomes an artifact in addition to its other types will also become an Equipment. Any of its abilities that refer to either \"enchanted creature\" or \"equipped creature\" refer to the creature it's currently attached to. It can be attached to other creatures using its equip ability. If you try to attach the Aura Equipment to a creature it can't legally be attached to, it remains where it is. If the creature it's attached to becomes an illegal permanent for it to enchant, the Aura Equipment will be put into its owner's graveyard as a state-based action."
    );
    supported("Mycosynth Lattice");
    supported("Animate Wall");
    supported("Tel-Jilad Defiance");
    // Mycosynth Lattice: "All permanents are artifacts in addition to their other types."
    let mut t = TestGame::new(2);
    t.battlefield(P0, DAN);
    t.battlefield(P0, "Mycosynth Lattice");
    // Holy Strength ("Enchant creature. Enchanted creature gets +1/+2.") on the Bears.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let strength = t.battlefield(P0, "Holy Strength");
    t.g.attach(strength, obj(bears));
    assert!(is_equipment(&mut t, strength));
    assert!(t.obj_now(strength).chars.has_subtype("Aura"));
    // +1/+2 (enchanted creature) and +1/+0 (equipped creature).
    assert_eq!(t.pt(bears), (4, 4));
    // Its equip ability moves it to the Hill Giant.
    equip(&mut t, strength, giant);
    assert_eq!(t.obj_now(strength).attached_to, Some(obj(giant)));
    assert_eq!(t.pt(giant), (5, 5));
    assert_eq!(t.pt(bears), (2, 2));
    // The Giant gains protection from artifacts (Tel-Jilad Defiance): an Aura can't
    // enchant it, so the Aura Equipment is put into its owner's graveyard rather than
    // becoming unattached.
    t.lands(P0, "Forest", 2);
    let defiance = t.hand(P0, "Tel-Jilad Defiance");
    t.cast(P0, defiance).target(giant).go();
    t.resolve_all();
    assert!(in_graveyard(&t, strength), "{}", t.dump_log());
    // Animate Wall ("Enchant Wall") on Wall of Stone can't be attached to the Bears with
    // its equip ability: it stays on the Wall.
    let wall = t.battlefield(P0, "Wall of Stone");
    let animate = t.battlefield(P0, "Animate Wall");
    t.g.attach(animate, obj(wall));
    assert!(is_equipment(&mut t, animate));
    equip(&mut t, animate, bears);
    assert_eq!(t.obj_now(animate).attached_to, Some(obj(wall)));
    assert!(t.on_battlefield(animate));
}
