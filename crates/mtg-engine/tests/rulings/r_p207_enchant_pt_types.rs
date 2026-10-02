//! Rulings batch P207 — enchant (CR 303.4, 613): Auras that set base power and toughness
//! or overwrite colors and types (layers 4, 5 and 7b), on Vehicles that are only
//! temporarily creatures, and Auras whose bonus counts permanents.

use crate::r_s01_common::*;
use crate::r_s02_common::target_candidates;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const COPTER: &str = "Smuggler's Copter";

/// P0's Smuggler's Copter (a 3/3 Vehicle with crew 1), crewed by Grizzly Bears.
fn crewed_copter(t: &mut TestGame) -> ObjectId {
    supported(COPTER);
    let copter = t.battlefield(P0, COPTER);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(is_creature(t, copter));
    copter
}

/// Ends the turn (P0's) so that "until end of turn" effects end, into P1's upkeep.
fn end_turn(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.settle();
}

/// Casts the Aura `name` (P0, with the mana for it) targeting `target` and resolves it.
fn cast_aura(t: &mut TestGame, name: &str, target: ObjectId) -> ObjectId {
    supported(name);
    let aura = in_hand_with_mana(t, P0, name);
    t.cast(P0, aura).target(target).go();
    t.resolve_all();
    t.g.current(aura)
}

fn types(t: &TestGame, id: ObjectId) -> CardTypeSet {
    t.obj_now(id).chars.card_types
}

// ---------------------------------------------------------------------------------------
// Tezzeret's Touch
// ---------------------------------------------------------------------------------------

#[test]
fn tezzerets_touch_makes_an_artifact_creature_or_a_vehicle_5_5() {
    cr!("613.4b", "301.7b", "702.122a");
    ruling!(
        "Tezzeret's Touch",
        "An artifact creature enchanted by Tezzeret’s Touch becomes 5/5 instead of its normal power and toughness."
    );
    ruling!(
        "Tezzeret's Touch",
        "A Vehicle enchanted by Tezzeret’s Touch becomes a 5/5 creature. Crewing it won’t change its power and toughness."
    );
    supported("Tezzeret's Touch");
    supported("Ornithopter");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P0, "Ornithopter");
    attach_new(&mut t, P0, "Tezzeret's Touch", thopter);
    assert_eq!(t.pt(thopter), (5, 5));
    assert!(has_kw(&t, thopter, KeywordKind::Flying));
    // A Vehicle: a 5/5 artifact creature before and after it's crewed.
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, COPTER);
    attach_new(&mut t, P0, "Tezzeret's Touch", copter);
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (5, 5));
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert_eq!(t.pt(copter), (5, 5));
    end_turn(&mut t);
    assert!(is_creature(&t, copter));
    assert_eq!(t.pt(copter), (5, 5));
}

// ---------------------------------------------------------------------------------------
// Frog and Fish Auras on Vehicles
// ---------------------------------------------------------------------------------------

/// The Aura `name` on a crewed Smuggler's Copter: it stays a 1/1 blue Frog creature after
/// the crew effect ends.
fn frog_on_a_vehicle(name: &str) {
    let mut t = TestGame::new(2);
    let copter = crewed_copter(&mut t);
    let aura = attach_new(&mut t, P0, name, copter);
    assert_eq!(t.pt(copter), (1, 1));
    end_turn(&mut t);
    assert!(t.on_battlefield(aura));
    assert!(is_creature(&t, copter), "{name}: not a creature");
    assert_eq!(types(&t, copter), CardTypeSet::single(CardType::Creature));
    assert_eq!(t.pt(copter), (1, 1));
    assert!(t.obj_now(copter).chars.has_subtype("Frog"));
    assert!(!t.obj_now(copter).chars.has_subtype("Vehicle"));
    assert_eq!(t.obj_now(copter).chars.colors, ColorSet::single(Color::Blue));
}

#[test]
fn frog_auras_keep_a_vehicle_a_creature() {
    cr!("613.1d", "613.4b", "205.1a", "301.7b");
    ruling!(
        "Amphibian Downpour",
        "Amphibian Downpour may enchant a permanent that is only temporarily a creature, such as a Vehicle. If this happens, Amphibian Downpour's effect causes the enchanted permanent to remain a 1/1 blue Frog creature even after the temporary effect making it a creature expires."
    );
    ruling!(
        "Frogify",
        "Frogify may enchant a permanent that is only temporarily a creature, such as a Vehicle. If this happens, Frogify's effect causes the enchanted permanent to remain a 1/1 blue Frog creature even after the temporary effect expires."
    );
    supported("Amphibian Downpour");
    supported("Frogify");
    frog_on_a_vehicle("Amphibian Downpour");
    frog_on_a_vehicle("Frogify");
}

#[test]
fn amphibian_downpour_overwrites_colors_and_types_but_not_supertypes() {
    cr!("613.1d", "613.1e", "205.1a", "205.4d");
    ruling!(
        "Amphibian Downpour",
        "Amphibian Downpour overwrites all colors and creature types the enchanted creature has. It's just a blue Frog. The creature keeps any supertypes it has (such as legendary) but loses any other card types it has (such as artifact)."
    );
    supported("Karn, Silver Golem");
    // Karn, Silver Golem: a legendary artifact creature — Golem.
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P1, "Karn, Silver Golem");
    cast_aura(&mut t, "Amphibian Downpour", karn);
    let c = &t.obj_now(karn).chars;
    assert_eq!(c.card_types, CardTypeSet::single(CardType::Creature));
    assert!(c.supertypes.contains(Supertype::Legendary));
    assert!(c.has_subtype("Frog") && !c.has_subtype("Golem"));
    assert_eq!(c.colors, ColorSet::single(Color::Blue));
    assert_eq!(t.pt(karn), (1, 1));
}

/// The Aura `name` on a crewed Smuggler's Copter: once the crew effect ends, it's no
/// longer a creature and the Aura is put into its owner's graveyard.
fn falls_off_a_vehicle(name: &str) -> TestGame {
    let mut t = TestGame::new(2);
    let copter = crewed_copter(&mut t);
    let aura = attach_new(&mut t, P0, name, copter);
    assert!(is_creature(&t, copter));
    end_turn(&mut t);
    assert!(!is_creature(&t, copter), "{name}: still a creature");
    assert!(!t.on_battlefield(aura), "{name}: still on the battlefield");
    t
}

#[test]
fn eye_of_nidhogg_and_ichthyomorphosis_fall_off_a_vehicle() {
    cr!("303.4d", "704.5m", "301.7b");
    ruling!(
        "Eye of Nidhogg",
        "Eye of Nidhogg may enchant a permanent that is only temporarily a creature, such as a Vehicle. If this happens, Eye of Nidhogg will be put into its owner's graveyard as a state-based action once the creature stops being a creature."
    );
    ruling!(
        "Ichthyomorphosis",
        "Ichthyomorphosis may enchant a permanent that is only temporarily a creature, such as a Vehicle. If this happens, Ichthyomorphosis will be put into its owner’s graveyard as a state-based action once the creature stops being a creature."
    );
    supported("Eye of Nidhogg");
    supported("Ichthyomorphosis");
    let mut t = falls_off_a_vehicle("Eye of Nidhogg");
    // (Its own ability returns it from the graveyard to its owner's hand.)
    t.resolve_all();
    assert!(t.in_hand(P0, "Eye of Nidhogg"));
    let t = falls_off_a_vehicle("Ichthyomorphosis");
    assert!(t.in_graveyard(P0, "Ichthyomorphosis"));
}

/// The Aura `name` on Isamaru (legendary) and on Nyx-Fleece Ram (an enchantment
/// creature): the colors and creature types are overwritten; supertypes and card types
/// are kept.
fn keeps_supertypes_and_card_types(name: &str, color: Color, subtype: &str, pt: (i32, i32)) {
    supported("Isamaru, Hound of Konda");
    supported("Nyx-Fleece Ram");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P1, "Isamaru, Hound of Konda");
    attach_new(&mut t, P0, name, isamaru);
    let c = &t.obj_now(isamaru).chars;
    assert!(c.supertypes.contains(Supertype::Legendary));
    assert!(c.has_subtype(subtype) && !c.has_subtype("Dog"));
    assert_eq!(c.colors, ColorSet::single(color));
    assert_eq!(t.pt(isamaru), pt);
    let ram = t.battlefield(P1, "Nyx-Fleece Ram");
    attach_new(&mut t, P0, name, ram);
    let c = &t.obj_now(ram).chars;
    assert!(c.is(CardType::Enchantment) && c.is(CardType::Creature));
    assert!(c.has_subtype(subtype) && !c.has_subtype("Sheep"));
    assert_eq!(c.colors, ColorSet::single(color));
}

#[test]
fn eye_of_nidhogg_makes_a_black_dragon_keeping_supertypes_and_card_types() {
    cr!("613.1d", "613.1e", "205.1b", "613.4b");
    ruling!(
        "Eye of Nidhogg",
        "Eye of Nidhogg overwrites all colors and creature types the enchanted creature has. It's just a black Dragon. The creature keeps any supertypes it has (such as legendary), as well as any other card types it has (such as enchantment)."
    );
    keeps_supertypes_and_card_types("Eye of Nidhogg", Color::Black, "Dragon", (4, 2));
}

#[test]
fn ichthyomorphosis_makes_a_blue_fish_keeping_supertypes_and_card_types() {
    cr!("613.1d", "613.1e", "205.1b", "613.4b");
    ruling!(
        "Ichthyomorphosis",
        "Ichthyomorphosis overwrites all colors and creature types the enchanted creature has. It’s just a blue Fish. The creature keeps any supertypes (such as legendary) it has, as well as any other card types it has (such as enchantment)."
    );
    keeps_supertypes_and_card_types("Ichthyomorphosis", Color::Blue, "Fish", (0, 1));
}

// ---------------------------------------------------------------------------------------
// Base power and toughness and other effects
// ---------------------------------------------------------------------------------------

/// Humble ("Until end of turn, target creature loses all abilities and has base power
/// and toughness 0/1") on `target`, cast by P0.
fn humble(t: &mut TestGame, target: ObjectId) {
    supported("Humble");
    let h = in_hand_with_mana(t, P0, "Humble");
    t.cast(P0, h).target(target).go();
    t.resolve_all();
}

fn giant_growth(t: &mut TestGame, target: ObjectId) {
    let gg = in_hand_with_mana(t, P0, "Giant Growth");
    t.cast(P0, gg).target(target).go();
    t.resolve_all();
}

#[test]
fn awakened_awareness_overwrites_earlier_setting_effects_but_not_modifications() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Awakened Awareness",
        "Awakened Awareness will overwrite any previous effects that set the enchanted creature's power and toughness to specific numbers. Effects that otherwise modify the enchanted creature's power and toughness will still apply no matter when they took effect. The same is true for +1/+1 counters."
    );
    supported("Awakened Awareness");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    humble(&mut t, giant);
    giant_growth(&mut t, giant);
    assert_eq!(t.pt(giant), (3, 4));
    // X = 2: two +1/+1 counters, and base 1/1.
    let aura = in_hand_with_mana(&mut t, P0, "Awakened Awareness");
    t.lands(P0, "Wastes", 2);
    t.cast(P0, aura).target(giant).x(2).go();
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::PLUS1), 2);
    // 1/1 base, +3/+3, +2/+2.
    assert_eq!(t.pt(giant), (6, 6));
    // A later Giant Growth applies too.
    giant_growth(&mut t, giant);
    assert_eq!(t.pt(giant), (9, 9));
}

#[test]
fn gigantiform_and_reprobation_let_modifications_apply_whenever_they_began() {
    cr!("613.4b", "613.4c", "613.4d", "613.7");
    ruling!(
        "Gigantiform",
        "Effects that modify the enchanted creature's power or toughness, such as the effects of Giant Growth or Glorious Anthem, will apply to it no matter when they started to take effect. The same is true for counters that change the creature's power or toughness (such as +1/+1 counters) and effects that switch its power and toughness."
    );
    ruling!(
        "Reprobation",
        "Effects that modify the enchanted creature’s power or toughness (such as the effects of Force of Virtue or Giant Growth) will apply no matter when they started to take effect. The same is true for counters that change the creature’s power or toughness (such as +1/+1 counters)."
    );
    supported("Gigantiform");
    supported("Reprobation");
    supported("Glorious Anthem");
    for (name, base) in [("Gigantiform", (8, 8)), ("Reprobation", (0, 1))] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Glorious Anthem");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, None);
        giant_growth(&mut t, bears);
        attach_new(&mut t, P0, name, bears);
        // Base, +1/+1 (Anthem), +3/+3 (Giant Growth, earlier), +1/+1 (counter).
        let expect = (base.0 + 5, base.1 + 5);
        assert_eq!(t.pt(bears), expect, "{name}");
        giant_growth(&mut t, bears);
        assert_eq!(t.pt(bears), (expect.0 + 3, expect.1 + 3), "{name}");
    }
}

#[test]
fn a_later_setting_effect_overwrites_gigantiform() {
    cr!("613.4b", "613.7");
    ruling!(
        "Gigantiform",
        "Gigantiform overwrites all previous effects that set the enchanted creature's power and toughness to specific values. Other effects that set its power or toughness to specific values that start to apply after Gigantiform becomes attached to it will overwrite Gigantiform."
    );
    // Earlier Humble: Gigantiform wins.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    humble(&mut t, bears);
    assert_eq!(t.pt(bears), (0, 1));
    attach_new(&mut t, P0, "Gigantiform", bears);
    assert_eq!(t.pt(bears), (8, 8));
    // Later Humble: Humble wins (and the creature loses trample).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Gigantiform", bears);
    assert_eq!(t.pt(bears), (8, 8));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    humble(&mut t, bears);
    assert_eq!(t.pt(bears), (0, 1));
}

#[test]
fn a_copy_of_an_awakened_mountain_is_just_a_mountain() {
    cr!("707.2", "613.1a", "613.1d");
    ruling!(
        "Awaken the Ancient",
        "Copies of the enchanted Mountain won't be 7/7 red Giant creatures with haste. For example, if Awaken the Ancient is enchanting a basic Mountain, a copy of that creature would just be a basic Mountain."
    );
    supported("Awaken the Ancient");
    supported("Clone");
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P0, "Mountain");
    attach_new(&mut t, P0, "Awaken the Ancient", mountain);
    assert_eq!(t.pt(mountain), (7, 7));
    assert!(has_kw(&t, mountain, KeywordKind::Haste));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(mountain)]);
    let clone = t.enter(P0, "Clone");
    t.settle();
    let c = &t.obj_now(clone).chars;
    assert_eq!(c.name, "Mountain");
    assert_eq!(c.card_types, CardTypeSet::single(CardType::Land));
    assert!(c.supertypes.contains(Supertype::Basic));
    assert!(c.has_subtype("Mountain") && !c.has_subtype("Giant"));
}

#[test]
fn awaken_the_ancient_can_enchant_any_land_with_the_mountain_subtype() {
    cr!("303.4a", "205.3i", "115.1");
    ruling!(
        "Awaken the Ancient",
        "Awaken the Ancient can enchant any land with the subtype Mountain, not just one named Mountain."
    );
    supported("Taiga");
    let mut t = TestGame::new(2);
    let taiga = t.battlefield(P0, "Taiga");
    let aura = cast_aura(&mut t, "Awaken the Ancient", taiga);
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(taiga)));
    assert_eq!(t.pt(taiga), (7, 7));
    assert!(t.obj_now(taiga).chars.is(CardType::Land));
    // A land that isn't a Mountain can't be targeted.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let taiga = t.battlefield(P0, "Taiga");
    let mountain = t.battlefield(P0, "Mountain");
    let aura = in_hand_with_mana(&mut t, P0, "Awaken the Ancient");
    let from = t.asked().len();
    t.cast(P0, aura).target(taiga).go();
    let legal = target_candidates(&t, P0, from).remove(0);
    assert!(!legal.contains(&Entity::Object(forest)));
    assert!(legal.contains(&Entity::Object(taiga)));
    assert!(legal.contains(&Entity::Object(mountain)));
}

// ---------------------------------------------------------------------------------------
// Auras that add types or abilities
// ---------------------------------------------------------------------------------------

#[test]
fn dub_on_a_knight_still_gives_its_bonus() {
    cr!("613.4c", "205.3d");
    ruling!(
        "Dub",
        "Dub can enchant a creature that's already a Knight. It will get +2/+2 and have first strike, but it won't benefit from becoming a Knight."
    );
    supported("Dub");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Silver Knight");
    assert!(has_kw(&t, knight, KeywordKind::FirstStrike));
    let aura = cast_aura(&mut t, "Dub", knight);
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(knight)));
    assert_eq!(t.pt(knight), (4, 4));
    assert!(has_kw(&t, knight, KeywordKind::FirstStrike));
    assert!(t.obj_now(knight).chars.has_subtype("Knight"));
}

#[test]
fn eldrazi_conscription_doesnt_make_the_creature_an_eldrazi() {
    cr!("308.1", "205.3d", "303.4");
    ruling!(
        "Eldrazi Conscription",
        "Eldrazi Conscription is an Eldrazi. However, it doesn't turn the enchanted creature into an Eldrazi."
    );
    supported("Eldrazi Conscription");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = attach_new(&mut t, P0, "Eldrazi Conscription", bears);
    assert!(t.obj_now(aura).chars.has_subtype("Eldrazi"));
    assert!(!t.obj_now(bears).chars.has_subtype("Eldrazi"));
    assert_eq!(t.pt(bears), (12, 12));
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    assert!(has_kw(&t, bears, KeywordKind::Annihilator));
}

#[test]
fn gauntlets_of_light_changes_only_combat_damage_assignment() {
    cr!("510.1a", "701.14a", "208.1");
    ruling!(
        "Gauntlets of Light",
        "Gauntlets of Light’s effect doesn’t change the enchanted creature’s power; it changes only the amount of combat damage that creature assigns. All other rules and effects that check power or toughness use the actual values. For example, if a creature fights while enchanted by Gauntlets of Light, that creature will deal damage equal to its power, not its toughness."
    );
    supported("Gauntlets of Light");
    supported("Prey Upon");
    // Grizzly Bears (2/4 enchanted) fights Hill Giant (3/3): it deals 2 damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Gauntlets of Light", bears);
    assert_eq!(t.pt(bears), (2, 4));
    let giant = t.battlefield(P1, "Hill Giant");
    let prey = in_hand_with_mana(&mut t, P0, "Prey Upon");
    t.cast(P0, prey).targets(&[Entity::Object(bears), Entity::Object(giant)]).go();
    t.resolve_all();
    assert_eq!(damage_on(&t, giant), 2);
    assert!(t.on_battlefield(giant));
    // In combat, it assigns 4.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Gauntlets of Light", bears);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
}

fn damage_on(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).damage
}

// ---------------------------------------------------------------------------------------
// Counting Auras
// ---------------------------------------------------------------------------------------

#[test]
fn hope_against_hope_counts_the_enchanted_creature() {
    cr!("613.4c", "109.4");
    ruling!(
        "Hope Against Hope",
        "Hope Against Hope’s second ability counts the enchanted creature, so that creature will normally get at least +1/+1."
    );
    supported("Hope Against Hope");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Hope Against Hope", bears);
    assert_eq!(t.pt(bears), (3, 3));
    // P1's creatures don't count.
    t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn all_that_glitters_counts_itself_and_an_artifact_enchantment_once() {
    cr!("613.4c", "109.4", "205.2a");
    ruling!(
        "All That Glitters",
        "Because All That Glitters is an enchantment, the enchanted creature usually gets at least +1/+1."
    );
    ruling!(
        "All That Glitters",
        "A permanent that's both an artifact and an enchantment is counted only once."
    );
    supported("All That Glitters");
    supported("Hammer of Purphoros");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "All That Glitters", bears);
    assert_eq!(t.pt(bears), (3, 3));
    // Hammer of Purphoros: a legendary enchantment artifact.
    t.battlefield(P0, "Hammer of Purphoros");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn ethereal_armor_counts_itself_and_auras_on_opponents_permanents() {
    cr!("613.4c", "109.4", "303.4e");
    ruling!(
        "Ethereal Armor",
        "Ethereal Armor counts each enchantment you control, including itself and any Auras you control that are attached to an opponent or to permanents controlled by an opponent."
    );
    supported("Ethereal Armor");
    supported("Pacifism");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Ethereal Armor", bears);
    assert_eq!(t.pt(bears), (3, 3));
    assert!(has_kw(&t, bears, KeywordKind::FirstStrike));
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Pacifism", giant);
    assert_eq!(t.pt(bears), (4, 4));
    // An enchantment P1 controls doesn't count.
    t.battlefield(P1, "Glorious Anthem");
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn ancestral_vengeance_still_puts_its_counter_when_the_creature_dies() {
    cr!("704.5f", "704.5m", "603.3");
    ruling!(
        "Ancestral Vengeance",
        "If Ancestral Vengeance causes the enchanted creature to have toughness 0, that creature and Ancestral Vengeance will be put into their owners' graveyards. The enters-the-battlefield ability will still apply."
    );
    supported("Ancestral Vengeance");
    supported("Llanowar Elves");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = in_hand_with_mana(&mut t, P0, "Ancestral Vengeance");
    t.cast(P0, aura).target(elves).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.in_graveyard(P0, "Ancestral Vengeance"));
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(t.pt(bears), (3, 3));
}
