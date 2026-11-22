//! Rulings batch P208 — enchant (CR 613, 704.5m): Auras that remove abilities, set
//! types or characteristics, and Auras whose enchant restriction stops being met.

use crate::r_p209_common::cast_spell;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Jump ("Target creature gains flying until end of turn") resolves on `id`.
fn jump(t: &mut TestGame, id: ObjectId) {
    cast_spell(t, P0, "Jump", &[Entity::Object(id)]);
    t.resolve_all();
}

#[test]
fn abilities_gained_after_lose_all_abilities_auras_are_kept() {
    cr!("613.7", "613.1f", "613.7a");
    ruling!(
        "Amphibian Downpour",
        "If the enchanted creature gains an ability after Amphibian Downpour becomes attached to it, it will keep that ability."
    );
    ruling!(
        "Duskmourn's Domination",
        "If the enchanted creature gains an ability after Duskmourn's Domination becomes attached to it, it will keep that ability."
    );
    ruling!(
        "Fresh Start",
        "If the enchanted creature gains an ability after Fresh Start becomes attached to it, it will keep that ability."
    );
    ruling!(
        "Reprobation",
        "If the enchanted creature gains an ability after Reprobation becomes attached to it, it will keep that ability."
    );
    ruling!(
        "Utter Insignificance",
        "If the enchanted creature gains an ability after Utter Insignificance becomes attached to it, it will keep that ability."
    );
    ruling!(
        "Tightening Coils",
        "If an effect gives the enchanted creature flying after Tightening Coils became attached to it, the creature will have flying."
    );
    supported("Healer's Hawk");
    supported("Jump");
    for aura in [
        "Amphibian Downpour",
        "Duskmourn's Domination",
        "Fresh Start",
        "Reprobation",
        "Utter Insignificance",
        "Tightening Coils",
    ] {
        supported(aura);
        let mut t = TestGame::new(2);
        let hawk = t.battlefield(P1, "Healer's Hawk");
        attach_new(&mut t, P0, aura, hawk);
        assert!(!has_kw(&t, hawk, KeywordKind::Flying), "{aura} removes flying");
        jump(&mut t, hawk);
        assert!(has_kw(&t, hawk, KeywordKind::Flying), "{aura}: gained later");
        if aura != "Tightening Coils" {
            assert!(!has_kw(&t, hawk, KeywordKind::Lifelink), "{aura}");
        }
    }
}

#[test]
fn earthbind_flying_gained_later_applies_in_timestamp_order() {
    cr!("613.7", "613.1f", "603.4");
    ruling!(
        "Earthbind",
        "If the enchanted creature gains flying after Earthbind is put onto it, it will have flying. The two effects are simply applied in timestamp order."
    );
    supported("Earthbind");
    supported("Air Elemental");
    let mut t = TestGame::new(2);
    let air = t.battlefield(P1, "Air Elemental");
    crate::r_p209_common::cast_aura(&mut t, P0, "Earthbind", air);
    assert_eq!(t.obj_now(air).damage, 2);
    assert!(!has_kw(&t, air, KeywordKind::Flying));
    jump(&mut t, air);
    assert!(has_kw(&t, air, KeywordKind::Flying));
}

#[test]
fn trapped_in_the_tower_falls_off_a_creature_that_gains_flying() {
    cr!("303.4c", "704.5m");
    ruling!(
        "Trapped in the Tower",
        "If the enchanted creature gains flying, Trapped in the Tower is put into your graveyard as a state-based action. The creature won't get stuck in the tower again when it loses flying."
    );
    supported("Trapped in the Tower");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let trap = attach_new(&mut t, P0, "Trapped in the Tower", bears);
    jump(&mut t, bears);
    assert!(!t.on_battlefield(trap));
    assert!(t.in_graveyard(P0, "Trapped in the Tower"));
    t.advance_to(P1, Step::BeginningOfCombat);
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
    assert!(t.in_graveyard(P0, "Trapped in the Tower"));
    assert!(can_attack(&mut t, bears));
}

#[test]
fn eaten_by_piranhas_and_witness_protection_remove_noncreature_subtypes() {
    cr!("613.1d", "205.1b", "201.2");
    ruling!(
        "Eaten by Piranhas",
        "If the enchanted creature had any subtypes other than creature types, such as Equipment, Vehicle, or Cave, it loses those as well."
    );
    ruling!(
        "Witness Protection",
        "If the enchanted creature had any subtypes other than creature types, such as Equipment, Vehicle, or Shrine, it loses those as well."
    );
    supported("Smuggler's Copter");
    for (aura, subtype, name) in [
        ("Eaten by Piranhas", "Skeleton", "Smuggler's Copter"),
        ("Witness Protection", "Citizen", "Legitimate Businessperson"),
    ] {
        supported(aura);
        let mut t = TestGame::new(2);
        let copter = t.battlefield(P0, "Smuggler's Copter");
        let bears = t.battlefield(P0, "Grizzly Bears");
        assert!(crew(&mut t, P0, copter, &[bears]));
        t.resolve_all();
        assert!(is_creature(&t, copter));
        attach_new(&mut t, P1, aura, copter);
        let o = t.obj_now(copter);
        assert!(!o.chars.subtypes.iter().any(|s| s == "Vehicle"), "{aura}");
        assert_eq!(o.chars.subtypes.len(), 1, "{aura}");
        assert!(o.chars.subtypes.iter().any(|s| s == subtype), "{aura}");
        assert!(!o.is(CardType::Artifact), "{aura}");
        assert_eq!(o.chars.name, name, "{aura}");
        assert_eq!(t.pt(copter), (1, 1), "{aura}");
        assert!(!has_kw(&t, copter, KeywordKind::Flying), "{aura}");
    }
}

#[test]
fn lignify_doesnt_stop_a_color_defining_ability() {
    cr!("613.1e", "613.1f", "604.3");
    ruling!(
        "Lignify",
        "If the enchanted creature has a characteristic-defining ability that specifies its color, that ability will still have its effect because it's applied before the creature loses the ability."
    );
    supported("Lignify");
    supported("Transguild Courier");
    let mut t = TestGame::new(2);
    let courier = t.battlefield(P1, "Transguild Courier");
    attach_new(&mut t, P0, "Lignify", courier);
    assert_eq!(t.pt(courier), (0, 4));
    assert_eq!(t.obj_now(courier).chars.colors, ColorSet::ALL);
}

#[test]
fn deep_freeze_stops_granting_abilities_but_keeps_later_ones() {
    cr!("613.1f", "613.7");
    ruling!(
        "Deep Freeze",
        "If the enchanted creature has an ability that grants abilities to other objects, Deep Freeze’s effect will stop it from doing so. If the enchanted creature gains an ability after Deep Freeze resolves, it will keep that ability."
    );
    supported("Deep Freeze");
    supported("Goblin King");
    supported("Raging Goblin");
    let mut t = TestGame::new(2);
    let king = t.battlefield(P1, "Goblin King");
    let goblin = t.battlefield(P1, "Raging Goblin");
    assert!(has_kw(&t, goblin, KeywordKind::Landwalk));
    crate::r_p209_common::cast_aura(&mut t, P0, "Deep Freeze", king);
    assert_eq!(t.pt(king), (0, 4));
    assert!(has_kw(&t, king, KeywordKind::Defender));
    assert!(!has_kw(&t, goblin, KeywordKind::Landwalk));
    assert_eq!(t.pt(goblin), (1, 1));
    jump(&mut t, king);
    assert!(has_kw(&t, king, KeywordKind::Flying));
}

#[test]
fn reprobation_keeps_an_animated_permanent_a_creature() {
    cr!("613.1d", "611.2a", "514.2");
    ruling!(
        "Reprobation",
        "If a noncreature permanent becomes a creature and is enchanted with Reprobation, it will remain a creature thanks to Reprobation’s effect, even when the original effect that made it a creature expires."
    );
    supported("Reprobation");
    supported("Mutavault");
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P1, "Mutavault");
    t.lands(P1, "Wastes", 1);
    activate_containing(&mut t, P1, vault, "becomes").unwrap();
    t.resolve_all();
    assert!(is_creature(&t, vault));
    attach_new(&mut t, P0, "Reprobation", vault);
    assert_eq!(t.pt(vault), (0, 1));
    t.advance_to(P1, Step::Upkeep);
    assert!(is_creature(&t, vault));
    assert!(t.on_battlefield(vault));
    assert!(t.obj_now(vault).chars.subtypes.iter().any(|s| s == "Coward"));
    assert_eq!(t.pt(vault), (0, 1));
}

#[test]
fn awakened_awareness_sets_a_vehicles_base_pt_once_it_is_a_creature() {
    cr!("613.4b", "301.7a");
    ruling!(
        "Awakened Awareness",
        "If Awakened Awareness is enchanting a Vehicle that is currently a creature or it becomes a creature later, its base power and toughness will be 1/1, not the power and toughness printed on the Vehicle."
    );
    supported("Awakened Awareness");
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Awakened Awareness", copter);
    assert!(!is_creature(&t, copter));
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (1, 1));
}

#[test]
fn one_with_the_stars_on_a_plain_enchantment_changes_nothing() {
    cr!("613.1d", "205.1a");
    ruling!(
        "One with the Stars",
        "If One with the Stars enchants an enchantment that has no other card types, the enchanted permanent won’t be affected."
    );
    supported("One with the Stars");
    supported("Glorious Anthem");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let before = t.obj_now(anthem).chars.clone();
    attach_new(&mut t, P1, "One with the Stars", anthem);
    let after = &t.obj_now(anthem).chars;
    assert_eq!(after.card_types, before.card_types);
    assert_eq!(after.subtypes, before.subtypes);
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn enchant_restrictions_that_stop_being_met_put_the_aura_into_the_graveyard() {
    cr!("303.4c", "704.5m", "305.7");
    ruling!(
        "Call to Serve",
        "If the enchanted creature becomes black, Call to Serve will be put into its owner’s graveyard the next time state-based actions are performed."
    );
    ruling!(
        "Awaken the Ancient",
        "If the enchanted Mountain stops being a Mountain for any reason, Awaken the Ancient will be put into its owner's graveyard as a state-based action."
    );
    ruling!(
        "Glimmerdust Nap",
        "If the enchanted creature becomes untapped, Glimmerdust Nap is put into its owner’s graveyard as a state-based action."
    );
    supported("Call to Serve");
    supported("Darkest Hour");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = attach_new(&mut t, P0, "Call to Serve", bears);
    t.settle();
    assert!(t.on_battlefield(a));
    t.battlefield(P1, "Darkest Hour");
    t.settle();
    assert!(t.in_graveyard(P0, "Call to Serve"));

    supported("Awaken the Ancient");
    supported("Spreading Seas");
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    let a = attach_new(&mut t, P0, "Awaken the Ancient", mountain);
    assert_eq!(t.pt(mountain), (7, 7));
    t.settle();
    assert!(t.on_battlefield(a));
    attach_new(&mut t, P1, "Spreading Seas", mountain);
    t.settle();
    assert!(t.in_graveyard(P0, "Awaken the Ancient"));
    assert!(!is_creature(&t, mountain));

    supported("Glimmerdust Nap");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p209_common::tap(&mut t, bears);
    let a = attach_new(&mut t, P0, "Glimmerdust Nap", bears);
    t.settle();
    assert!(t.on_battlefield(a));
    crate::r_p209_common::untap(&mut t, bears);
    t.settle();
    assert!(t.in_graveyard(P0, "Glimmerdust Nap"));
}

#[test]
fn tezzerets_touch_unattaches_an_animated_equipment() {
    cr!("301.5c", "704.5n");
    ruling!(
        "Tezzeret's Touch",
        "If the enchanted artifact is an Equipment, it becomes unattached and it can’t be attached to anything for as long as it remains a creature."
    );
    supported("Tezzeret's Touch");
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(splitter, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    attach_new(&mut t, P0, "Tezzeret's Touch", splitter);
    t.settle();
    assert!(is_creature(&t, splitter));
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(splitter), (5, 5));
    // It can't be equipped while it's a creature.
    t.lands(P0, "Wastes", 1);
    let _ = t.activate(P0, splitter, 0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), None);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn domestication_checks_the_modified_power_at_end_step() {
    cr!("603.4", "611.2a", "514.2");
    ruling!(
        "Domestication",
        "If an effect changes the enchanted creature's power until end of turn, it will still have the modified power during your end step."
    );
    supported("Domestication");
    supported("Giant Growth");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let dom = attach_new(&mut t, P0, "Domestication", bears);
    assert_eq!(t.obj_now(bears).controller, P0);
    cast_spell(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifice"), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(dom));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn psychic_possession_controlled_by_the_enchanted_player_falls_off() {
    cr!("303.4c", "704.5m", "303.4e");
    ruling!(
        "Psychic Possession",
        "If Psychic Possession's controller ever happens to be the player it's enchanting, Psychic Possession will be put into its owner's graveyard as a state-based action."
    );
    supported("Psychic Possession");
    let mut t = TestGame::new(2);
    let pp = attach_new(&mut t, P0, "Psychic Possession", P1);
    t.settle();
    assert!(t.on_battlefield(pp));
    give_control(&mut t, pp, P1);
    assert!(t.in_graveyard(P0, "Psychic Possession"));
}

#[test]
fn grievous_wound_stops_a_higher_life_total_from_being_set() {
    cr!("119.5", "119.7");
    ruling!(
        "Grievous Wound",
        "If an effect says to set the enchanted player's life total to a number that's higher than their current life total, that player's life total won't change."
    );
    supported("Grievous Wound");
    supported("Blessed Wind");
    let mut t = TestGame::new(2);
    t.g.players[P1.idx()].life = 10;
    attach_new(&mut t, P0, "Grievous Wound", P1);
    cast_spell(&mut t, P0, "Blessed Wind", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 10);
    // A lower life total is set.
    t.g.players[P1.idx()].life = 25;
    cast_spell(&mut t, P0, "Blessed Wind", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn indestructible_auras_lethal_damage_stays_and_destroys_once_removed() {
    cr!("702.12b", "704.5g", "120.6");
    ruling!(
        "Indestructibility",
        "If a creature enchanted by Indestructibility is dealt lethal damage, the creature isn’t destroyed, but the damage remains marked on the creature."
    );
    ruling!(
        "Shield of the Oversoul",
        "If a green creature enchanted by Shield of the Oversoul is dealt lethal damage, the creature isn’t destroyed, but the damage remains on the creature."
    );
    for aura in ["Indestructibility", "Shield of the Oversoul"] {
        supported(aura);
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let a = attach_new(&mut t, P0, aura, bears);
        damage(&mut t, giant, 3, bears);
        assert!(t.on_battlefield(bears), "{aura}");
        assert_eq!(t.obj_now(bears).damage, 3);
        destroy(&mut t, a);
        assert!(!t.on_battlefield(bears), "{aura}");
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
    }
}

#[test]
fn destroying_an_indestructible_aura_and_its_creature_together_destroys_only_the_aura() {
    cr!("702.12b", "701.8b");
    ruling!(
        "Indestructibility",
        "If an effect would simultaneously destroy Indestructibility and the creature it’s enchanting, only Indestructibility is destroyed."
    );
    ruling!(
        "Shield of the Oversoul",
        "If an effect would simultaneously destroy Shield of the Oversoul and a green creature it’s enchanting, only the Shield is destroyed."
    );
    for aura in ["Indestructibility", "Shield of the Oversoul"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let a = attach_new(&mut t, P0, aura, bears);
        t.g.destroy_all(vec![bears, a], None, false);
        t.g.flush_events();
        t.settle();
        assert!(!t.on_battlefield(a), "{aura}");
        assert!(t.on_battlefield(bears), "{aura}");
    }
}
