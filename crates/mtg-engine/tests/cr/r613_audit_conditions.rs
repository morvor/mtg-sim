//! CR 613 audit: a conditional static ability's effect keeps its object's timestamp when
//! its condition becomes true (CR 613.7a), and Blood Moon against lands that are also
//! creatures (CR 305.7).

use crate::r613_audit_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_conditional_static_keeps_its_objects_timestamp_when_its_condition_becomes_true() {
    // Vampire Nocturnus: "As long as the top card of your library is black, this creature
    // and other Vampire creatures you control get +2/+1 and have flying." The effect has
    // Nocturnus's timestamp (CR 613.7a) even if its condition becomes true only after
    // Gravity Sphere ("All creatures lose flying") entered, so the Sphere's later effect
    // still removes the flying while the +2/+1 (layer 7c) applies.
    cr!("613.7a", "613.7", "613.1f", "613.4c", "613.9");
    let mut t = TestGame::new(2);
    let nocturnus = t.battlefield(P0, "Vampire Nocturnus");
    t.g.recompute();
    assert!(!has_kw(&t, nocturnus, KeywordKind::Flying), "a colorless filler is on top");
    assert_eq!(t.pt(nocturnus), (3, 3));
    let sphere = t.battlefield(P1, "Gravity Sphere");
    t.g.recompute();
    // A black card on top: the condition is true now, but the effect isn't newer.
    t.library_top(P0, "Doom Blade");
    t.g.recompute();
    assert_eq!(t.pt(nocturnus), (5, 4), "the +2/+1 applies");
    assert!(!has_kw(&t, nocturnus, KeywordKind::Flying), "Gravity Sphere is later");
    // The Sphere leaves: flying. A new Sphere is later again.
    t.g.destroy(sphere, None);
    t.settle();
    t.g.recompute();
    assert!(has_kw(&t, nocturnus, KeywordKind::Flying));
    t.battlefield(P1, "Gravity Sphere");
    t.g.recompute();
    assert!(!has_kw(&t, nocturnus, KeywordKind::Flying));
    assert_eq!(t.pt(nocturnus), (5, 4));
}

#[test]
fn humility_turns_off_a_conditional_creature_lord_even_when_its_condition_becomes_true() {
    // Nocturnus's effect first applies in layer 6, the layer in which Humility removes
    // its ability, so it depends on Humility's effect (CR 613.8a: applying Humility's
    // changes its existence) and waits (613.8b); by then the ability is gone, the effect
    // never starts, and CR 613.6 doesn't carry its +2/+1 into layer 7c either.
    cr!("613.8", "613.8a", "613.8b", "613.6", "613.1f", "613.4c");
    ruling!("Humility", "Removes all creature abilities");
    let mut t = TestGame::new(2);
    let nocturnus = t.battlefield(P0, "Vampire Nocturnus");
    let humility = t.battlefield(P1, "Humility");
    t.g.recompute();
    assert_eq!(t.pt(nocturnus), (1, 1));
    t.library_top(P0, "Doom Blade");
    t.g.recompute();
    assert_eq!(t.pt(nocturnus), (1, 1), "no +2/+1: the effect never started");
    assert!(!has_kw(&t, nocturnus, KeywordKind::Flying));
    assert_eq!(ability_count(&t, nocturnus), 0);
    // Humility leaves: the whole effect applies.
    t.g.destroy(humility, None);
    t.settle();
    t.g.recompute();
    assert_eq!(t.pt(nocturnus), (5, 4));
    assert!(has_kw(&t, nocturnus, KeywordKind::Flying));
}

#[test]
fn dryad_arbor_under_blood_moon_is_a_mountain_that_is_still_a_green_dryad_creature() {
    // Setting a land's subtype (CR 305.7) doesn't touch its card types or color.
    cr!("305.7", "613.1d", "613.1e", "613.8a");
    ruling!(
        "Dryad Arbor",
        "If Dryad Arbor is changed into another basic land type, it continues to be a green Dryad creature"
    );
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P0, "Dryad Arbor");
    t.battlefield(P1, "Blood Moon");
    t.g.recompute();
    assert!(has_subtype(&t, arbor, "Mountain"));
    assert!(!has_subtype(&t, arbor, "Forest"));
    assert!(has_subtype(&t, arbor, "Dryad"));
    assert!(t.obj_now(arbor).is_creature());
    assert!(is_type(&t, arbor, CardType::Land));
    assert_eq!(t.pt(arbor), (1, 1));
    assert_eq!(colors(&t, arbor), color_set(&[Color::Green]));
    assert!(has_ability_text(&t, arbor, "Add {R}"));
    assert!(!has_ability_text(&t, arbor, "Add {G}"));
    // Dryad of the Ilysian Grove (later): every basic land type again.
    t.battlefield(P0, "Dryad of the Ilysian Grove");
    t.g.recompute();
    assert!(has_subtype(&t, arbor, "Forest") && has_subtype(&t, arbor, "Mountain"));
    assert!(has_ability_text(&t, arbor, "Add {G}"));
}

#[test]
fn blood_moon_doesnt_undo_an_animated_lands_granted_characteristics() {
    // Inkmoth Nexus animated before Blood Moon enters: Blood Moon removes its rules text
    // (its mana and animation abilities) and land type, not the characteristics the
    // resolved animation gave it (CR 305.7: granted abilities stay; layers 4, 6 and 7b).
    cr!("305.7", "613.1d", "613.1f", "613.4b", "613.7", "613.7b");
    ruling!(
        "Blood Moon",
        "Nonbasic lands will lose any other land types and abilities they had"
    );
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P0, "Inkmoth Nexus");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, nexus, "becomes", &[]);
    t.battlefield(P1, "Blood Moon");
    t.g.recompute();
    assert!(has_subtype(&t, nexus, "Mountain"));
    assert!(t.obj_now(nexus).is_creature());
    assert!(is_type(&t, nexus, CardType::Artifact));
    assert!(has_subtype(&t, nexus, "Blinkmoth"));
    assert!(has_kw(&t, nexus, KeywordKind::Flying));
    assert!(has_kw(&t, nexus, KeywordKind::Infect));
    assert_eq!(t.pt(nexus), (1, 1));
    assert!(has_ability_text(&t, nexus, "Add {R}"));
    assert!(!has_ability_text(&t, nexus, "Add {C}"));
    assert!(!has_ability_text(&t, nexus, "becomes"), "its animation ability is gone");
    end_turn(&mut t);
    assert!(!t.obj_now(nexus).is_creature());
    assert!(has_subtype(&t, nexus, "Mountain"));
    assert!(t.obj_now(nexus).chars.power.is_none());
    assert_eq!(ability_count(&t, nexus), 1, "only the Mountain's mana ability");
}
