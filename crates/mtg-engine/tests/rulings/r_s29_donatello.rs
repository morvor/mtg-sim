//! Rulings batch S29 — Donatello, Mutant Mechanic: "{T}: Put three +1/+1 counters on target
//! artifact you control. If it isn't a creature, it becomes a 0/0 Robot creature in
//! addition to its other types. Activate only as a sorcery." The effect sets its base
//! power and toughness (layer 7b, CR 613.4b) with a later timestamp than a station
//! striation's or a Vehicle's printed power and toughness (CR 613.7), adds types without
//! removing any (CR 205.1b), and doesn't remove abilities.

use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new, attached_to};
use crate::r_s16_common::add_charge;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Donatello targets `artifact`, and the ability resolves.
fn donatello(t: &mut TestGame, artifact: ObjectId) {
    supported("Donatello, Mutant Mechanic");
    let d = t.battlefield(P0, "Donatello, Mutant Mechanic");
    t.answer_targets(P0, &[Entity::Object(artifact)]);
    activate_containing(t, P0, d, "Robot").expect("Donatello's ability");
    t.resolve_all();
}

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.is(CardType::Creature)
}

#[test]
fn a_station_artifact_stays_0_0_as_charge_counters_are_added() {
    cr!("613.4b", "613.7", "721.2a");
    ruling!(
        "Donatello, Mutant Mechanic",
        "If the target artifact has station, its base power and toughness will be set to 0/0. Adding charge counters to that artifact won't restore the power and toughness printed in any of its striations."
    );
    supported("Atmospheric Greenhouse");
    // Atmospheric Greenhouse: "Station. 8+ | Flying, trample" with a 5/4 box.
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Atmospheric Greenhouse");
    donatello(&mut t, ship);
    assert!(is_creature(&t, ship));
    assert!(t.obj_now(ship).chars.has_subtype("Robot"));
    assert_eq!(t.pt(ship), (3, 3));
    add_charge(&mut t, ship, 8);
    // The striation's abilities apply, but not its 5/4.
    assert!(t.obj_now(ship).chars.has_keyword(KeywordKind::Flying));
    assert_eq!(t.pt(ship), (3, 3));
    // An artifact that's already a creature (a charged Greenhouse, 5/4) just gets the
    // counters.
    let mut t = TestGame::new(2);
    let ship = t.battlefield(P0, "Atmospheric Greenhouse");
    add_charge(&mut t, ship, 8);
    donatello(&mut t, ship);
    assert!(!t.obj_now(ship).chars.has_subtype("Robot"));
    assert_eq!(t.pt(ship), (8, 7));
}

#[test]
fn a_vehicle_stays_0_0_when_crewed() {
    cr!("613.4b", "613.7", "702.122a");
    ruling!(
        "Donatello, Mutant Mechanic",
        "If the target artifact is a Vehicle, its base power and toughness will be set to 0/0. Crewing that Vehicle will not restore its power and toughness."
    );
    ruling!(
        "Donatello, Mutant Mechanic",
        "Donatello's first ability doesn't remove any abilities the target artifact has."
    );
    ruling!(
        "Donatello, Mutant Mechanic",
        "The artifact retains any types, subtypes, or supertypes it has."
    );
    supported("Aradara Express");
    // Aradara Express: an 8/6 Vehicle with menace and crew 4.
    let mut t = TestGame::new(2);
    let express = t.battlefield(P0, "Aradara Express");
    donatello(&mut t, express);
    let o = t.obj_now(express);
    assert!(o.chars.is(CardType::Artifact) && o.chars.is(CardType::Creature));
    assert!(o.chars.has_subtype("Vehicle") && o.chars.has_subtype("Robot"));
    assert!(o.chars.has_keyword(KeywordKind::Menace));
    assert!(o.chars.has_keyword(KeywordKind::Crew));
    assert_eq!(t.pt(express), (3, 3));
    // Crewing it (Hill Giant and Grizzly Bears, total power 5) doesn't restore its 8/6.
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crate::r_s04_common::crew(&mut t, P0, express, &[giant, bears]));
    t.resolve_all();
    assert_eq!(t.pt(express), (3, 3));
}

#[test]
fn an_attached_equipment_becomes_unattached_and_cant_be_attached() {
    cr!("301.5c", "704.5n");
    ruling!(
        "Donatello, Mutant Mechanic",
        "If the target artifact is an attached Equipment, it becomes unattached. If an Equipment without reconfigure becomes an artifact creature, it can't be attached to another creature."
    );
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", bears);
    assert_eq!(t.pt(bears), (4, 2));
    donatello(&mut t, splitter);
    assert!(is_creature(&t, splitter));
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(bears), (2, 2));
    // Its equip ability can't attach it.
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let _ = activate_containing(&mut t, P0, splitter, "Equip");
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(bears), (2, 2));
}
