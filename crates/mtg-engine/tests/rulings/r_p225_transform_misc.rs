//! Rulings batch P225 — more transform rulings: an Aura that transforms into a creature
//! becomes unattached (CR 704.5p), a daybound Aura cast at night still targets and enters
//! transformed (CR 702.145b, 303.4a), counters stay on a transforming permanent
//! (CR 712.18), and the Incubator to transform is chosen on resolution (CR 608.2c,
//! 701.53a).

use crate::r_s01_common::*;
use crate::r_s06_common::{attach_new, attached_to};
use crate::r_s13_common::add;
use crate::r_s17_common::*;
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

const LEECHES: &str = "Curse of Leeches // Leeching Lurker";
const SHADOWS: &str = "Grasping Shadows // Shadows' Lair";

#[test]
fn curse_of_leeches_becomes_unattached_as_it_becomes_leeching_lurker() {
    cr!("704.5p", "702.145c");
    ruling!(
        "Curse of Leeches // Leeching Lurker",
        "After Curse of Leeches transforms into Leeching Lurker, it will become unattached from the player it's attached to as a state-based action."
    );
    // (Curse of Leeches's "As this permanent transforms into Curse of Leeches, attach it
    // to a player" doesn't compile; it isn't involved.)
    let mut t = TestGame::new(2);
    let curse = attach_new(&mut t, P0, LEECHES, P1);
    assert_eq!(attached_to(&t, curse), Some(Entity::Player(P1)));
    t.g.day = Some(false);
    t.g.recompute();
    t.settle();
    assert_eq!(name_of(&t, curse), "Leeching Lurker");
    assert!(t.on_battlefield(curse));
    assert_eq!(attached_to(&t, curse), None);
}

#[test]
fn curse_of_leeches_cast_at_night_targets_a_player_and_enters_unattached() {
    cr!("702.145b", "303.4a", "608.3");
    ruling!(
        "Curse of Leeches // Leeching Lurker",
        "If you cast Curse of Leeches while it is night, you must still choose a target player"
    );
    let mut t = TestGame::new(2);
    t.g.day = Some(false);
    give_mana_for(&mut t, P0, LEECHES);
    let card = t.hand(P0, LEECHES);
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, card).go();
    assert!(crate::r_s02_common::target_candidates(&t, P0, from)
        .iter()
        .any(|c| c.contains(&Entity::Player(P1))));
    t.resolve_all();
    assert!(t.on_battlefield(spell));
    assert_eq!(name_of(&t, spell), "Leeching Lurker");
    assert_eq!(face(&t, spell), FaceState::Back);
    assert_eq!(attached_to(&t, spell), None);
}

#[test]
fn grasping_shadows_keeps_its_dread_counters_as_it_transforms() {
    cr!("712.18");
    ruling!(
        "Grasping Shadows // Shadows' Lair",
        "Grasping Shadows keeps its dread counters as it transforms into Shadows' Lair."
    );
    supported(SHADOWS);
    let mut t = TestGame::new(2);
    let shadows = t.battlefield(P0, SHADOWS);
    add(&mut t, shadows, "dread", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(name_of(&t, shadows), "Shadows' Lair");
    assert_eq!(t.counters(shadows, "dread"), 3);
}

#[test]
fn sunder_the_gateway_can_transform_an_incubator_you_already_controlled() {
    cr!("608.2c", "701.53a");
    ruling!(
        "Sunder the Gateway",
        "You can transform the Incubator token you just created or one you already controlled."
    );
    supported("Sunder the Gateway");
    let mut t = TestGame::new(2);
    t.enter(P0, "Norn's Inquisitor");
    t.resolve_all();
    let old = with_subtype(&t, P0, "Incubator")[0];
    give_mana_for(&mut t, P0, "Sunder the Gateway");
    let card = t.hand(P0, "Sunder the Gateway");
    t.answer_choose(P0, &[Entity::Object(old)]);
    t.cast(P0, card).modes(&[1]).go();
    t.resolve_all();
    let incubators = with_subtype(&t, P0, "Incubator");
    assert_eq!(incubators.len(), 1);
    assert_ne!(incubators[0], old);
    assert_eq!(face(&t, old), FaceState::Back);
    assert!(t.obj_now(old).chars.has_subtype("Phyrexian"));
    assert_eq!(t.counters(old, counters::PLUS1), 3);
}
