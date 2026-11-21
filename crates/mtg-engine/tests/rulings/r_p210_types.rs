//! Rulings batch P210 — enchant (CR 303.4, 702.5): Auras that change the enchanted
//! permanent's types, abilities and other characteristics (CR 205.1, 305.7, 613), and
//! Auras whose enchant restriction stops being met (CR 303.4d, 704.5m).

use crate::r_p209_common::{cast_aura, cast_spell};
use crate::r_p210_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The texts of the mana abilities the permanent has now.
fn mana_abilities(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
        .map(|a| a.text.to_string())
        .collect::<Vec<_>>()
        .join(" | ")
}

fn n_abilities(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id).chars.abilities.len()
}

fn types(t: &TestGame, id: ObjectId) -> Vec<CardType> {
    t.obj_now(id).chars.card_types.iter().collect()
}

fn subtypes(t: &TestGame, id: ObjectId) -> Vec<String> {
    t.obj_now(id)
        .chars
        .subtypes
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn has_super(t: &TestGame, id: ObjectId, s: Supertype) -> bool {
    t.obj_now(id).chars.has_supertype(s)
}

// ---------------------------------------------------------------------------------------
// Creatures that become something else
// ---------------------------------------------------------------------------------------

#[test]
fn darksteel_mutation_and_reprobation_keep_supertypes_and_replace_other_types() {
    cr!("205.1a", "205.4a", "613.1d");
    ruling!(
        "Darksteel Mutation",
        "The creature will keep any supertypes it previously had. Notably, if Darksteel Mutation is enchanting a legendary creature, that creature will continue to be legendary."
    );
    ruling!(
        "Darksteel Mutation",
        "The enchanted creature will be only an artifact and a creature, not any other card types. It will be only an Insect, not any other creature types."
    );
    ruling!(
        "Reprobation",
        "The enchanted creature retains any supertypes it had, such as legendary or snow."
    );
    supported("Darksteel Mutation");
    supported("Reprobation");
    // A legendary creature stays legendary.
    for aura in ["Darksteel Mutation", "Reprobation"] {
        let mut t = TestGame::new(2);
        let isamaru = t.battlefield(P1, "Isamaru, Hound of Konda");
        attach_new(&mut t, P0, aura, isamaru);
        assert!(has_super(&t, isamaru, Supertype::Legendary), "{aura}");
        assert_eq!(t.pt(isamaru), (0, 1), "{aura}");
    }
    // Dryad Arbor (Land Creature — Forest Dryad): only an artifact creature Insect.
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P1, "Dryad Arbor");
    attach_new(&mut t, P0, "Darksteel Mutation", arbor);
    assert_eq!(types(&t, arbor), vec![CardType::Artifact, CardType::Creature]);
    assert_eq!(subtypes(&t, arbor), vec!["Insect".to_string()]);
    assert_eq!(mana_abilities(&t, arbor), "");
}

#[test]
fn permanents_turned_into_something_else_keep_supertypes_name_and_mana_cost() {
    cr!("205.1a", "205.4a", "202.3", "111.1");
    ruling!(
        "Imprisoned in the Moon",
        "The permanent will keep any supertypes it previously had. Notably, if Imprisoned in the Moon is enchanting a legendary permanent, that permanent will continue to be legendary."
    );
    ruling!(
        "Minimus Containment",
        "The enchanted permanent still retains its name, colors, mana cost, and mana value. It isn't a token unless it already was one."
    );
    ruling!(
        "Sugar Coat",
        "The enchanted permanent still retains its name, mana cost, and mana value. It isn't a token unless it already was one."
    );
    supported("Imprisoned in the Moon");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P1, "Isamaru, Hound of Konda");
    attach_new(&mut t, P0, "Imprisoned in the Moon", isamaru);
    assert!(has_super(&t, isamaru, Supertype::Legendary));
    assert_eq!(types(&t, isamaru), vec![CardType::Land]);

    supported("Minimus Containment");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Minimus Containment", giant);
    let o = t.obj_now(giant);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.chars.colors.contains(Color::Red));
    assert_eq!(o.chars.mana_value(), 4);
    assert!(!o.is_token());
    assert!(o.chars.has_subtype("Treasure") && !o.is(CardType::Creature));

    supported("Sugar Coat");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Sugar Coat", giant);
    let o = t.obj_now(giant);
    assert_eq!(o.chars.name, "Hill Giant");
    assert!(o.chars.colors.is_colorless());
    assert_eq!(o.chars.mana_value(), 4);
    assert!(!o.is_token());
    assert!(o.chars.has_subtype("Food") && !o.is(CardType::Creature));
}

#[test]
fn one_with_the_stars_keeps_abilities_and_power_references_use_zero() {
    cr!("208.3", "613.1d");
    ruling!(
        "One with the Stars",
        "The enchanted permanent keeps its abilities but has no power or toughness. If one of its abilities refers to them, it uses 0."
    );
    supported("One with the Stars");
    supported("Viridian Joiner");
    let mut t = TestGame::new(2);
    let joiner = t.battlefield(P0, "Viridian Joiner");
    attach_new(&mut t, P1, "One with the Stars", joiner);
    let o = t.obj_now(joiner);
    assert_eq!(o.chars.power, None);
    assert_eq!(o.chars.toughness, None);
    assert!(!o.is(CardType::Creature));
    // "{T}: Add an amount of {G} equal to this creature's power." adds nothing.
    assert!(mana_abilities(&t, joiner).contains("{G}"));
    t.activate(P0, joiner, 0, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn song_of_the_dryads_keeps_supertypes_name_and_abilities_from_other_effects() {
    cr!("305.7", "613.1d", "613.1e", "613.1f");
    ruling!(
        "Song of the Dryads",
        "The enchanted permanent loses any card types, subtypes, and colors it previously had. It keeps any supertypes it had and its name remains unchanged. It gains “{T}: Add {G}” and loses all other abilities from its rules text. It will still have any abilities it gained from other effects."
    );
    supported("Song of the Dryads");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P1, "Isamaru, Hound of Konda");
    // Isamaru gains hexproof from another effect first.
    let defense = cast_spell(&mut t, P1, "Blossoming Defense", &[isamaru.into()]);
    t.resolve_all();
    assert!(!t.g.stack.contains(&defense));
    assert!(has_kw(&t, isamaru, mtg_engine::keywords::KeywordKind::Hexproof));
    attach_new(&mut t, P0, "Song of the Dryads", isamaru);
    let o = t.obj_now(isamaru);
    assert_eq!(o.chars.name, "Isamaru, Hound of Konda");
    assert!(o.chars.has_supertype(Supertype::Legendary));
    assert!(o.chars.colors.is_colorless());
    assert_eq!(types(&t, isamaru), vec![CardType::Land]);
    assert_eq!(subtypes(&t, isamaru), vec!["Forest".to_string()]);
    assert!(mana_abilities(&t, isamaru).contains("{G}"));
    assert!(has_kw(&t, isamaru, mtg_engine::keywords::KeywordKind::Hexproof));
}

#[test]
fn swift_reconfiguration_on_a_vehicle_adds_crew_5_to_its_own_crew() {
    cr!("702.122a", "613.1d", "613.1f");
    ruling!(
        "Swift Reconfiguration",
        "The permanent keeps all of its abilities. Notably, if Swift Reconfiguration enchants a Vehicle, that Vehicle gains crew 5 in addition to any crew abilities it already had."
    );
    supported("Swift Reconfiguration");
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    attach_new(&mut t, P0, "Swift Reconfiguration", copter);
    let crews: Vec<Option<i64>> = t
        .obj_now(copter)
        .chars
        .abilities
        .iter()
        .filter_map(|a| match &a.kind {
            AbilityKind::Keyword(k) if k.kind == mtg_engine::keywords::KeywordKind::Crew => {
                Some(k.n.map(|n| n as i64))
            }
            _ => None,
        })
        .collect();
    assert_eq!(crews, vec![Some(1), Some(5)]);
    // Crew 1 still works: a 2/2 crews it.
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(is_creature(&t, copter));
}

#[test]
fn awakened_awareness_toughness_becomes_1_before_its_trigger_resolves() {
    cr!("613.4b", "704.5g", "603.3");
    ruling!(
        "Awakened Awareness",
        "The enchanted creature's toughness becomes 1 before the triggered ability that puts counters on it goes on the stack. If it has damage equal to or greater than its toughness at that point, it will be destroyed before the triggered ability that puts counters on it resolves."
    );
    supported("Awakened Awareness");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, other, 1, giant);
    assert!(t.on_battlefield(giant));
    let aa = in_hand_with_mana_x(&mut t, P0, "Awakened Awareness", 3);
    t.cast(P0, aa).target(giant).x(3).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // An undamaged creature gets its counters: a 1/1 with three +1/+1 counters.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let aa = in_hand_with_mana_x(&mut t, P0, "Awakened Awareness", 3);
    t.cast(P0, aa).target(giant).x(3).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (4, 4));
}

fn in_hand_with_mana_x(t: &mut TestGame, p: PlayerId, name: &str, x: usize) -> ObjectId {
    let c = crate::r_s03_common::in_hand_with_mana(t, p, name);
    t.lands(p, "Wastes", x);
    c
}

// ---------------------------------------------------------------------------------------
// Lands
// ---------------------------------------------------------------------------------------

#[test]
fn land_type_setting_auras_replace_types_and_printed_abilities_only() {
    cr!("305.7", "205.1a", "613.1d", "613.1f");
    ruling!(
        "Lush Growth",
        "The enchanted land loses its existing land types and any abilities printed on it. It now has the ability to tap to add {R}, {G}, or {W}. Lush Growth doesn't change the enchanted land's name or whether it's legendary or basic."
    );
    ruling!(
        "Spreading Seas",
        "The enchanted land loses its existing land types and any abilities printed on it. It now has the land type Island and has the ability to tap to produce {U}. Spreading Seas doesn't change the enchanted land's name or whether it's legendary, basic, or snow."
    );
    ruling!(
        "Contaminated Ground",
        "The enchanted land loses its existing land types and any abilities printed on it. It now has the land type Swamp and has the ability to tap to add {B}. Contaminated Ground doesn't change the enchanted land's name or whether it's legendary, basic, or snow."
    );
    ruling!(
        "Evil Presence",
        "The enchanted land loses its existing land types and any abilities printed on it. It now has the land type Swamp and has the ability “{T}: Add {B}.” Evil Presence doesn’t change the enchanted land’s name or whether it’s legendary, basic, or snow."
    );
    for (aura, new_types, mana) in [
        (
            "Lush Growth",
            &["Mountain", "Forest", "Plains"][..],
            &["{R}", "{G}", "{W}"][..],
        ),
        ("Spreading Seas", &["Island"][..], &["{U}"][..]),
        ("Contaminated Ground", &["Swamp"][..], &["{B}"][..]),
        ("Evil Presence", &["Swamp"][..], &["{B}"][..]),
    ] {
        supported(aura);
        // A snow basic land: keeps its name, basic and snow.
        let mut t = TestGame::new(2);
        let land = t.battlefield(P1, "Snow-Covered Island");
        attach_new(&mut t, P0, aura, land);
        let o = t.obj_now(land);
        assert_eq!(o.chars.name, "Snow-Covered Island", "{aura}");
        assert!(o.chars.has_supertype(Supertype::Basic), "{aura}");
        assert!(o.chars.has_supertype(Supertype::Snow), "{aura}");
        let mut st = subtypes(&t, land);
        st.sort();
        let mut want: Vec<String> = new_types.iter().map(|s| s.to_string()).collect();
        want.sort();
        assert_eq!(st, want, "{aura}");
        let m = mana_abilities(&t, land);
        for sym in mana {
            assert!(m.contains(sym), "{aura}: {m}");
        }
        if !new_types.contains(&"Island") {
            assert!(!m.contains("{U}"), "{aura}: {m}");
        }
        // A legendary land with a printed ability: it stays legendary and loses it.
        let mut t = TestGame::new(2);
        let shizo = t.battlefield(P1, "Shizo, Death's Storehouse");
        attach_new(&mut t, P0, aura, shizo);
        assert!(has_super(&t, shizo, Supertype::Legendary), "{aura}");
        assert_eq!(t.obj_now(shizo).chars.name, "Shizo, Death's Storehouse");
        assert_eq!(n_abilities(&t, shizo), mana.len(), "{aura}");
    }
}

#[test]
fn land_auras_that_add_without_replacing_keep_the_lands_types_and_abilities() {
    cr!("305.6", "305.7", "613.1d");
    ruling!(
        "Nylea's Presence",
        "The enchanted land will have the land types Plains, Island, Swamp, Mountain, and Forest. It will also have the mana ability of each basic land type (for example, Forests can tap to produce {G}). It still has its other subtypes and abilities."
    );
    ruling!(
        "Glittering Frost",
        "The enchanted land will retain any other supertypes it may have, such as basic or legendary. It will also retain any subtypes it may have, such as Forest or Island."
    );
    ruling!(
        "Urban Utopia",
        "The enchanted land won't lose any other abilities it had. It also won't gain or lose any land types."
    );
    ruling!(
        "Abundant Growth",
        "The enchanted land retains any other abilities it has. Activating one of the land's activated abilities won't cause the other(s) to be activated."
    );
    // Nylea's Presence on Urza's Tower.
    supported("Nylea's Presence");
    let mut t = TestGame::new(2);
    let tower = t.battlefield(P0, "Urza's Tower");
    let before = n_abilities(&t, tower);
    attach_new(&mut t, P0, "Nylea's Presence", tower);
    for st in [
        "Urza's", "Tower", "Plains", "Island", "Swamp", "Mountain", "Forest",
    ] {
        assert!(t.obj_now(tower).chars.has_subtype(st), "{st}");
    }
    let m = mana_abilities(&t, tower);
    for sym in ["{W}", "{U}", "{B}", "{R}", "{G}"] {
        assert!(m.contains(sym), "{m}");
    }
    assert_eq!(n_abilities(&t, tower), before + 5);

    // Glittering Frost: a basic Forest and a legendary land keep their supertypes and
    // subtypes, and become snow.
    supported("Glittering Frost");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    attach_new(&mut t, P0, "Glittering Frost", forest);
    assert!(has_super(&t, forest, Supertype::Basic));
    assert!(has_super(&t, forest, Supertype::Snow));
    assert_eq!(subtypes(&t, forest), vec!["Forest".to_string()]);
    let shizo = t.battlefield(P0, "Shizo, Death's Storehouse");
    attach_new(&mut t, P0, "Glittering Frost", shizo);
    assert!(has_super(&t, shizo, Supertype::Legendary));
    assert!(has_super(&t, shizo, Supertype::Snow));

    // Urban Utopia and Abundant Growth: Mishra's Factory keeps its abilities and has no
    // land types; activating the new mana ability adds one mana.
    for aura in ["Urban Utopia", "Abundant Growth"] {
        supported(aura);
        let mut t = TestGame::new(2);
        let factory = t.battlefield(P0, "Mishra's Factory");
        let before = n_abilities(&t, factory);
        attach_new(&mut t, P0, aura, factory);
        assert_eq!(n_abilities(&t, factory), before + 1, "{aura}");
        assert!(subtypes(&t, factory).is_empty(), "{aura}");
        assert!(activate_containing(&mut t, P0, factory, "any color").is_ok());
        assert_eq!(t.g.player(P0).mana_pool.total(), 1, "{aura}");
        assert!(!is_creature(&t, factory), "{aura}");
        // The Factory's own ability still works (the land is tapped, but its {1} cost
        // can be paid from the pool).
        assert!(
            activate_containing(&mut t, P0, factory, "becomes a 2/2").is_ok(),
            "{aura}"
        );
        t.resolve_all();
        assert!(is_creature(&t, factory), "{aura}");
    }
}

// ---------------------------------------------------------------------------------------
// Enchant restrictions
// ---------------------------------------------------------------------------------------

#[test]
fn suppression_bonds_can_enchant_any_nonland_permanent() {
    cr!("303.4a", "702.5a", "115.1b");
    ruling!(
        "Suppression Bonds",
        "Suppression Bonds can enchant any nonland permanent, not just a creature."
    );
    for target in ["Bonesplitter", "Jace Beleren", "Glorious Anthem"] {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P1, target);
        assert!(cast_aura(&mut t, P0, "Suppression Bonds", x).is_some(), "{target}");
        assert_eq!(attached_to(&t, t.named_on_battlefield("Suppression Bonds")[0]), Some(x.into()));
    }
    let mut t = TestGame::new(2);
    let land = t.battlefield(P1, "Forest");
    assert!(cast_aura(&mut t, P0, "Suppression Bonds", land).is_none());
}

#[test]
fn sky_tether_enchants_creatures_without_flying_which_can_gain_it_later() {
    cr!("613.1f", "613.7", "702.9a");
    ruling!(
        "Sky Tether",
        "Sky Tether can enchant a creature that never had flying at all."
    );
    ruling!(
        "Sky Tether",
        "The enchanted creature can later gain flying from another effect."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(cast_aura(&mut t, P0, "Sky Tether", bears).is_some());
    assert!(has_kw(&t, bears, mtg_engine::keywords::KeywordKind::Defender));
    cast_spell(&mut t, P1, "Jump", &[bears.into()]);
    t.resolve_all();
    assert!(has_kw(&t, bears, mtg_engine::keywords::KeywordKind::Flying));
}

#[test]
fn domineer_needs_an_artifact_creature_and_falls_off_when_it_stops_being_one() {
    cr!("303.4a", "303.4d", "704.5m", "613.2");
    ruling!(
        "Domineer",
        "The enchanted permanent must be both an artifact and a creature. Domineer can be cast only on an artifact creature. Domineer “falls off” and is put into the graveyard if the artifact creature it’s enchanting stops being an artifact or stops being a creature."
    );
    // Not on a non-artifact creature or a noncreature artifact.
    for target in ["Grizzly Bears", "Bonesplitter"] {
        let mut t = TestGame::new(2);
        let x = t.battlefield(P1, target);
        assert!(cast_aura(&mut t, P0, "Domineer", x).is_none(), "{target}");
    }
    // On P1's Ornithopter: P0 controls it until it stops being a creature.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    assert!(cast_aura(&mut t, P0, "Domineer", thopter).is_some());
    assert_eq!(t.obj_now(thopter).controller, P0);
    attach_new(&mut t, P1, "Minimus Containment", thopter);
    t.settle();
    assert!(t.in_graveyard(P0, "Domineer"));
    assert_eq!(t.obj_now(thopter).controller, P1);
}

#[test]
fn spreading_algae_enchants_only_a_swamp_and_falls_off_when_it_isnt_one() {
    cr!("303.4a", "303.4d", "704.5m");
    ruling!(
        "Spreading Algae",
        "This card now has Enchant Swamp, which works exactly like any other Enchant ability. This means it can only be cast targeting a Swamp, and it will be put into its owner's graveyard if the permanent it's attached to ever stops being a Swamp."
    );
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    assert!(cast_aura(&mut t, P0, "Spreading Algae", forest).is_none());
    let mut t = TestGame::new(2);
    let swamp = t.battlefield(P1, "Swamp");
    let algae = cast_aura(&mut t, P0, "Spreading Algae", swamp).unwrap();
    assert_eq!(attached_to(&t, algae), Some(swamp.into()));
    // Spreading Seas makes it an Island: Algae goes to the graveyard (and its last ability
    // returns it to its owner's hand).
    attach_new(&mut t, P0, "Spreading Seas", swamp);
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(swamp));
    assert!(!t.on_battlefield(algae));
    assert!(t.in_hand(P0, "Spreading Algae"));
}
