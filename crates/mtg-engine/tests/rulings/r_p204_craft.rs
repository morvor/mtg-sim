//! Rulings batch P204 — craft (CR 702.167): Wretched Bonemass, The Grim Captain.

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Action;
use mtg_engine::events::Event;
use mtg_engine::testing::*;
use mtg_engine::*;

const ALTAR: &str = "Altar of the Wretched // Wretched Bonemass";
const THRONE: &str = "Throne of the Grim Captain // The Grim Captain";

/// The uid of `src`'s activated ability whose text starts with "Craft".
fn craft_uid(t: &mut TestGame, src: ObjectId) -> u64 {
    t.g.recompute();
    t.g.obj(t.g.current(src))
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with("Craft"))
        .map(|a| a.uid)
        .expect("a craft ability")
}

/// Whether P0 could activate the craft ability of `src` now.
fn can_craft(t: &mut TestGame, src: ObjectId) -> bool {
    let uid = craft_uid(t, src);
    t.g.turn.priority = Some(P0);
    t.g.legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Activate { source, ability } if *source == src && *ability == uid))
}

/// P0 activates the craft ability of `src` exiling `materials`, and it resolves.
fn craft(t: &mut TestGame, src: ObjectId, materials: &[ObjectId]) {
    let es: Vec<Entity> = materials.iter().map(|o| Entity::Object(*o)).collect();
    t.answer_choose(P0, &es);
    let uid = craft_uid(t, src);
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, src, uid).expect("craft");
    t.clear_answers();
    t.resolve_all();
}

fn one(t: &TestGame, name: &str) -> ObjectId {
    let v = t.named_on_battlefield(name);
    assert_eq!(v.len(), 1, "{name}");
    v[0]
}

#[test]
fn wretched_bonemass_uses_a_crafting_cards_characteristic_defining_ability() {
    cr!("702.167c", "604.3", "208.2a");
    ruling!(
        "Altar of the Wretched // Wretched Bonemass",
        "If any of the exiled cards has a characteristic-defining ability that defines its power and/or toughness, that ability will apply."
    );
    // (Its keyword-granting ability is not supported yet; its power and toughness
    // ability is.)
    supported("Tarmogoyf");
    // Tarmogoyf: "Tarmogoyf's power is equal to the number of card types among cards in
    // all graveyards and its toughness is equal to that number plus 1." Wretched
    // Bonemass: "power and toughness are each equal to the total power of the exiled
    // cards used to craft it."
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, ALTAR);
    let goyf = t.graveyard(P0, "Tarmogoyf");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P1, "Forest");
    t.lands(P0, "Swamp", 4);
    craft(&mut t, altar, &[goyf]);
    let bonemass = one(&t, "Wretched Bonemass");
    // Instant and land: Tarmogoyf is a 2/3 in exile.
    assert_eq!(t.pt(bonemass), (2, 2));
    // It changes with the graveyards.
    t.graveyard(P1, "Mind Rot");
    t.graveyard(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(bonemass), (4, 4));
}

#[test]
fn the_grim_captains_materials_must_be_separate_objects() {
    cr!("702.167a", "702.73a", "601.2h");
    ruling!(
        "Throne of the Grim Captain // The Grim Captain",
        "The Dinosaur, Merfolk, Pirate, and Vampire exiled to pay Throne of the Grim Captain's craft cost need to be separate objects. One creature with changeling is not enough."
    );
    supported(THRONE);
    supported("Changeling Outcast");
    // Changeling Outcast (every creature type) with a Vampire and a Merfolk: three
    // objects, not enough.
    let mut t = TestGame::new(2);
    let throne = t.battlefield(P0, THRONE);
    t.graveyard(P0, "Changeling Outcast");
    t.graveyard(P0, "Vampire Interloper");
    t.graveyard(P0, "Merfolk of the Pearl Trident");
    t.lands(P0, "Wastes", 4);
    assert!(!can_craft(&mut t, throne));
    // A fourth object (a Dinosaur): the changeling can be the Pirate.
    let dino = t.graveyard(P0, "Colossal Dreadmaw");
    assert!(can_craft(&mut t, throne));
    let mut materials: Vec<ObjectId> = t.g.player(P0).graveyard.iter().copied().collect();
    materials.retain(|m| *m != dino);
    materials.push(dino);
    craft(&mut t, throne, &materials);
    one(&t, "The Grim Captain");
}
