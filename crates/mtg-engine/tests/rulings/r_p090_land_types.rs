//! Rulings batch P090 — land conversion: a land that becomes a basic land type loses its
//! other land types and the abilities from its rules text and gains the type's mana
//! ability (CR 305.7, 205.1a); "in addition to its other types" keeps them (CR 205.1b,
//! 305.6). Its name, supertypes and other card types don't change.

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use crate::r_s20_common::tap_for_mana;
use crate::r_s35_common::pick_options;
use mtg_engine::ability::AbilityKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::{CardType, Supertype};
use mtg_engine::*;

/// The texts of the mana abilities of the permanent now.
fn mana_abilities(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
        .map(|a| a.text.to_string())
        .collect()
}

fn activated(t: &mut TestGame, id: ObjectId) -> usize {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Activated(_)))
        .count()
}

const BASICS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];

fn land_types(t: &mut TestGame, id: ObjectId) -> Vec<&'static str> {
    t.g.recompute();
    BASICS
        .into_iter()
        .filter(|b| t.obj_now(id).chars.has_subtype(b))
        .collect()
}

/// Checks that `land` (originally `name`) is now exactly a Swamp: loses its other
/// abilities and land types, taps for {B}, keeps its name, supertypes and card types.
fn check_now_swamp(
    t: &mut TestGame,
    land: ObjectId,
    name: &str,
    sup: &[Supertype],
    creature: bool,
) {
    assert_eq!(land_types(t, land), ["Swamp"], "{name}");
    let o = t.obj_now(land);
    assert_eq!(o.chars.name, name);
    for s in Supertype::ALL {
        assert_eq!(o.chars.has_supertype(s), sup.contains(&s), "{name} {s:?}");
    }
    assert!(o.chars.is(CardType::Land));
    assert_eq!(o.chars.is(CardType::Creature), creature, "{name}");
    assert_eq!(
        activated(t, land),
        1,
        "{name}: only the Swamp's mana ability: {:?}",
        t.obj_now(land)
            .chars
            .abilities
            .iter()
            .map(|a| a.text.to_string())
            .collect::<Vec<_>>()
    );
    assert!(
        tap_for_mana(t, P0, land, "{B}"),
        "{name}: tapped {} {}",
        t.obj_now(land).tapped,
        t.dump_log()
    );
}

#[test]
fn becoming_a_basic_land_type_replaces_types_and_abilities_but_not_name_or_supertypes() {
    cr!("305.7", "205.1a", "613.1d", "613.1f");
    ruling!(
        "Convincing Mirage",
        "The affected land loses its existing land types and any abilities printed on it. It becomes the chosen basic land type and now has the ability to tap to add one mana of the appropriate color. Convincing Mirage's ability doesn't change the affected land's name or whether it's legendary or basic."
    );
    ruling!(
        "Grixis Illusionist",
        "The affected land loses its existing land types and any abilities printed on it. It becomes the chosen basic land type and now has the ability to tap to add one mana of the appropriate color. Grixis Illusionist's ability doesn't change the affected land's name or whether it's legendary or basic."
    );
    ruling!(
        "Unstable Frontier",
        "The affected land loses its existing land types and any abilities printed on it. It becomes the chosen basic land type and now has the ability to tap to add one mana of the appropriate color. Unstable Frontier's ability doesn't change the affected land's name or whether it's legendary or basic."
    );
    // Pendelhaven (legendary, two abilities) and a Snow-Covered Forest (basic, snow).
    let targets: [(&str, &[Supertype]); 2] = [
        ("Pendelhaven", &[Supertype::Legendary]),
        ("Snow-Covered Forest", &[Supertype::Basic, Supertype::Snow]),
    ];
    for (name, sup) in targets {
        // Convincing Mirage enchanting it, Swamp chosen.
        supported("Convincing Mirage");
        let mut t = TestGame::new(2);
        let land = t.battlefield(P0, name);
        pick_options(&mut t, P0, &["Swamp"]);
        add_mana(&mut t, P0, ManaType::U, 2);
        let m = t.hand(P0, "Convincing Mirage");
        t.cast(P0, m).target(land).go();
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Convincing Mirage").len(), 1);
        check_now_swamp(&mut t, land, name, sup, false);

        // Grixis Illusionist and Unstable Frontier: "{T}: Target land you control becomes
        // the basic land type of your choice until end of turn."
        for source in ["Grixis Illusionist", "Unstable Frontier"] {
            supported(source);
            let mut t = TestGame::new(2);
            let land = t.battlefield(P0, name);
            let s = t.battlefield(P0, source);
            pick_options(&mut t, P0, &["Swamp"]);
            let idx = usize::from(source == "Unstable Frontier");
            t.activate(P0, s, idx, &[Entity::Object(land)])
                .expect("activate");
            t.resolve_all();
            check_now_swamp(&mut t, land, name, sup, false);
        }
    }
}

#[test]
fn tide_shaper_makes_an_island_keeping_other_types() {
    cr!("305.7", "205.1a", "611.2b");
    ruling!(
        "Tide Shaper",
        "A land that becomes an Island gains \"{T}: Add {U}\" and loses all other abilities. It loses any other land types, but it doesn't lose any types or supertypes, such as creature, legendary, or snow."
    );
    ruling!(
        "Tide Shaper",
        "If Tide Shaper leaves the battlefield before its triggered ability resolves, the target land will not become an Island."
    );
    supported("Tide Shaper");
    supported("Dryad Arbor");
    // (land, supertypes, a creature)
    let cases: [(&str, &[Supertype], bool); 3] = [
        ("Dryad Arbor", &[], true),
        ("Pendelhaven", &[Supertype::Legendary], false),
        (
            "Snow-Covered Forest",
            &[Supertype::Basic, Supertype::Snow],
            false,
        ),
    ];
    for (name, sup, creature) in cases {
        let mut t = TestGame::new(2);
        let land = t.battlefield(P0, name);
        add_mana(&mut t, P0, ManaType::U, 2);
        let ts = t.hand(P0, "Tide Shaper");
        t.answer_targets(P0, &[Entity::Object(land)]);
        t.cast(P0, ts).kicked(true).go();
        t.resolve_all();
        assert_eq!(land_types(&mut t, land), ["Island"], "{name}");
        let o = t.obj_now(land);
        assert_eq!(o.chars.name, name);
        for s in Supertype::ALL {
            assert_eq!(o.chars.has_supertype(s), sup.contains(&s), "{name} {s:?}");
        }
        assert_eq!(o.chars.is(CardType::Creature), creature, "{name}");
        assert_eq!(activated(&mut t, land), 1, "{name}");
        assert!(tap_for_mana(&mut t, P0, land, "{U}"), "{name}");
    }
    // Tide Shaper leaves before its ability resolves: no change.
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    add_mana(&mut t, P0, ManaType::U, 2);
    let ts = t.hand(P0, "Tide Shaper");
    t.answer_targets(P0, &[Entity::Object(land)]);
    t.cast(P0, ts).kicked(true).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1, "the trigger");
    let shaper = t.named_on_battlefield("Tide Shaper")[0];
    crate::r_s02_common::destroy(&mut t, shaper);
    t.resolve_all();
    assert_eq!(land_types(&mut t, land), ["Forest"]);
}

#[test]
fn gaining_basic_land_types_keeps_name_supertypes_and_abilities() {
    cr!("305.6", "205.1b", "613.1d");
    ruling!(
        "Navigator's Compass",
        "Gaining a basic land type causes the target land to gain the corresponding mana ability. Because the new basic land type is “in addition to” its other types, it keeps the abilities it had previously."
    );
    ruling!(
        "Nylea's Presence",
        "Giving a land extra basic land types doesn't change its name or whether it's legendary or basic."
    );
    ruling!(
        "Energybending",
        "The basic land types are Plains, Island, Swamp, Mountain, and Forest. Gaining a basic land type also allows the land to tap for the associated color of mana, so a land with all basic land types can tap for any color of mana."
    );
    // Navigator's Compass: Pendelhaven becomes a Swamp too.
    supported("Navigator's Compass");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Pendelhaven");
    let c = t.battlefield(P0, "Navigator's Compass");
    pick_options(&mut t, P0, &["Swamp"]);
    t.activate(P0, c, 0, &[Entity::Object(land)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(land_types(&mut t, land), ["Swamp"]);
    assert_eq!(
        activated(&mut t, land),
        3,
        "{:?}",
        mana_abilities(&mut t, land)
    );
    assert!(t.obj_now(land).chars.has_supertype(Supertype::Legendary));
    assert!(tap_for_mana(&mut t, P0, land, "{B}"));

    // Nylea's Presence on Pendelhaven and on a basic Forest: names and supertypes stay.
    supported("Nylea's Presence");
    for (name, sup) in [
        ("Pendelhaven", Supertype::Legendary),
        ("Forest", Supertype::Basic),
    ] {
        let mut t = TestGame::new(2);
        let land = t.battlefield(P0, name);
        let aura = crate::r_s06_common::attach_new(&mut t, P0, "Nylea's Presence", land);
        assert!(t.on_battlefield(aura));
        assert_eq!(land_types(&mut t, land), BASICS);
        let o = t.obj_now(land);
        assert_eq!(o.chars.name, name);
        assert!(o.chars.has_supertype(sup), "{name}");
        assert_eq!(
            o.chars.has_supertype(Supertype::Legendary),
            name == "Pendelhaven"
        );
        assert_eq!(o.chars.has_supertype(Supertype::Basic), name == "Forest");
    }

    // Energybending: Wastes (no land type) taps for any color.
    supported("Energybending");
    for (i, color) in ["{W}", "{U}", "{B}", "{R}", "{G}"].into_iter().enumerate() {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, "Wastes");
        add_mana(&mut t, P0, ManaType::C, 2);
        let s = t.hand(P0, "Energybending");
        t.cast(P0, s).go();
        t.resolve_all();
        assert!(!t.obj_now(w).tapped);
        assert_eq!(land_types(&mut t, w), BASICS, "{i}");
        assert!(tap_for_mana(&mut t, P0, w, color), "{color}");
    }
}
