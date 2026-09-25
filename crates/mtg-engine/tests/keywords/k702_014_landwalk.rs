//! CR 702.14 Landwalk.

use crate::common_k702_011_017::*;
use mtg_engine::ability::Filter;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The land filter of the object's (first) landwalk ability, for inspection.
fn landwalk_filter(t: &TestGame, id: ObjectId) -> String {
    let f: Option<Filter> = t
        .obj_now(id)
        .chars
        .keyword(KeywordKind::Landwalk)
        .and_then(|k| k.filter.clone());
    format!("{f:?}")
}

#[test]
fn landwalk_keywords_name_a_land_type_or_types() {
    cr!("702.14", "702.14a");
    for name in [
        "Bog Wraith",
        "Dryad Sophisticate",
        "Livonya Silone",
        "Legions of Lim-Dûl",
        "Vectis Gloves",
    ] {
        assert_supported(name);
    }
    let mut t = TestGame::new(2);
    // "[type]walk" with a land type...
    let wraith = t.battlefield(P0, "Bog Wraith");
    assert_eq!(landwalk_filter(&t, wraith), "Some(Subtype(\"Swamp\"))");
    // ...or "land" with supertypes/card types, and a supertype with a land type.
    let dryad = t.battlefield(P0, "Dryad Sophisticate");
    assert_eq!(
        landwalk_filter(&t, dryad),
        "Some(And([Type(Land), Not(Supertype(Basic))]))"
    );
    let livonya = t.battlefield(P0, "Livonya Silone");
    assert_eq!(
        landwalk_filter(&t, livonya),
        "Some(And([Type(Land), Supertype(Legendary)]))"
    );
    let legions = t.battlefield(P0, "Legions of Lim-Dûl");
    assert_eq!(
        landwalk_filter(&t, legions),
        "Some(And([Supertype(Snow), Subtype(\"Swamp\")]))"
    );
}

#[test]
fn landwalk_is_an_evasion_ability() {
    cr!("702.14b");
    assert!(KeywordKind::Landwalk.is_evasion());
    let mut t = TestGame::new(2);
    let wraith = t.battlefield(P0, "Bog Wraith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Swamp", 1);
    attack_with(&mut t, &[(wraith, Entity::Player(P1))]);
    assert!(!t.g.can_block(bears, wraith));
    block_and_finish(&mut t, P1, &[(bears, wraith)]);
    assert!(!is_blocking(&t, bears));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn landwalk_unblockable_while_defending_player_controls_such_a_land() {
    cr!("702.14c");
    // Swampwalk: only while the defending player controls a Swamp.
    let mut t = TestGame::new(2);
    let wraith = t.battlefield(P0, "Bog Wraith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 1);
    t.lands(P0, "Swamp", 1); // the attacking player's Swamps don't matter
    attack_with(&mut t, &[(wraith, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, wraith));
    t.lands(P1, "Swamp", 1);
    assert!(!t.g.can_block(bears, wraith));
    // Nonbasic landwalk ("without the specified supertype").
    let mut t = TestGame::new(2);
    let dryad = t.battlefield(P0, "Dryad Sophisticate");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 3);
    attack_with(&mut t, &[(dryad, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, dryad));
    t.battlefield(P1, "Evolving Wilds");
    assert!(!t.g.can_block(bears, dryad));
    // Legendary landwalk.
    let mut t = TestGame::new(2);
    let livonya = t.battlefield(P0, "Livonya Silone");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(livonya, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, livonya));
    t.battlefield(P1, "Karakas");
    assert!(!t.g.can_block(bears, livonya));
    // Artifact landwalk ("with the specified type"), granted by an Equipment.
    let mut t = TestGame::new(2);
    let bears0 = t.battlefield(P0, "Grizzly Bears");
    let gloves = t.battlefield(P0, "Vectis Gloves");
    t.g.attach(gloves, Entity::Object(bears0));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Ornithopter"); // an artifact, but not a land
    attack_with(&mut t, &[(bears0, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, bears0));
    t.battlefield(P1, "Darksteel Citadel");
    assert!(!t.g.can_block(bears, bears0));
    // Snow swampwalk: both the supertype and the subtype.
    let mut t = TestGame::new(2);
    let legions = t.battlefield(P0, "Legions of Lim-Dûl");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Swamp", 1);
    t.lands(P1, "Snow-Covered Forest", 1);
    attack_with(&mut t, &[(legions, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, legions));
    t.lands(P1, "Snow-Covered Swamp", 1);
    assert!(!t.g.can_block(bears, legions));
}

#[test]
fn landwalk_abilities_dont_cancel_one_another() {
    cr!("702.14d");
    // The rule's example: a snow Forest, and a blocker with snow forestwalk itself.
    let mut t = TestGame::new(2);
    let dryad = t.battlefield(P0, "Rime Dryad");
    let their_dryad = t.battlefield(P1, "Rime Dryad");
    t.lands(P1, "Snow-Covered Forest", 1);
    attack_with(&mut t, &[(dryad, Entity::Player(P1))]);
    assert!(!t.g.can_block(their_dryad, dryad));
    block_and_finish(&mut t, P1, &[(their_dryad, dryad)]);
    assert!(!is_blocking(&t, their_dryad));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn multiple_instances_of_the_same_landwalk_are_redundant() {
    cr!("702.14e");
    assert_supported("Lord of Atlantis");
    let mut t = TestGame::new(2);
    // Each Lord gives the other Merfolk islandwalk: the Pearl Trident merfolk has two.
    t.battlefield(P0, "Lord of Atlantis");
    t.battlefield(P0, "Lord of Atlantis");
    let merfolk = t.battlefield(P0, "Merfolk of the Pearl Trident");
    assert_eq!(keyword_count(&t, merfolk, KeywordKind::Landwalk), 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(merfolk, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, merfolk));
    t.lands(P1, "Island", 1);
    assert!(!t.g.can_block(bears, merfolk));
}

#[test]
fn creatures_with_landwalk_can_be_blocked_as_though_they_didnt_have_it() {
    cr!("702.14c");
    assert_supported("Undertow");
    assert_supported("Staff of the Ages");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lord of Atlantis");
    let merfolk = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let wraith = t.battlefield(P0, "Bog Wraith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Island", 1);
    t.lands(P1, "Swamp", 1);
    // Undertow only affects islandwalk.
    t.battlefield(P1, "Undertow");
    attack_with(
        &mut t,
        &[(merfolk, Entity::Player(P1)), (wraith, Entity::Player(P1))],
    );
    assert!(t.g.can_block(bears, merfolk));
    assert!(!t.g.can_block(giant, wraith));
    // Staff of the Ages affects every landwalk ability.
    t.battlefield(P0, "Staff of the Ages");
    t.g.recompute();
    assert!(t.g.can_block(giant, wraith));
}

#[test]
fn a_creature_that_loses_its_landwalk_abilities_can_be_blocked() {
    cr!("702.14c");
    assert_supported("Hammerheim");
    let mut t = TestGame::new(2);
    let wraith = t.battlefield(P0, "Bog Wraith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Swamp", 1);
    // "{T}: Target creature loses all landwalk abilities until end of turn."
    let hammerheim = t.battlefield(P1, "Hammerheim");
    t.activate(P1, hammerheim, 1, &[Entity::Object(wraith)])
        .unwrap();
    t.resolve_all();
    assert_eq!(keyword_count(&t, wraith, KeywordKind::Landwalk), 0);
    attack_with(&mut t, &[(wraith, Entity::Player(P1))]);
    assert!(t.g.can_block(bears, wraith));
}
