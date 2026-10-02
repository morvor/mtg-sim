//! Rulings batch P209 — enchant (CR 303.4, 702.5): Auras whose continuous effects change
//! the enchanted permanent's characteristics, and how they interact with the layer system
//! (CR 613): losing abilities, setting types, colors and power/toughness.

use crate::r_p209_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn activated_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

// ---------------------------------------------------------------------------------------
// Ray of Frost
// ---------------------------------------------------------------------------------------

#[test]
fn ray_of_frost_removes_prior_abilities_of_a_red_creature_but_not_later_ones() {
    cr!("613.1f", "613.7", "611.3a");
    ruling!(
        "Ray of Frost",
        "If the enchanted creature is red (or if it becomes red), it loses all abilities that it had prior to the time Ray of Frost became attached to it."
    );
    supported("Ray of Frost");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let goblin = t.battlefield(P1, "Raging Goblin");
    assert!(has_kw(&t, goblin, KeywordKind::Haste));
    cast_aura(&mut t, P0, "Ray of Frost", goblin).unwrap();
    assert!(!has_kw(&t, goblin, KeywordKind::Haste));
    // An ability granted afterwards works.
    cast_spell(&mut t, P1, "Jump", &[Entity::Object(goblin)]);
    t.resolve_all();
    assert!(has_kw(&t, goblin, KeywordKind::Flying));
    assert!(!has_kw(&t, goblin, KeywordKind::Haste));
    // A nonred creature keeps its abilities.
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P1, "Wind Drake");
    attach_new(&mut t, P0, "Ray of Frost", bird);
    assert!(has_kw(&t, bird, KeywordKind::Flying));
}

#[test]
fn ray_of_frost_turns_off_a_power_defining_ability() {
    cr!("604.3", "613.4a", "613.1f");
    ruling!(
        "Ray of Frost",
        "If the enchanted creature is red and has a characteristic-defining ability that specifies its power and/or toughness, that ability will not have its effect. In that case, its power and/or toughness (as appropriate) become 0."
    );
    supported("Kolaghan Forerunners");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P1, "Kolaghan Forerunners");
    t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(k), (2, 3));
    attach_new(&mut t, P0, "Ray of Frost", k);
    assert_eq!(t.pt(k), (0, 3));
}

#[test]
fn ray_of_frost_doesnt_stop_a_type_defining_ability() {
    cr!("604.3", "613.1d", "702.73a");
    ruling!(
        "Ray of Frost",
        "If the enchanted creature is red and has a characteristic-defining ability that specifies its types or colors, that ability will still have its effect because it's applied before the creature loses the ability."
    );
    supported("War-Spike Changeling");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P1, "War-Spike Changeling");
    attach_new(&mut t, P0, "Ray of Frost", c);
    assert!(!has_kw(&t, c, KeywordKind::Changeling));
    assert!(t.obj_now(c).chars.all_creature_types);
    assert!(t.obj_now(c).chars.has_subtype("Elf"));
}

// ---------------------------------------------------------------------------------------
// Lignify, Reprobation, Kenrith's Transformation, Darksteel Mutation
// ---------------------------------------------------------------------------------------

#[test]
fn lignify_removes_abilities_from_before_but_not_after() {
    cr!("613.1f", "613.7");
    ruling!(
        "Lignify",
        "Lignify causes the enchanted creature to lose all abilities it had at the time it became enchanted. Any abilities granted to the creature after Lignify entered the battlefield will work normally."
    );
    supported("Lignify");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let drake = t.battlefield(P1, "Wind Drake");
    cast_aura(&mut t, P0, "Lignify", drake).unwrap();
    assert!(!has_kw(&t, drake, KeywordKind::Flying));
    assert_eq!(t.pt(drake), (0, 4));
    assert!(t.obj_now(drake).chars.has_subtype("Treefolk"));
    assert!(!t.obj_now(drake).chars.has_subtype("Drake"));
    cast_spell(&mut t, P1, "Jump", &[Entity::Object(drake)]);
    t.resolve_all();
    assert!(has_kw(&t, drake, KeywordKind::Flying));
}

#[test]
fn lignify_overwrites_earlier_creature_type_changes_but_not_later_ones() {
    cr!("613.1d", "613.7", "205.1b");
    ruling!(
        "Lignify",
        "Lignify overwrites previous effects that changed the enchanted creature's creature types. Any such effects that apply after Lignify enters will work normally."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Kenrith's Transformation", bears);
    assert!(t.obj_now(bears).chars.has_subtype("Elk"));
    attach_new(&mut t, P0, "Lignify", bears);
    assert!(!t.obj_now(bears).chars.has_subtype("Elk"));
    assert!(t.obj_now(bears).chars.has_subtype("Treefolk"));
    // Inner Demon, later: a Demon in addition.
    attach_new(&mut t, P0, "Inner Demon", bears);
    assert!(t.obj_now(bears).chars.has_subtype("Treefolk"));
    assert!(t.obj_now(bears).chars.has_subtype("Demon"));
}

#[test]
fn lignify_doesnt_overwrite_modifications_of_power_and_toughness() {
    cr!("613.4b", "613.4c", "613.4d");
    ruling!(
        "Lignify",
        "Lignify will not overwrite effects such as those from Glorious Anthem, counters, or Giant Growth that change the enchanted creature's power or toughness without setting it to a specific value."
    );
    supported("Glorious Anthem");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Glorious Anthem");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    cast_spell(&mut t, P1, "Giant Growth", &[Entity::Object(bears)]);
    t.resolve_all();
    attach_new(&mut t, P0, "Lignify", bears);
    // 0/4, +1/+1 (Anthem), +1/+1 (counter), +3/+3 (Giant Growth).
    assert_eq!(t.pt(bears), (5, 9));
}

#[test]
fn lignify_overwrites_earlier_power_setting_effects_but_not_later_ones() {
    cr!("613.4b", "613.7");
    ruling!(
        "Lignify",
        "Lignify will overwrite any previous effects that set the enchanted creature's power or toughness (such as Serendib Sorcerer does). Any such effects that apply after Lignify entered will work normally."
    );
    supported("Serendib Sorcerer");
    // Earlier: Lignify wins.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sorc = t.battlefield(P0, "Serendib Sorcerer");
    t.activate(P0, sorc, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (0, 2));
    attach_new(&mut t, P0, "Lignify", bears);
    assert_eq!(t.pt(bears), (0, 4));
    // Later: the Sorcerer wins.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sorc = t.battlefield(P0, "Serendib Sorcerer");
    attach_new(&mut t, P0, "Lignify", bears);
    t.activate(P0, sorc, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (0, 2));
}

#[test]
fn lignify_turns_off_the_creatures_own_static_abilities() {
    cr!("613.1f", "611.3a");
    ruling!(
        "Lignify",
        "Lignify will overwrite static abilities that come from the creature itself (such as the one on Verduran Enchantress) because Lignify causes the creature to lose that ability before it gets a chance to apply."
    );
    supported("Benalish Marshal");
    let mut t = TestGame::new(2);
    let marshal = t.battlefield(P1, "Benalish Marshal");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    attach_new(&mut t, P0, "Lignify", marshal);
    assert_eq!(t.pt(bears), (2, 2));
    // Verduran Enchantress's ability is gone too: no card drawn.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let ench = t.battlefield(P1, "Verduran Enchantress");
    attach_new(&mut t, P0, "Lignify", ench);
    let hand = t.hand_size(P1);
    t.library_top(P1, "Forest");
    t.answer_yes(P1, true);
    cast_aura(&mut t, P1, "Holy Strength", ench).unwrap();
    assert_eq!(t.hand_size(P1), hand);
}

#[test]
fn reprobation_overwrites_earlier_type_and_pt_settings_but_not_later_ones() {
    cr!("613.1d", "613.4b", "613.7", "205.1a");
    ruling!(
        "Reprobation",
        "Reprobation overwrites all previous effects that set the enchanted creature’s creature types, card types, power, and/or toughness to specific values."
    );
    supported("Reprobation");
    // Earlier Lignify: Reprobation wins.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Lignify", bears);
    attach_new(&mut t, P0, "Reprobation", bears);
    assert_eq!(t.pt(bears), (0, 1));
    assert!(t.obj_now(bears).chars.has_subtype("Coward"));
    assert!(!t.obj_now(bears).chars.has_subtype("Treefolk"));
    // Earlier artifact type: lost.
    let karn = t.battlefield(P1, "Karn, Silver Golem");
    attach_new(&mut t, P0, "Reprobation", karn);
    assert!(!t.obj_now(karn).chars.is(CardType::Artifact));
    assert!(t.obj_now(karn).chars.is_legendary());
    // Later Lignify: Lignify wins.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Reprobation", bears);
    attach_new(&mut t, P0, "Lignify", bears);
    assert_eq!(t.pt(bears), (0, 4));
    assert!(t.obj_now(bears).chars.has_subtype("Treefolk"));
    assert!(!t.obj_now(bears).chars.has_subtype("Coward"));
}

#[test]
fn kenriths_transformation_makes_a_green_elk_keeping_supertypes() {
    cr!("613.1d", "613.1e", "205.1a", "205.4a");
    ruling!(
        "Kenrith's Transformation",
        "Kenrith's Transformation overwrites all colors and creature types the enchanted creature has. It's just a green Elk."
    );
    supported("Kenrith's Transformation");
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P1, "Karn, Silver Golem");
    attach_new(&mut t, P0, "Kenrith's Transformation", karn);
    let c = &t.obj_now(karn).chars;
    assert!(c.is_legendary());
    assert!(!c.is(CardType::Artifact));
    assert!(c.is_creature());
    assert!(c.has_subtype("Elk"));
    assert!(!c.has_subtype("Golem"));
    assert_eq!(c.colors, ColorSet::single(Color::Green));
    assert_eq!(t.pt(karn), (3, 3));
    assert!(t.obj_now(karn).chars.has_no_abilities());
}

/// Crews `vehicle` (P0's) with a Grizzly Bears, then attaches `aura` to it: once the crew
/// effect ends (cleanup), the vehicle is still what the Aura says.
fn crewed_vehicle_with(aura: &str) -> (TestGame, ObjectId) {
    supported(aura);
    supported("Smuggler's Copter");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert!(is_creature(&t, copter));
    cast_aura(&mut t, P0, aura, copter).expect("the Aura couldn't be cast");
    (t, copter)
}

#[test]
fn kenriths_transformation_keeps_a_temporary_creature_an_elk() {
    cr!("613.1d", "702.122a", "514.2");
    ruling!(
        "Kenrith's Transformation",
        "Kenrith's Transformation may enchant a permanent that is only temporarily a creature, such as Enchanted Carriage."
    );
    let (mut t, copter) = crewed_vehicle_with("Kenrith's Transformation");
    t.advance_to(P1, Step::Upkeep);
    assert!(is_creature(&t, copter));
    assert!(t.obj_now(copter).chars.has_subtype("Elk"));
    assert!(!t.obj_now(copter).chars.is(CardType::Artifact));
    assert_eq!(t.pt(copter), (3, 3));
}

#[test]
fn one_with_the_stars_keeps_a_temporary_creature_an_enchantment() {
    cr!("613.1d", "702.122a", "514.2");
    ruling!(
        "One with the Stars",
        "One with the Stars may enchant a permanent that is only temporarily a creature, such as a Vehicle."
    );
    let (mut t, copter) = crewed_vehicle_with("One with the Stars");
    assert!(!is_creature(&t, copter));
    assert!(t.obj_now(copter).chars.is(CardType::Enchantment));
    t.advance_to(P1, Step::Upkeep);
    let c = &t.obj_now(copter).chars;
    assert!(c.is(CardType::Enchantment));
    assert!(!c.is(CardType::Artifact));
    assert!(!c.is_creature());
    assert!(t.named_on_battlefield("One with the Stars").len() == 1);
}

#[test]
fn darksteel_mutation_keeps_artifact_subtypes_only() {
    cr!("205.1b", "613.1d", "205.3d");
    ruling!(
        "Darksteel Mutation",
        "If it had any other artifact subtypes (such as Equipment), it will retain those."
    );
    supported("Darksteel Mutation");
    supported("Lion Sash");
    let mut t = TestGame::new(2);
    let sash = t.battlefield(P1, "Lion Sash");
    attach_new(&mut t, P0, "Darksteel Mutation", sash);
    let c = &t.obj_now(sash).chars;
    assert!(c.has_subtype("Equipment"));
    assert!(c.has_subtype("Insect"));
    assert!(!c.has_subtype("Cat"));
    assert!(c.is(CardType::Artifact) && c.is_creature());
    // Dryad Arbor loses its land type, and Forest with it.
    let arbor = t.battlefield(P1, "Dryad Arbor");
    attach_new(&mut t, P0, "Darksteel Mutation", arbor);
    let c = &t.obj_now(arbor).chars;
    assert!(!c.is_land());
    assert!(!c.has_subtype("Forest"));
    assert!(!c.has_subtype("Dryad"));
    assert_eq!(t.pt(arbor), (0, 1));
    assert!(has_kw(&t, arbor, KeywordKind::Indestructible));
}

#[test]
fn imprisoned_in_the_moon_keeps_land_types_but_only_taps_for_colorless() {
    cr!("305.6", "613.1d", "613.1f");
    ruling!(
        "Imprisoned in the Moon",
        "If the enchanted permanent is a land and has land types, it retains those types even though it loses any intrinsic mana abilities associated with them."
    );
    supported("Imprisoned in the Moon");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let plains = t.battlefield(P1, "Plains");
    attach_new(&mut t, P0, "Imprisoned in the Moon", plains);
    assert!(t.obj_now(plains).chars.has_subtype("Plains"));
    assert_eq!(activated_count(&t, plains), 1);
    t.activate(P1, plains, 0, &[]).unwrap();
    let pool = &t.g.player(P1).mana_pool;
    assert_eq!(pool.count(ManaType::C), 1);
    assert_eq!(pool.count(ManaType::W), 0);
}

#[test]
fn natures_embrace_on_a_land_creature_gives_both_benefits() {
    cr!("611.3a", "305.6");
    ruling!(
        "Nature's Embrace",
        "If the enchanted permanent is both a land and a creature, it gets both benefits."
    );
    supported("Nature's Embrace");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let arbor = t.battlefield(P0, "Dryad Arbor");
    attach_new(&mut t, P0, "Nature's Embrace", arbor);
    assert_eq!(t.pt(arbor), (3, 3));
    assert_eq!(activated_count(&t, arbor), 2);
    activate_containing(&mut t, P0, arbor, "two mana").unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

// ---------------------------------------------------------------------------------------
// Mutate, Nyx Infusion, Bonds of Faith, Swift Reconfiguration
// ---------------------------------------------------------------------------------------

#[test]
fn mystic_subdual_isnt_undone_by_mutating() {
    cr!("702.140e", "730.2c", "613.1f");
    ruling!(
        "Mystic Subdual",
        "Mutating the enchanted creature won’t cause it to gain abilities."
    );
    supported("Mystic Subdual");
    supported("Gemrazer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P1, "Mystic Subdual", bears);
    assert_eq!(t.pt(bears), (0, 2));
    let gem = t.hand(P0, "Gemrazer");
    add_mana(&mut t, P0, ManaType::G, 2);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, gem)
        .method(CastMethod::Keyword(KeywordKind::Mutate))
        .target(bears)
        .go();
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.resolve();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Gemrazer");
    assert_eq!(t.pt(bears), (2, 4));
    assert!(!has_kw(&t, bears, KeywordKind::Reach));
    assert!(!has_kw(&t, bears, KeywordKind::Trample));
}

#[test]
fn nyx_infusion_rechecks_whether_the_creature_is_an_enchantment() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Nyx Infusion",
        "Nyx Infusion continuously checks to see whether the creature it's enchanting is an enchantment or not."
    );
    supported("Nyx Infusion");
    supported("Leafcrown Dryad");
    let mut t = TestGame::new(2);
    let dryad = t.battlefield(P1, "Leafcrown Dryad");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P1, "Nyx Infusion", dryad);
    attach_new(&mut t, P0, "Nyx Infusion", bears);
    assert_eq!(t.pt(dryad), (4, 4));
    t.settle();
    // The Bears aren't an enchantment: -2/-2, and they die.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let _ = bears;
    // The Dryad stops being an enchantment: -2/-2 now.
    attach_new(&mut t, P0, "Kenrith's Transformation", dryad);
    assert_eq!(t.pt(dryad), (1, 1));
}

#[test]
fn bonds_of_faith_attacker_that_stops_being_human_keeps_attacking() {
    cr!("506.4", "611.3a");
    ruling!(
        "Bonds of Faith",
        "Once the enchanted creature has been declared as an attacking or blocking creature, causing it to stop being a Human won’t remove it from combat. It will lose the +2/+2 bonus, however."
    );
    supported("Bonds of Faith");
    supported("Elite Vanguard");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Elite Vanguard");
    attach_new(&mut t, P0, "Bonds of Faith", v);
    assert_eq!(t.pt(v), (4, 3));
    attack_with(&mut t, &[(v, Entity::Player(P1))]);
    attach_new(&mut t, P1, "Lignify", v);
    t.settle();
    assert!(t.g.is_attacking(v));
    assert_eq!(t.pt(v), (0, 4));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn swift_reconfiguration_removes_the_creature_from_combat() {
    cr!("506.4", "301.7b");
    ruling!(
        "Swift Reconfiguration",
        "If the enchanted permanent was attacking or blocking (or being attacked if it's a planeswalker) when Swift Reconfiguration becomes attached to it, that permanent is removed from combat."
    );
    supported("Swift Reconfiguration");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    assert!(t.g.is_attacking(bears));
    cast_aura(&mut t, P0, "Swift Reconfiguration", bears).unwrap();
    assert!(!is_creature(&t, bears));
    assert!(!t.g.is_attacking(bears));
}

// ---------------------------------------------------------------------------------------
// Granted abilities
// ---------------------------------------------------------------------------------------

#[test]
fn lavamancers_skill_on_a_wizard_gives_both_abilities() {
    cr!("611.3a", "602.1");
    ruling!(
        "Lavamancer's Skill",
        "If this card enchants a Wizard, the Wizard has both abilities and its controller can use either one."
    );
    supported("Lavamancer's Skill");
    for (two, life) in [(false, 1), (true, 2)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let w = t.battlefield(P0, "Prodigal Sorcerer");
        let target = t.battlefield(P1, "Hill Giant");
        attach_new(&mut t, P0, "Lavamancer's Skill", w);
        assert_eq!(activated_count(&t, w), 3);
        let needle = if two { "deals 2 damage" } else { "deals 1 damage to target creature" };
        t.answer_targets(P0, &[Entity::Object(target)]);
        activate_containing(&mut t, P0, w, needle).unwrap();
        t.resolve_all();
        assert_eq!(t.obj_now(target).damage as i32, life);
    }
}

#[test]
fn curators_ward_hexproof_on_an_opponents_permanent() {
    cr!("702.11b", "702.11c");
    ruling!(
        "Curator's Ward",
        "If you give hexproof to an opponent’s permanent, such as by enchanting it with Curator’s Ward, that player can still target that permanent, but you can’t."
    );
    supported("Curator's Ward");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Curator's Ward", bears);
    assert!(!spell_targets(&mut t, P0, "Shock").contains(&Entity::Object(bears)));
    assert!(spell_targets(&mut t, P1, "Shock").contains(&Entity::Object(bears)));
}

#[test]
fn flickering_ward_choosing_white_drops_other_white_auras() {
    cr!("702.16c", "704.5m", "702.16a");
    ruling!(
        "Flickering Ward",
        "If you choose White, the enchanted creature will have Protection from White. This will cause any other white Auras attached to that creature to be put into their owner's graveyard, but Flickering Ward will remain on the battlefield."
    );
    supported("Flickering Ward");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Holy Strength", bears);
    let i = Color::ALL.iter().position(|c| *c == Color::White).unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let ward = cast_aura(&mut t, P0, "Flickering Ward", bears).unwrap();
    t.settle();
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert_eq!(attached_to(&t, ward), Some(Entity::Object(bears)));
}

#[test]
fn essence_leak_on_a_permanent_neither_red_nor_green_does_nothing() {
    cr!("611.3a", "303.4");
    ruling!(
        "Essence Leak",
        "It can enchant a permanent that is not red or green, but it doesn’t do anything in that case."
    );
    supported("Essence Leak");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let drake = t.battlefield(P1, "Wind Drake");
    let n = t.obj_now(drake).chars.abilities.len();
    cast_aura(&mut t, P0, "Essence Leak", drake).expect("can enchant it");
    assert_eq!(t.obj_now(drake).chars.abilities.len(), n);
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A green one gets the ability.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let n = t.obj_now(bears).chars.abilities.len();
    attach_new(&mut t, P0, "Essence Leak", bears);
    assert_eq!(t.obj_now(bears).chars.abilities.len(), n + 1);
}

#[test]
fn inner_demon_shrinks_only_non_demons_on_the_battlefield_when_it_resolves() {
    cr!("611.2c", "608.2h");
    ruling!(
        "Inner Demon",
        "Inner Demon's triggered ability affects only non-Demon creatures on the battlefield at the time it resolves."
    );
    supported("Inner Demon");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P1, "Hill Giant");
    cast_aura(&mut t, P0, "Inner Demon", giant).unwrap();
    assert_eq!(t.pt(giant), (5, 5));
    assert_eq!(t.pt(other), (1, 1));
    let late = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(late), (2, 2));
}

#[test]
fn scourge_of_the_nobilis_pump_survives_the_aura_leaving() {
    cr!("113.7a", "611.2a");
    ruling!(
        "Scourge of the Nobilis",
        "Once the +1/+0 ability is activated, it exists independently of the creature and Scourge of the Nobilis."
    );
    supported("Scourge of the Nobilis");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let scourge = attach_new(&mut t, P0, "Scourge of the Nobilis", goblin);
    assert_eq!(t.pt(goblin), (2, 2));
    add_mana(&mut t, P0, ManaType::R, 1);
    activate_containing(&mut t, P0, goblin, "+1/+0").unwrap();
    destroy(&mut t, scourge);
    assert_eq!(t.pt(goblin), (1, 1));
    t.resolve_all();
    assert_eq!(t.pt(goblin), (2, 1));
}
