//! Rulings batch P192 — universal type changes: Conspiracy and Xenograft (creature
//! types), Armed with Proof (Clue Equipment), Blanket of Night, Urborg and Yavimaya
//! (each land is a Swamp/Forest), Rootpath Purifier (lands are basic), Terraformer,
//! Realmwright, Dryad of the Ilysian Grove and Prismatic Omen (land types and their
//! intrinsic mana abilities, CR 305.6–305.7), Encroaching Mycosynth (artifact Auras),
//! Ashes of the Fallen (types of cards in a graveyard) and Rude Awakening (locked-in sets
//! of affected objects, CR 611.2c).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_play_land, create_token, destroy};
use crate::r_s04_common::add_mana;
use crate::r_s06_common::{activate_containing, attached_to};
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::{legendary, name_now};
use crate::r_s29_common::{cast_and_resolve, choose_modes};
use crate::r_s32_common::{basic_now, subtypes_now};
use crate::r_s35_common::{creature_type_options, pick_options};
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn has_sub(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.has_subtype(s)
}

/// The mana abilities of the object now whose text contains `needle`.
fn mana_abilities_with(t: &TestGame, id: ObjectId, needle: &str) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| {
            matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability)
                && a.text.contains(needle)
        })
        .count()
}

// ---------------------------------------------------------------- Conspiracy, Xenograft

#[test]
fn conspiracy_doesnt_offer_legend_as_a_creature_type() {
    cr!("205.3m");
    ruling!(
        "Conspiracy",
        "\"Legend\" is no longer a creature type, and may not be chosen."
    );
    supported("Conspiracy");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Conspiracy");
    let offered = creature_type_options(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].iter().any(|o| o == "Elf"));
    assert!(!offered[0].iter().any(|o| o == "Legend"));
}

#[test]
fn conspiracy_replaces_every_creature_type_in_every_zone() {
    cr!("205.1a", "613.1d");
    ruling!(
        "Conspiracy",
        "Affected spells, cards, and creatures lose all of their other creature types."
    );
    let mut t = TestGame::new(2);
    let on_bf = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Grizzly Bears");
    let in_gy = t.graveyard(P0, "Raging Goblin");
    let in_lib = t.library_top(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Raging Goblin");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Conspiracy");
    add_mana(&mut t, P0, ManaType::R, 1);
    let spell = t.cast(P0, spell).go();
    t.g.recompute();
    for id in [on_bf, in_hand, in_gy, in_lib, spell] {
        assert_eq!(subtypes_now(&t, id), vec!["Elf"], "{id:?}");
        assert!(t.obj_now(id).is(CardType::Creature));
    }
    assert_eq!(subtypes_now(&t, theirs), vec!["Bear"]);
}

#[test]
fn conspiracy_doesnt_change_creature_types_in_rules_text() {
    cr!("613.1d", "205.3m");
    ruling!(
        "Conspiracy",
        "It does not replace any use of creature types in card text."
    );
    // Goblin King: "Other Goblins get +1/+1 and have mountainwalk."
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Conspiracy");
    t.battlefield(P0, "Goblin King");
    let elf = t.battlefield(P0, "Grizzly Bears");
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.g.recompute();
    // Goblin King is an Elf now but still pumps Goblins, not Elves.
    assert!(has_sub(&t, elf, "Elf"));
    assert_eq!(t.pt(elf), (2, 2));
    assert_eq!(t.pt(goblin), (2, 2));
}

#[test]
fn xenograft_affects_only_creatures_on_the_battlefield() {
    cr!("613.1d", "611.3a");
    ruling!(
        "Xenograft",
        "Creature cards not on the battlefield and creature spells are not affected."
    );
    supported("Xenograft");
    let mut t = TestGame::new(2);
    let on_bf = t.battlefield(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Grizzly Bears");
    let in_gy = t.graveyard(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Raging Goblin");
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Xenograft");
    add_mana(&mut t, P0, ManaType::R, 1);
    let spell = t.cast(P0, spell).go();
    t.g.recompute();
    assert!(has_sub(&t, on_bf, "Elf") && has_sub(&t, on_bf, "Bear"));
    for id in [in_hand, in_gy, spell] {
        assert!(!has_sub(&t, id, "Elf"), "{id:?}");
    }
    // Once the spell resolves, the Goblin is an Elf Goblin.
    t.resolve_all();
    assert!(has_sub(&t, spell, "Elf") && has_sub(&t, spell, "Goblin"));
}

// ---------------------------------------------------------------- Armed with Proof

/// P0 controls Armed with Proof (its investigate trigger not used) and Red Herring (an
/// artifact creature Clue).
fn herring_with_proof(t: &mut TestGame, copies: usize) -> ObjectId {
    for _ in 0..copies {
        t.battlefield(P0, "Armed with Proof");
    }
    t.battlefield(P0, "Red Herring")
}

#[test]
fn armed_with_proof_clue_creature_cant_become_attached() {
    cr!("301.5c", "702.6a");
    ruling!(
        "Armed with Proof",
        "An Equipment without reconfigure that's also a creature can't be attached to anything."
    );
    supported("Armed with Proof");
    supported("Red Herring");
    let mut t = TestGame::new(2);
    let herring = herring_with_proof(&mut t, 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert!(t.obj_now(herring).chars.has_subtype("Equipment"));
    assert!(t.obj_now(herring).is(CardType::Creature));
    add_mana(&mut t, P0, ManaType::C, 2);
    // The equip ability can be activated, but it doesn't attach.
    let ok = activate_containing(&mut t, P0, herring, "Equip").is_ok() || {
        t.answer_targets(P0, &[obj(bears)]);
        activate_containing(&mut t, P0, herring, "Equip").is_ok()
    };
    assert!(ok, "the equip ability couldn't be activated");
    t.resolve_all();
    assert_eq!(attached_to(&t, herring), None);
    assert_eq!(t.pt(bears), (2, 2));
    // A noncreature Clue can be attached.
    let clue = create_token(&mut t, P0, "Clue");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, clue, "Equip").unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, clue), Some(obj(bears)));
    assert_eq!(t.pt(bears), (4, 2));
}

#[test]
fn armed_with_proof_leaving_unattaches_the_clues() {
    cr!("704.5p", "611.3a");
    ruling!(
        "Armed with Proof",
        "Clues stop being Equipment if Armed with Proof leaves the battlefield. If any of them are attached to creatures at that time, they become unattached."
    );
    let mut t = TestGame::new(2);
    let proof = t.battlefield(P0, "Armed with Proof");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let clue = create_token(&mut t, P0, "Clue");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, clue, "Equip").unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, clue), Some(obj(bears)));
    destroy(&mut t, proof);
    t.g.recompute();
    assert!(!t.obj_now(clue).chars.has_subtype("Equipment"));
    assert_eq!(attached_to(&t, clue), None);
    assert!(t.on_battlefield(clue));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn two_armed_with_proofs_give_plus_four_and_equip_still_costs_two() {
    cr!("613.4c", "702.6a");
    ruling!(
        "Armed with Proof",
        "If you control more than one Armed with Proof, creatures equipped with Clue Equipment will get +2/+0 for each Armed with Proof you control. The equip cost is still {2}."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Armed with Proof");
    t.battlefield(P0, "Armed with Proof");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let clue = create_token(&mut t, P0, "Clue");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(&mut t, P0, clue, "Equip").unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert_eq!(attached_to(&t, clue), Some(obj(bears)));
    assert_eq!(t.pt(bears), (6, 2));
}

// ---------------------------------------------------------------- Blanket of Night

#[test]
fn blanket_of_night_doesnt_double_count_a_real_swamp() {
    cr!("305.6", "613.1d");
    ruling!(
        "Blanket of Night",
        "Anything which counts the number of Swamps on the battlefield will not double count an actual Swamp card"
    );
    ruling!(
        "Blanket of Night",
        "More than one Blanket of Night has no meaningful additional effect."
    );
    supported("Nightmare");
    // Nightmare: P/T each equal to the number of Swamps you control.
    for blankets in [0, 1, 2] {
        let mut t = TestGame::new(2);
        for _ in 0..blankets {
            t.battlefield(P0, "Blanket of Night");
        }
        let swamp = t.battlefield(P0, "Swamp");
        let forest = t.battlefield(P0, "Forest");
        let nightmare = t.battlefield(P0, "Nightmare");
        t.g.recompute();
        let want = if blankets == 0 { 1 } else { 2 };
        assert_eq!(t.pt(nightmare), (want, want), "{blankets} blankets");
        // One Swamp subtype and one {B} ability each, however many Blankets.
        assert_eq!(mana_abilities_with(&t, swamp, "{B}"), 1);
        let forest_b = if blankets == 0 { 0 } else { 1 };
        assert_eq!(mana_abilities_with(&t, forest, "{B}"), forest_b);
        assert_eq!(
            subtypes_now(&t, forest)
                .iter()
                .filter(|s| *s == "Swamp")
                .count(),
            forest_b
        );
    }
}

#[test]
fn blanket_of_night_makes_a_snow_land_a_snow_swamp() {
    cr!("205.4g", "305.6");
    ruling!(
        "Blanket of Night",
        "If this effect is applied to a snow land, the land is now a snow Swamp and not just a regular Swamp."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blanket of Night");
    let snow = t.battlefield(P0, "Snow-Covered Forest");
    t.g.recompute();
    let o = t.obj_now(snow);
    assert!(o.chars.supertypes.contains(Supertype::Snow));
    assert!(has_sub(&t, snow, "Swamp") && has_sub(&t, snow, "Forest"));
    // Its {B} comes from a snow source.
    assert!(tap_for_mana(&mut t, P0, snow, "{B}"));
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::B), 1);
}

#[test]
fn blanket_of_night_affects_lands_that_dont_produce_mana() {
    cr!("305.6", "305.7");
    ruling!(
        "Blanket of Night",
        "It can affect lands which do not produce mana."
    );
    supported("Maze of Ith");
    let mut t = TestGame::new(2);
    let maze = t.battlefield(P0, "Maze of Ith");
    t.g.recompute();
    assert_eq!(mana_abilities_with(&t, maze, "Add"), 0);
    t.battlefield(P0, "Blanket of Night");
    t.g.recompute();
    assert!(has_sub(&t, maze, "Swamp"));
    assert!(tap_for_mana(&mut t, P0, maze, "{B}"));
}

// ---------------------------------------------------------------- Urborg, Yavimaya

#[test]
fn urborg_and_yavimaya_dont_change_land_cards_off_the_battlefield() {
    cr!("611.3a", "305.7");
    ruling!(
        "Urborg, Tomb of Yawgmoth",
        "Land cards not on the battlefield aren't Swamps while Urborg is on the battlefield."
    );
    ruling!(
        "Yavimaya, Cradle of Growth",
        "Land cards not on the battlefield aren't Forests while Yavimaya is on the battlefield."
    );
    for (land, ty) in [
        ("Urborg, Tomb of Yawgmoth", "Swamp"),
        ("Yavimaya, Cradle of Growth", "Forest"),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, land);
        let on_bf = t.battlefield(P0, "Plains");
        let in_hand = t.hand(P0, "Plains");
        let in_lib = t.library_top(P0, "Plains");
        let in_gy = t.graveyard(P1, "Plains");
        t.g.recompute();
        assert!(has_sub(&t, on_bf, ty), "{land}");
        for id in [in_hand, in_lib, in_gy] {
            assert!(!has_sub(&t, id, ty), "{land}");
        }
        // Played, it becomes one.
        assert!(can_play_land(&mut t, P0, in_hand));
        t.play_land(P0, in_hand).unwrap();
        t.g.recompute();
        assert!(has_sub(&t, in_hand, ty), "{land}");
    }
}

#[test]
fn urborg_that_lost_its_abilities_to_magus_of_the_moon_makes_no_swamps() {
    cr!("613.8a", "613.8b", "613.1d");
    ruling!(
        "Urborg, Tomb of Yawgmoth",
        "it won't turn lands into Swamps, no matter in what order those effects started to apply."
    );
    supported("Magus of the Moon");
    // Magus of the Moon: "Nonbasic lands are Mountains."
    for magus_first in [true, false] {
        let mut t = TestGame::new(2);
        if magus_first {
            t.battlefield(P1, "Magus of the Moon");
        }
        let urborg = t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
        if !magus_first {
            t.battlefield(P1, "Magus of the Moon");
        }
        let forest = t.battlefield(P0, "Forest");
        t.g.recompute();
        assert_eq!(subtypes_now(&t, urborg), vec!["Mountain"], "{magus_first}");
        assert_eq!(subtypes_now(&t, forest), vec!["Forest"], "{magus_first}");
        assert_eq!(mana_abilities_with(&t, forest, "{B}"), 0);
        assert_eq!(mana_abilities_with(&t, urborg, "{B}"), 0);
    }
    // Control: without Magus, the Forest is a Swamp too.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
    let forest = t.battlefield(P0, "Forest");
    t.g.recompute();
    assert!(has_sub(&t, forest, "Swamp"));
}

// ---------------------------------------------------------------- Rootpath Purifier

#[test]
fn rootpath_purifier_adds_basic_but_nothing_else() {
    cr!("205.4c", "613.1d");
    ruling!(
        "Rootpath Purifier",
        "Being basic doesn’t grant any abilities to a land that it didn’t already have, and doesn’t remove any card types, subtypes, or supertypes."
    );
    supported("Rootpath Purifier");
    let mut t = TestGame::new(2);
    let urborg = t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
    let in_lib = t.library_top(P0, "Urborg, Tomb of Yawgmoth");
    let in_hand = t.hand(P0, "Urborg, Tomb of Yawgmoth");
    let theirs = t.battlefield(P1, "Azorius Guildgate");
    t.g.recompute();
    let abilities_before = t.obj_now(urborg).chars.abilities.len();
    assert!(!basic_now(&t, urborg) && !basic_now(&t, in_lib));
    t.battlefield(P0, "Rootpath Purifier");
    t.g.recompute();
    for id in [urborg, in_lib] {
        assert!(basic_now(&t, id) && legendary(&t, id), "{id:?}");
        assert!(t.obj_now(id).is(CardType::Land));
    }
    assert_eq!(t.obj_now(urborg).chars.abilities.len(), abilities_before);
    // Not in other zones, and not an opponent's lands.
    assert!(!basic_now(&t, in_hand));
    assert!(!basic_now(&t, theirs));
}

#[test]
fn rootpath_purifier_lands_escape_blood_moon() {
    cr!("613.8a", "613.8b", "613.1d", "205.4c");
    ruling!(
        "Rootpath Purifier",
        "Lands that become basic are no longer nonbasic lands. This may change what effects can apply to them."
    );
    supported("Blood Moon");
    for moon_first in [true, false] {
        let mut t = TestGame::new(2);
        if moon_first {
            t.battlefield(P1, "Blood Moon");
        }
        let gate = t.battlefield(P0, "Azorius Guildgate");
        let theirs = t.battlefield(P1, "Azorius Guildgate");
        if !moon_first {
            t.battlefield(P1, "Blood Moon");
        }
        t.g.recompute();
        assert_eq!(subtypes_now(&t, gate), vec!["Mountain"]);
        t.battlefield(P0, "Rootpath Purifier");
        t.g.recompute();
        assert_eq!(
            subtypes_now(&t, gate),
            vec!["Gate"],
            "moon first: {moon_first}"
        );
        assert!(tap_for_mana(&mut t, P0, gate, "{W}"));
        assert_eq!(subtypes_now(&t, theirs), vec!["Mountain"]);
    }
}

// ---------------------------------------------------------------- Terraformer, Realmwright

#[test]
fn terraformer_doesnt_change_name_legendary_or_basic() {
    cr!("305.7", "613.1d");
    ruling!(
        "Terraformer",
        "Changing a land’s type doesn’t change its name or whether it’s legendary or basic."
    );
    let mut t = TestGame::new(2);
    let terraformer = t.battlefield(P0, "Terraformer");
    let urborg = t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
    let forest = t.battlefield(P0, "Forest");
    add_mana(&mut t, P0, ManaType::C, 1);
    pick_options(&mut t, P0, &["Island"]);
    activate_containing(&mut t, P0, terraformer, "basic land type").unwrap();
    t.resolve_all();
    assert_eq!(subtypes_now(&t, forest), vec!["Island"]);
    assert_eq!(subtypes_now(&t, urborg), vec!["Island"]);
    assert_eq!(name_now(&t, forest), "Forest");
    assert!(basic_now(&t, forest) && !legendary(&t, forest));
    assert_eq!(name_now(&t, urborg), "Urborg, Tomb of Yawgmoth");
    assert!(legendary(&t, urborg) && !basic_now(&t, urborg));
}

#[test]
fn realmwright_lands_tap_for_the_chosen_color_and_keep_their_abilities() {
    cr!("305.6", "305.7");
    ruling!(
        "Realmwright",
        "Lands you control will have the ability to tap for the color of mana corresponding to the chosen type. They’ll also retain any other abilities they have."
    );
    let mut t = TestGame::new(2);
    pick_options(&mut t, P0, &["Swamp"]);
    t.enter(P0, "Realmwright");
    let gate = t.battlefield(P0, "Azorius Guildgate");
    let gate2 = t.battlefield(P0, "Azorius Guildgate");
    let theirs = t.battlefield(P1, "Azorius Guildgate");
    t.g.recompute();
    assert!(tap_for_mana(&mut t, P0, gate, "{B}"));
    assert!(tap_for_mana(&mut t, P0, gate2, "{W}"));
    assert_eq!(mana_abilities_with(&t, theirs, "{B}"), 0);
}

// ---------------------------------------------------------------- Dryad, Prismatic Omen

#[test]
fn every_basic_land_type_adds_the_five_mana_abilities_and_keeps_the_rest() {
    cr!("305.6", "305.7", "205.4c");
    ruling!(
        "Prismatic Omen",
        "Each land you control will have the land types Plains, Island, Swamp, Mountain, and Forest. They’ll also have the mana ability of each basic land type (for example, Forests can tap to produce {G}). They’ll still have their other subtypes and abilities."
    );
    ruling!(
        "Dryad of the Ilysian Grove",
        "Each land you control will have the land types Plains, Island, Swamp, Mountain, and Forest. They’ll also have the mana ability of each basic land type (for example, Forests have “{T}: Add {G}.”). They’ll still have their other subtypes and abilities."
    );
    ruling!(
        "Prismatic Omen",
        "Giving a land extra basic land types doesn’t change its name or whether it’s legendary or basic."
    );
    ruling!(
        "Dryad of the Ilysian Grove",
        "Giving a land additional basic land types doesn’t change its name or whether it’s legendary or basic."
    );
    for source in ["Prismatic Omen", "Dryad of the Ilysian Grove"] {
        supported(source);
        let mut t = TestGame::new(2);
        t.battlefield(P0, source);
        let gate = t.battlefield(P0, "Azorius Guildgate");
        let urborg = t.battlefield(P0, "Urborg, Tomb of Yawgmoth");
        let forest = t.battlefield(P0, "Forest");
        let theirs = t.battlefield(P1, "Azorius Guildgate");
        t.g.recompute();
        let mut subs = subtypes_now(&t, gate);
        subs.sort();
        assert_eq!(
            subs,
            vec!["Forest", "Gate", "Island", "Mountain", "Plains", "Swamp"],
            "{source}"
        );
        // Its own "{T}: Add {W} or {U}." plus one intrinsic ability per basic land type.
        for (sym, n) in [("{W}", 2), ("{U}", 2), ("{B}", 1), ("{R}", 1), ("{G}", 1)] {
            assert_eq!(mana_abilities_with(&t, gate, sym), n, "{source}: {sym}");
        }
        assert_eq!(mana_abilities_with(&t, theirs, "{G}"), 0, "{source}");
        assert!(tap_for_mana(&mut t, P0, gate, "{G}"), "{source}");
        // Its own "{T}: Add {W} or {U}." is still there (on another Guildgate-like land).
        assert!(t
            .obj_now(gate)
            .chars
            .abilities
            .iter()
            .any(|a| a.text.contains("{W} or {U}")));
        assert_eq!(name_now(&t, forest), "Forest");
        assert!(basic_now(&t, forest) && !legendary(&t, forest));
        assert!(legendary(&t, urborg) && !basic_now(&t, urborg));
        assert_eq!(name_now(&t, urborg), "Urborg, Tomb of Yawgmoth");
        assert!(!basic_now(&t, gate));
    }
}

#[test]
fn dryad_of_the_ilysian_grove_additional_land_plays_are_cumulative() {
    cr!("305.2", "305.2a");
    ruling!(
        "Dryad of the Ilysian Grove",
        "Dryad of the Ilysian Grove’s first ability is cumulative if you control more than one."
    );
    supported("Hugs, Grisly Guardian");
    // (lands, other source, number of lands that can be played)
    for (dryads, hugs, plays) in [(0, false, 1), (1, false, 2), (2, false, 3), (1, true, 3)] {
        let mut t = TestGame::new(2);
        for _ in 0..dryads {
            t.battlefield(P0, "Dryad of the Ilysian Grove");
        }
        if hugs {
            t.battlefield(P0, "Hugs, Grisly Guardian");
        }
        for i in 0..plays {
            let f = t.hand(P0, "Forest");
            assert!(can_play_land(&mut t, P0, f), "{dryads} {hugs}: play {i}");
            t.play_land(P0, f).unwrap();
        }
        let f = t.hand(P0, "Forest");
        assert!(!can_play_land(&mut t, P0, f), "{dryads} {hugs}: extra play");
    }
}

// ---------------------------------------------------------------- Encroaching Mycosynth

#[test]
fn encroaching_mycosynth_aura_animated_by_march_of_the_machines_falls_off() {
    cr!("303.4d", "704.5m", "704.5p");
    ruling!(
        "Encroaching Mycosynth",
        "An Aura that's also a creature can't enchant anything. It's unattached the next time state-based actions are checked, and then immediately put into its owner's graveyard as a second state-based action."
    );
    supported("Encroaching Mycosynth");
    supported("March of the Machines");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Encroaching Mycosynth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.hand(P0, "Holy Strength");
    add_mana(&mut t, P0, ManaType::W, 1);
    let aura = {
        let s = t.cast(P0, aura).target(bears).go();
        t.resolve_all();
        t.g.current(s)
    };
    t.g.recompute();
    // An artifact Aura: still attached, still giving +1/+2.
    assert!(t.obj_now(aura).is(CardType::Artifact));
    assert_eq!(attached_to(&t, aura), Some(obj(bears)));
    assert_eq!(t.pt(bears), (3, 4));
    // March of the Machines makes it a 1/1 artifact creature Aura (mana value 1).
    t.battlefield(P1, "March of the Machines");
    t.g.recompute();
    assert!(t.obj_now(aura).is(CardType::Creature));
    // First state-based action check: unattached, still on the battlefield.
    assert!(t.g.check_sbas());
    assert!(t.on_battlefield(aura));
    assert_eq!(attached_to(&t, aura), None);
    // Second check: put into its owner's graveyard.
    assert!(t.g.check_sbas());
    assert!(!t.on_battlefield(aura));
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert_eq!(t.pt(bears), (2, 2));
}

// ---------------------------------------------------------------- Ashes of the Fallen

#[test]
fn ashes_of_the_fallen_spirit_lets_soulshift_return_a_creature_that_died_with_it() {
    cr!("702.46a", "603.10a", "613.1d");
    ruling!(
        "Ashes of the Fallen",
        "The Spirit creature type is added as the cards enter the graveyard, just before you choose the target for the soulshift ability."
    );
    supported("Ashes of the Fallen");
    supported("Kami of Empty Graves");
    for ashes in [false, true] {
        let mut t = TestGame::new(2);
        if ashes {
            choose_creature_type(&mut t, P0, "Spirit");
            t.enter(P0, "Ashes of the Fallen");
        }
        // Kami of Empty Graves: soulshift 3.
        t.battlefield(P0, "Kami of Empty Graves");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let from = t.asked().len();
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_yes(P0, true);
        cast_and_resolve(&mut t, P0, "Wrath of God", &[]);
        let candidates: Vec<Vec<Entity>> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
                _ => None,
            })
            .collect();
        let offered = candidates.iter().flatten().any(|e| match e {
            Entity::Object(id) => t.g.obj(*id).chars.name == "Grizzly Bears",
            _ => false,
        });
        assert_eq!(offered, ashes, "with Ashes: {ashes}");
        assert_eq!(t.in_hand(P0, "Grizzly Bears"), ashes);
    }
}

// ---------------------------------------------------------------- Rude Awakening

#[test]
fn rude_awakening_doesnt_animate_lands_that_arrive_later() {
    cr!("611.2c");
    ruling!(
        "Rude Awakening",
        "Lands that enter the battlefield or otehrwise come under your control after Rude Awakening resolves won’t be affected."
    );
    supported("Rude Awakening");
    let mut t = TestGame::new(2);
    let early = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Forest");
    choose_modes(&mut t, P0, &[1]);
    t.answer(
        P0,
        DecisionKind::OptionalCost,
        mtg_engine::decision::Answer::Bool(false),
    );
    cast_and_resolve(&mut t, P0, "Rude Awakening", &[]);
    t.g.recompute();
    assert!(t.obj_now(early).is(CardType::Creature));
    assert_eq!(t.pt(early), (2, 2));
    let late = t.hand(P0, "Forest");
    t.play_land(P0, late).unwrap();
    crate::r_s06_common::give_control(&mut t, theirs, P0);
    t.g.recompute();
    assert!(!t.obj_now(late).is(CardType::Creature));
    assert!(!t.obj_now(theirs).is(CardType::Creature));
}
