//! Rulings batch P193 — universal type changes (Arcane Adaptation, Maskwood Nexus,
//! Conspiracy, Standardize, Enchanted Evening, Terraformer, Realmwright, Blanket of Night,
//! Urborg, Tomb of Yawgmoth, Yavimaya, Cradle of Growth, Toph, the First Metalbender).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_play_land, destroy};
use crate::r_s04_common::add_mana;
use crate::r_s05_common::move_to;
use crate::r_s06_common::activate_containing;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::name_now;
use crate::r_s29_common::cast_and_resolve;
use crate::r_s32_common::{basic_now, options_offered, subtypes_now};
use crate::r_s35_common::pick_options;
use mtg_engine::object::Zone;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn has_sub(t: &TestGame, id: ObjectId, s: &str) -> bool {
    t.obj_now(id).chars.has_subtype(s)
}

#[test]
fn arcane_adaptation_and_maskwood_nexus_apply_before_metallic_mimics_replacement() {
    cr!("614.12", "613.1d");
    ruling!(
        "Arcane Adaptation",
        "If you control Arcane Adaptation and the Aether Revolt card Metallic Mimic, with the same creature type chosen for both, then any creature you control will enter the battlefield with an additional +1/+1 counter on it."
    );
    ruling!(
        "Maskwood Nexus",
        "if you control Maskwood Nexus and an effect says that each Warrior you control enters the battlefield with an additional +1/+1 counter on it, any creature that enters the battlefield under your control will get that counter."
    );
    supported("Arcane Adaptation");
    supported("Maskwood Nexus");
    supported("Metallic Mimic");
    // Control: a Bear isn't an Elf, so Metallic Mimic (Elf) doesn't give it a counter.
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Metallic Mimic");
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    // Arcane Adaptation (Elf): the Bear is an Elf as it would enter.
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Arcane Adaptation");
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    // An opponent's Bear isn't affected by either.
    let theirs = t.enter(P1, "Grizzly Bears");
    assert_eq!(t.counters(theirs, counters::PLUS1), 0);
    // Maskwood Nexus with Metallic Mimic (Warrior).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Maskwood Nexus");
    choose_creature_type(&mut t, P0, "Warrior");
    t.enter(P0, "Metallic Mimic");
    let bears = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn maskwood_nexus_changeling_tokens_keep_every_type_after_it_leaves() {
    cr!("702.73a", "611.3a");
    ruling!(
        "Maskwood Nexus",
        "The tokens created by the activated ability will continue to have all creature types if Maskwood Nexus leaves the battlefield because those tokens have changeling."
    );
    supported("Maskwood Nexus");
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P0, "Maskwood Nexus");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::C, 3);
    activate_containing(&mut t, P0, nexus, "Shapeshifter").unwrap();
    t.resolve_all();
    let token = tokens(&t, P0)[0];
    assert!(has_sub(&t, bears, "Warrior"));
    destroy(&mut t, nexus);
    t.g.recompute();
    assert!(has_sub(&t, token, "Warrior") && has_sub(&t, token, "Elf"));
    assert!(!has_sub(&t, bears, "Warrior"));
    assert!(has_sub(&t, bears, "Bear"));
}

#[test]
fn conspiracy_gives_a_creature_type_to_an_animated_land() {
    cr!("613.1d");
    ruling!(
        "Conspiracy",
        "This can grant a creature type to animated lands and artifacts that would otherwise have no creature type."
    );
    supported("Conspiracy");
    supported("Living Lands");
    // Living Lands: "All Forests are 1/1 creatures that are still lands."
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    t.battlefield(P0, "Living Lands");
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Conspiracy");
    t.g.recompute();
    assert!(t.obj_now(forest).is(CardType::Creature));
    assert!(has_sub(&t, forest, "Elf"));
    // A Forest an opponent controls isn't changed.
    let theirs = t.battlefield(P1, "Forest");
    t.g.recompute();
    assert!(!has_sub(&t, theirs, "Elf"));
}

#[test]
fn enchanted_evening_only_lets_permanents_interact_with_enchantment_effects() {
    cr!("205.1b", "613.1d");
    ruling!(
        "Enchanted Evening",
        "This has no effect on those permanents other than letting them interact with things that care about enchantments."
    );
    supported("Enchanted Evening");
    supported("Tranquility");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Enchanted Evening");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.recompute();
    let o = t.obj_now(bears);
    assert!(o.is(CardType::Enchantment) && o.is(CardType::Creature));
    assert!(has_sub(&t, bears, "Bear"));
    assert_eq!(t.pt(bears), (2, 2));
    // "Destroy all enchantments." destroys the Bears too.
    cast_and_resolve(&mut t, P0, "Tranquility", &[]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn standardize_affects_only_creatures_on_the_battlefield_as_it_resolves() {
    cr!("611.2c", "613.1d");
    ruling!(
        "Standardize",
        "This spell only affects creatures on the battlefield when it resolves."
    );
    supported("Standardize");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    pick_options(&mut t, P0, &["Elf"]);
    cast_and_resolve(&mut t, P0, "Standardize", &[]);
    assert!(has_sub(&t, bears, "Elf") && !has_sub(&t, bears, "Bear"));
    let later = t.enter(P1, "Grizzly Bears");
    t.g.recompute();
    assert!(!has_sub(&t, later, "Elf") && has_sub(&t, later, "Bear"));
}

#[test]
fn toph_doesnt_let_you_play_artifacts_as_lands() {
    cr!("305.1", "205.1b");
    ruling!(
        "Toph, the First Metalbender",
        "does not allow you to play nontoken artifacts as lands. It only affects permanents on the battlefield."
    );
    supported("Toph, the First Metalbender");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toph, the First Metalbender");
    let on_bf = t.battlefield(P0, "Sol Ring");
    let in_hand = t.hand(P0, "Sol Ring");
    t.g.recompute();
    assert!(t.obj_now(on_bf).is(CardType::Land));
    assert!(!t.obj_now(in_hand).is(CardType::Land));
    assert!(!can_play_land(&mut t, P0, in_hand));
    // A real land can still be played.
    let forest = t.hand(P0, "Forest");
    assert!(can_play_land(&mut t, P0, forest));
}

#[test]
fn terraformer_overwrites_and_grants_land_types() {
    cr!("305.7", "613.1d");
    ruling!(
        "Terraformer",
        "Terraformer’s effect overwrites the land type of all lands you control that had other types and gives a land type to all lands you control that don’t normally have one."
    );
    supported("Terraformer");
    let mut t = TestGame::new(2);
    let terraformer = t.battlefield(P0, "Terraformer");
    let forest = t.battlefield(P0, "Forest");
    let gate = t.battlefield(P0, "Azorius Guildgate");
    let wastes = t.battlefield(P0, "Wastes");
    let theirs = t.battlefield(P1, "Forest");
    add_mana(&mut t, P0, ManaType::C, 1);
    pick_options(&mut t, P0, &["Island"]);
    activate_containing(&mut t, P0, terraformer, "basic land type").unwrap();
    t.resolve_all();
    assert_eq!(subtypes_now(&t, forest), vec!["Island"]);
    assert_eq!(subtypes_now(&t, gate), vec!["Island"]);
    assert_eq!(subtypes_now(&t, wastes), vec!["Island"]);
    assert_eq!(subtypes_now(&t, theirs), vec!["Forest"]);
    // The Forest lost "{T}: Add {G}." and has "{T}: Add {U}."
    assert!(tap_for_mana(&mut t, P0, wastes, "{U}"));
    assert!(t
        .obj_now(forest)
        .chars
        .abilities
        .iter()
        .all(|a| !a.text.contains("{G}")));
}

#[test]
fn blanket_of_night_is_ordered_by_timestamp_with_other_land_type_changes() {
    cr!("613.7a", "613.1d", "305.7");
    ruling!(
        "Blanket of Night",
        "The ability is a land type change effect so it is ordered along with other land type change effects in the order they entered the battlefield."
    );
    supported("Blanket of Night");
    supported("Terraformer");
    // Blanket first, then Terraformer's Island: the later effect overwrites Swamp.
    for blanket_first in [true, false] {
        let mut t = TestGame::new(2);
        if blanket_first {
            t.battlefield(P0, "Blanket of Night");
        }
        let terraformer = t.battlefield(P0, "Terraformer");
        let forest = t.battlefield(P0, "Forest");
        add_mana(&mut t, P0, ManaType::C, 1);
        pick_options(&mut t, P0, &["Island"]);
        activate_containing(&mut t, P0, terraformer, "basic land type").unwrap();
        t.resolve_all();
        if !blanket_first {
            t.enter(P0, "Blanket of Night");
        }
        t.g.recompute();
        let want: Vec<&str> = if blanket_first {
            vec!["Island"]
        } else {
            vec!["Island", "Swamp"]
        };
        let mut got = subtypes_now(&t, forest);
        got.sort();
        assert_eq!(got, want, "blanket first: {blanket_first}");
    }
}

#[test]
fn realmwright_offers_only_basic_land_types() {
    cr!("205.3i", "305.6");
    ruling!(
        "Realmwright",
        "The basic land types are Plains, Island, Swamp, Mountain, and Forest. Other land types, like Gate, aren’t basic lands types."
    );
    supported("Realmwright");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    pick_options(&mut t, P0, &["Swamp"]);
    t.enter(P0, "Realmwright");
    let offered = options_offered(&t, from);
    let mut opts = offered[0].clone();
    opts.sort();
    assert_eq!(opts, vec!["Forest", "Island", "Mountain", "Plains", "Swamp"]);
    let gate = t.battlefield(P0, "Azorius Guildgate");
    t.g.recompute();
    let mut subs = subtypes_now(&t, gate);
    subs.sort();
    assert_eq!(subs, vec!["Gate", "Swamp"]);
}

#[test]
fn blanket_of_night_adds_a_black_mana_ability_but_one_tap() {
    cr!("305.6", "605.1a");
    ruling!(
        "Blanket of Night",
        "In other words, the land can be tapped for its own ability _or_ it can be tapped for black mana. Not both."
    );
    ruling!(
        "Blanket of Night",
        "The lands do not get the name “Swamp” in addition to their current name."
    );
    ruling!(
        "Blanket of Night",
        "Whether the land is a basic land or not is unchanged by this effect."
    );
    supported("Blanket of Night");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blanket of Night");
    let forest = t.battlefield(P0, "Forest");
    let gate = t.battlefield(P0, "Azorius Guildgate");
    t.g.recompute();
    assert_eq!(name_now(&t, forest), "Forest");
    assert!(basic_now(&t, forest));
    assert!(!basic_now(&t, gate));
    assert!(has_sub(&t, gate, "Gate") && has_sub(&t, gate, "Swamp"));
    assert!(tap_for_mana(&mut t, P0, forest, "{B}"));
    assert!(!tap_for_mana(&mut t, P0, forest, "{G}"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    let forest2 = t.battlefield(P0, "Forest");
    assert!(tap_for_mana(&mut t, P0, forest2, "{G}"));
    assert!(!tap_for_mana(&mut t, P0, forest2, "{B}"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

#[test]
fn urborg_and_yavimaya_change_only_land_types_on_the_battlefield() {
    cr!("305.6", "305.7", "611.3a");
    ruling!(
        "Urborg, Tomb of Yawgmoth",
        "Nothing else changes about those lands, including their names, other subtypes, other abilities, and whether they're legendary, basic, or snow."
    );
    ruling!(
        "Urborg, Tomb of Yawgmoth",
        "Urborg, Tomb of Yawgmoth isn't a Swamp while it's not on the battlefield."
    );
    ruling!(
        "Yavimaya, Cradle of Growth",
        "Nothing else changes about those lands, including their names, other subtypes, and whether they're legendary, basic, or snow."
    );
    ruling!(
        "Yavimaya, Cradle of Growth",
        "Yavimaya, Cradle of Growth isn't a Forest while it's not on the battlefield."
    );
    for (land, ty, mana, other_mana) in [
        ("Urborg, Tomb of Yawgmoth", "Swamp", "{B}", "{W}"),
        ("Yavimaya, Cradle of Growth", "Forest", "{G}", "{W}"),
    ] {
        supported(land);
        let mut t = TestGame::new(2);
        let in_hand = t.hand(P0, land);
        let in_gy = t.graveyard(P0, land);
        let snow = t.battlefield(P1, "Snow-Covered Plains");
        let gate = t.battlefield(P1, "Azorius Guildgate");
        let src = t.battlefield(P0, land);
        t.g.recompute();
        assert!(!has_sub(&t, in_hand, ty) && !has_sub(&t, in_gy, ty), "{land}");
        assert!(has_sub(&t, src, ty), "{land}");
        assert!(t.obj_now(src).chars.supertypes.contains(Supertype::Legendary));
        // Each land on the battlefield (any controller) gets the type and nothing else changes.
        let o = t.obj_now(snow);
        assert_eq!(o.chars.name.as_str(), "Snow-Covered Plains");
        assert!(o.chars.supertypes.contains(Supertype::Basic));
        assert!(o.chars.supertypes.contains(Supertype::Snow));
        assert!(has_sub(&t, snow, "Plains") && has_sub(&t, snow, ty));
        assert!(has_sub(&t, gate, "Gate") && has_sub(&t, gate, ty) && !basic_now(&t, gate));
        assert!(tap_for_mana(&mut t, P1, gate, mana), "{land}");
        assert!(tap_for_mana(&mut t, P1, snow, other_mana), "{land}");
        // Once it leaves, the other lands aren't that type any more.
        move_to(&mut t, src, Zone::Graveyard(P0));
        t.g.recompute();
        assert!(!has_sub(&t, gate, ty), "{land}");
    }
}
