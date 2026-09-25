//! CR 301: artifacts — casting and resolving, artifact types, Equipment, Fortifications,
//! and Vehicles.

use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::replacement::TokenCreate;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;

#[test]
fn artifact_spells_are_cast_at_sorcery_speed_and_use_the_stack() {
    cr!("301.1");
    check_sorcery_timing("Ornithopter", "{0}");
    check_sorcery_timing("Bonesplitter", "{1}");
}

#[test]
fn an_artifact_spell_resolves_onto_the_battlefield_under_its_controllers_control() {
    cr!("301.2");
    // P0 casts an artifact card P1 owns.
    let mut t = TestGame::new(2);
    let perm = cast_others_card(&mut t, P0, P1, "Ornithopter", "{0}", &[]);
    assert!(t.on_battlefield(perm));
    assert_eq!(t.obj(perm).controller, P0);
    assert_eq!(t.obj(perm).owner, P1);
}

#[test]
fn artifact_subtypes_are_single_words_and_an_artifact_may_have_several() {
    cr!("301.3");
    // "Artifact — Equipment Vehicle": two artifact types.
    assert_eq!(subtypes_of("Rover Blades"), vec!["Equipment", "Vehicle"]);
    for s in ["Equipment", "Vehicle", "Fortification", "Treasure", "Clue"] {
        assert_eq!(subtype_kind(s), Some(SubtypeKind::Artifact), "{s}");
    }
    // Each is a subtype of the permanent: it's an Equipment (it has equip and can be
    // attached) and a Vehicle.
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P0, "Rover Blades");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.obj(blades).chars.has_subtype("Equipment"));
    assert!(t.obj(blades).chars.has_subtype("Vehicle"));
    t.lands(P0, "Wastes", 4);
    t.activate(P0, blades, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.obj(blades).attached_to, Some(Entity::Object(bears)));
    assert!(t.obj(bears).has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn being_colorless_and_being_an_artifact_are_unrelated() {
    cr!("301.4");
    let mut t = TestGame::new(2);
    // A colorless artifact, a blue artifact creature, and a colorless nonartifact
    // creature.
    let thopter = t.battlefield(P1, "Ornithopter");
    let master = t.battlefield(P1, "Master of Etherium");
    let crusher = t.battlefield(P1, "Ulamog's Crusher");
    assert!(t.obj(thopter).chars.colors.is_colorless());
    assert!(t.obj(master).chars.colors.contains(Color::Blue));
    assert!(t.obj(master).is(CardType::Artifact));
    assert!(t.obj(crusher).chars.colors.is_colorless());
    assert!(!t.obj(crusher).is(CardType::Artifact));
    // "Destroy target artifact" can destroy the colored artifact but can't target the
    // colorless creature.
    t.lands(P0, "Mountain", 4);
    let shatter = t.hand(P0, "Shatter");
    t.cast(P0, shatter).target(master).go();
    let offered = last_target_candidates(&t, P0);
    assert!(offered.contains(&Entity::Object(master)));
    assert!(offered.contains(&Entity::Object(thopter)));
    assert!(!offered.contains(&Entity::Object(crusher)));
    t.resolve();
    assert!(!t.on_battlefield(master));
    assert!(t.on_battlefield(crusher));
}

#[test]
fn the_equipped_creature_is_the_one_the_equipment_is_attached_to() {
    cr!("301.5a");
    let mut t = TestGame::new(2);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, splitter, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert_eq!(t.obj(splitter).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (4, 2));
    // Equipping another creature: that one is now the equipped creature.
    t.activate(P0, splitter, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(giant), (5, 3));
}

#[test]
fn an_equipment_put_onto_the_battlefield_attached_to_something_it_cant_equip_is_unattached() {
    cr!("301.5e");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.graveyard(P0, "Bonesplitter");
    let b = t.graveyard(P0, "Bonesplitter");
    let put_attached = Effect::Move {
        what: Sel::Target(0),
        to: Destination {
            attached_to: Some(Sel::Target(1)),
            ..Destination::battlefield()
        },
    };
    // Attached to a land, which it can't equip: it enters unattached.
    run_effect_slots(
        &mut t,
        P0,
        None,
        put_attached.clone(),
        vec![vec![Entity::Object(a)], vec![Entity::Object(forest)]],
    );
    let a = t.g.current(a);
    assert!(t.on_battlefield(a));
    assert_eq!(t.obj(a).attached_to, None);
    // Attached to a creature: it enters attached to it.
    run_effect_slots(
        &mut t,
        P0,
        None,
        put_attached,
        vec![vec![Entity::Object(b)], vec![Entity::Object(bears)]],
    );
    let b = t.g.current(b);
    assert_eq!(t.obj(b).attached_to, Some(Entity::Object(bears)));
    assert_eq!(t.pt(bears), (4, 2));
    // An Equipment token created attached to an undefined object is created unattached.
    let mut chars = t.obj(b).chars.clone();
    chars.name = SmolStr::new("Blade Token");
    let spec = TokenCreate {
        chars,
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    let toks = t.g.create_tokens_attached(P0, spec, 1, None, None);
    assert_eq!(toks.len(), 1);
    assert!(t.on_battlefield(toks[0]));
    assert_eq!(t.obj(toks[0]).attached_to, None);
}

#[test]
fn equipped_creature_refers_to_what_a_non_equipment_permanent_is_attached_to() {
    cr!("301.5f");
    // An Aura whose ability refers to the "equipped creature".
    let odd = oracle_card(
        "Odd Harness",
        "Enchantment — Aura",
        "{1}",
        None,
        "Enchant creature\nEquipped creature gets +2/+2.",
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let c = t.custom(P0, odd, Zone::Hand(P0));
    t.lands(P0, "Wastes", 1);
    t.cast(P0, c).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn a_fortification_is_attached_to_a_land_by_fortify() {
    cr!("301.6", "702.67a");
    ruling!("Darksteel Garrison", "Fortification is to lands what Equipment is to creatures");
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Darksteel Garrison");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lands = t.lands(P0, "Wastes", 3);
    let forest = t.battlefield(P0, "Forest");
    let their_land = t.battlefield(P1, "Forest");
    t.activate(P0, garrison, 0, &[Entity::Object(forest)])
        .unwrap();
    // Fortify targets a land its controller controls: not a creature.
    let offered = last_target_candidates(&t, P0);
    assert!(offered.contains(&Entity::Object(forest)));
    assert!(!offered.contains(&Entity::Object(bears)));
    assert!(!offered.contains(&Entity::Object(their_land)));
    t.resolve();
    assert_eq!(t.obj(garrison).attached_to, Some(Entity::Object(forest)));
    // The fortified land has indestructible.
    assert!(t.obj(forest).has_keyword(KeywordKind::Indestructible));
    t.g.destroy(forest, None);
    t.settle();
    assert!(t.on_battlefield(forest));
    // An effect trying to attach it to a creature doesn't move it (as for Equipment,
    // CR 301.5b).
    run_effect(
        &mut t,
        P0,
        Some(garrison),
        Effect::Attach {
            what: Sel::This,
            to: Sel::Target(0),
        },
        &[Entity::Object(bears)],
    );
    assert_eq!(t.obj(garrison).attached_to, Some(Entity::Object(forest)));
    // If the fortified permanent stops being a land, it becomes unattached but stays on
    // the battlefield.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetTypes {
                types: vec![CardType::Artifact],
                subtypes: vec![],
            }],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(forest)],
    );
    t.settle();
    assert_eq!(t.obj(garrison).attached_to, None);
    assert!(t.on_battlefield(garrison));
    let _ = lands;
}

#[test]
fn a_fortification_that_is_a_creature_cant_fortify() {
    cr!("301.6");
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Darksteel Garrison");
    let forest = t.battlefield(P0, "Forest");
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::AddTypes(vec![CardType::Creature])],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(garrison)],
    );
    run_effect(
        &mut t,
        P0,
        Some(garrison),
        Effect::Attach {
            what: Sel::This,
            to: Sel::Target(0),
        },
        &[Entity::Object(forest)],
    );
    assert_eq!(t.obj(garrison).attached_to, None);
}

#[test]
fn a_vehicle_has_its_printed_power_and_toughness_only_while_its_a_creature() {
    cr!("301.7", "301.7a", "301.7b");
    let mut t = TestGame::new(2);
    let balloon = t.battlefield(P0, "War Balloon");
    let o = t.obj(balloon);
    assert!(o.chars.has_subtype("Vehicle") && o.has_keyword(KeywordKind::Crew));
    // Not a creature: no power or toughness, and it can't attack.
    assert!(!o.is_creature());
    assert_eq!((o.chars.power, o.chars.toughness), (None, None));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(balloon, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    // With three fire counters it's an artifact creature with its printed 4/3...
    t.set_step(P0, Step::PostcombatMain);
    t.g.add_counters(Entity::Object(balloon), "fire", 3, None);
    t.g.recompute();
    let o = t.obj(balloon);
    assert!(o.is_creature() && o.is(CardType::Artifact));
    assert_eq!(t.pt(balloon), (4, 3));
    // ...which other effects modify.
    t.battlefield(P0, "Glorious Anthem");
    assert_eq!(t.pt(balloon), (5, 4));
}
