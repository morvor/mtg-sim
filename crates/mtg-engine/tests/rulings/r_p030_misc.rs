//! Rulings batch P030 — other rulings of cards that copy: Thespian's Stage (CR 707.2,
//! 707.4, 611.2a), storm triggers that are countered (CR 702.40), Auras that set base power
//! and toughness (CR 613.4b), effects that affect only the objects there as they resolve
//! (CR 611.2c), "until this leaves the battlefield" (CR 610.3b), last known information
//! for X (CR 608.2h, 107.1b), the defending player of a creature attacking a planeswalker
//! (CR 506.2), myriad granted by a lieutenant ability (CR 702.116a).

use crate::r_p030_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::{activate_containing, attach_new, damage};
use crate::r_s25_common::cast_new;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::*;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

/// Whether the permanent has a mana ability whose text contains `text`.
fn has_ability(t: &TestGame, id: ObjectId, text: &str) -> bool {
    t.obj_now(id).chars.abilities.iter().any(|a| a.text.contains(text))
}

/// Thespian's Stage ("{T}: Add {C}. {2}, {T}: This land becomes a copy of target land,
/// except it has this ability.") becomes a copy of `land`.
fn stage_copies(t: &mut TestGame, stage: ObjectId, land: ObjectId) {
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[obj(land)]);
    activate_containing(t, P0, stage, "becomes a copy").unwrap();
    t.resolve_all();
    assert_eq!(t.obj_now(stage).chars.name, t.obj_now(land).chars.name);
}

// --- Thespian's Stage ---------------------------------------------------------------------

#[test]
fn thespian_s_stage_copies_an_animated_land_unanimated() {
    cr!("707.2", "613.2");
    ruling!(
        "Thespian's Stage",
        "Notably, if you copy a land that is also a creature because of a temporary effect (such as Celestial Colonnade), Thespian's Stage will become just the \"unanimated\" land."
    );
    supported("Thespian's Stage");
    supported("Celestial Colonnade");
    let mut t = TestGame::new(2);
    let stage = t.battlefield(P0, "Thespian's Stage");
    let col = t.battlefield(P0, "Celestial Colonnade");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, col, "becomes a 4/4").unwrap();
    t.resolve_all();
    assert!(is_creature(&t, col));
    t.g.untap(stage);
    stage_copies(&mut t, stage, col);
    assert!(!is_creature(&t, stage));
    assert!(is_creature(&t, col));
}

#[test]
fn thespian_s_stage_doesnt_enter_so_nothing_applies_as_it_enters() {
    cr!("707.2", "603.6a", "614.1c");
    ruling!(
        "Thespian's Stage",
        "No enters-the-battlefield abilities of the land Thespian's Stage is copying will trigger. Thespian's Stage was already on the battlefield."
    );
    supported("Khalni Garden");
    supported("Gemstone Mine");
    // Khalni Garden: "When this land enters, create a 0/1 green Plant creature token."
    // Gemstone Mine: "This land enters with three mining counters on it."
    let mut t = TestGame::new(2);
    let stage = t.battlefield(P0, "Thespian's Stage");
    let garden = t.battlefield(P0, "Khalni Garden");
    stage_copies(&mut t, stage, garden);
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Plant").is_empty());
    let mut t = TestGame::new(2);
    let stage = t.battlefield(P0, "Thespian's Stage");
    let mine = t.battlefield(P0, "Gemstone Mine");
    stage_copies(&mut t, stage, mine);
    assert_eq!(t.counters(stage, "mining"), 0);
}

#[test]
fn thespian_s_stage_s_copy_effect_has_no_duration() {
    cr!("707.2", "611.2a", "707.9a");
    ruling!(
        "Thespian's Stage",
        "The copy effect created by the last activated ability doesn't have a duration. It will last until Thespian's Stage leaves the battlefield or another copy effect overwrites it. The permanent will no longer have the first ability of Thespian's Stage."
    );
    let mut t = TestGame::new(2);
    let stage = t.battlefield(P0, "Thespian's Stage");
    let forest = t.battlefield(P1, "Forest");
    stage_copies(&mut t, stage, forest);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(t.obj_now(stage).chars.name, "Forest");
    assert!(has_ability(&t, stage, "becomes a copy"));
    assert!(!has_ability(&t, stage, "{C}"));
}

#[test]
fn thespian_s_stage_stays_tapped_when_it_becomes_a_copy() {
    cr!("707.2", "110.5");
    ruling!(
        "Thespian's Stage",
        "Thespian's Stage doesn't become untapped when it becomes a copy, even if the target land is untapped."
    );
    let mut t = TestGame::new(2);
    let stage = t.battlefield(P0, "Thespian's Stage");
    let forest = t.battlefield(P1, "Forest");
    stage_copies(&mut t, stage, forest);
    assert!(t.obj_now(stage).tapped);
    assert!(!t.obj_now(forest).tapped);
}

// --- Chatterstorm --------------------------------------------------------------------------

#[test]
fn a_countered_storm_trigger_makes_no_copies() {
    cr!("702.40a", "701.6a");
    ruling!(
        "Chatterstorm",
        "The triggered ability that creates the copies can itself be countered by anything that can counter a triggered ability. If it is countered, no copies will be put onto the stack."
    );
    supported("Chatterstorm");
    supported("Stifle");
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Chatterstorm", &[]);
    t.settle();
    let storm = *t.g.stack.last().unwrap();
    assert!(matches!(
        t.obj(storm).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { .. })
    ));
    cast_new(&mut t, P1, "Stifle", &[obj(storm)]);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Squirrel").len(), 1);
}

// --- Amphibian Downpour --------------------------------------------------------------------

#[test]
fn amphibian_downpour_overwrites_earlier_pt_setting_effects_only() {
    cr!("613.4b", "613.7");
    ruling!(
        "Amphibian Downpour",
        "Amphibian Downpour overwrites all previous effects that set the creature's base power and toughness to specific values. Any power- or toughness-setting effects that start to apply afterward will overwrite this effect."
    );
    supported("Amphibian Downpour");
    // "Enchanted creature loses all abilities and is a blue Frog creature with base power
    // and toughness 1/1."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let set = |n| Modification::SetPT(Some(Value::c(n)), Some(Value::c(n)));
    modify_until_eot(&mut t, bears, vec![set(5)]);
    assert_eq!(t.pt(bears), (5, 5));
    attach_new(&mut t, P0, "Amphibian Downpour", bears);
    assert_eq!(t.pt(bears), (1, 1));
    modify_until_eot(&mut t, bears, vec![set(4)]);
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn amphibian_downpour_can_make_marked_damage_lethal() {
    cr!("704.5g", "120.6");
    ruling!(
        "Amphibian Downpour",
        "nonlethal damage dealt to a creature may become lethal if Amphibian Downpour becomes attached to it during that turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    damage(&mut t, giant, 1, bears);
    assert!(t.on_battlefield(bears));
    cast_new(&mut t, P0, "Amphibian Downpour", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

// --- Labyrinth Guardian --------------------------------------------------------------------

#[test]
fn an_aura_spell_targets_labyrinth_guardian() {
    cr!("115.1b", "303.4a");
    ruling!(
        "Labyrinth Guardian",
        "Aura spells you cast target the permanent they will enchant."
    );
    supported("Labyrinth Guardian");
    // "When this creature becomes the target of a spell, sacrifice it."
    let mut t = TestGame::new(2);
    let lg = t.battlefield(P1, "Labyrinth Guardian");
    cast_new(&mut t, P0, "Pacifism", &[obj(lg)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Labyrinth Guardian"));
    assert!(t.in_graveyard(P0, "Pacifism"));
}

// --- Effects that affect only what's there as they resolve -----------------------------------

#[test]
fn flowerfoot_swordmaster_pumps_only_the_mice_there_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Flowerfoot Swordmaster",
        "Mice you begin to control later in the turn or creatures that become Mice later in the turn won't be affected."
    );
    supported("Flowerfoot Swordmaster");
    // "Valiant — Whenever this creature becomes the target of a spell or ability you
    // control for the first time each turn, Mice you control get +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Flowerfoot Swordmaster");
    let b = t.battlefield(P0, "Flowerfoot Swordmaster");
    cast_new(&mut t, P0, "Mutagenic Growth", &[obj(a)]);
    t.resolve_all();
    assert_eq!(t.pt(a), (4, 4));
    assert_eq!(t.pt(b), (2, 2));
    let c = t.battlefield(P0, "Flowerfoot Swordmaster");
    assert_eq!(t.pt(c), (1, 2));
}

#[test]
fn haze_of_rage_pumps_only_the_creatures_there_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Haze of Rage",
        "Haze of Rage affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get +1/+0."
    );
    supported("Haze of Rage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Haze of Rage", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
    let giant = t.battlefield(P0, "Hill Giant");
    assert_eq!(t.pt(giant), (3, 3));
}

// --- Angel of Sanctions ---------------------------------------------------------------------

#[test]
fn angel_of_sanctions_leaving_first_means_nothing_is_exiled() {
    cr!("610.3", "610.3b");
    ruling!(
        "Angel of Sanctions",
        "If Angel of Sanctions leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled."
    );
    supported("Angel of Sanctions");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    let angel = t.enter(P0, "Angel of Sanctions");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, angel);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

// --- Coastline Marauders ---------------------------------------------------------------------

#[test]
fn coastline_marauders_counts_the_planeswalker_controller_s_lands() {
    cr!("506.2", "508.5");
    ruling!(
        "Coastline Marauders",
        "If Coastline Marauders is attacking a planeswalker, that planeswalker's controller is the defending player."
    );
    supported("Coastline Marauders");
    // "Whenever this creature attacks, it gets +1/+0 until end of turn for each land
    // defending player controls."
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Coastline Marauders");
    let pw = t.battlefield(P1, "Chandra, the Firebrand");
    t.lands(P1, "Mountain", 3);
    t.lands(P0, "Mountain", 5);
    attack_with(&mut t, &[(m, obj(pw))]);
    t.resolve_all();
    assert_eq!(t.pt(m).0, 3);
}

// --- Resilient Khenra -------------------------------------------------------------------------

/// `p`'s Resilient Khenra enters ("When this creature enters, you may have target creature
/// get +X/+X until end of turn, where X is this creature's power."), targeting `target`;
/// its trigger is on the stack.
fn khenra_enters(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Resilient Khenra");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(target)]);
    let k = t.enter(P0, "Resilient Khenra");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    k
}

#[test]
fn resilient_khenra_s_x_uses_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Resilient Khenra",
        "If Resilient Khenra leaves the battlefield before its triggered ability resolves, use its power as it last existed on the battlefield to determine the value of X."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let k = khenra_enters(&mut t, bears);
    modify_until_eot(&mut t, k, vec![Modification::ModifyPT(Value::c(3), Value::c(3))]);
    kill(&mut t, k);
    t.resolve_all();
    assert_eq!(t.pt(bears), (7, 7));
}

#[test]
fn resilient_khenra_s_negative_power_is_x_of_0() {
    cr!("107.1b", "608.2h");
    ruling!(
        "Resilient Khenra",
        "If Resilient Khenra's power is negative as its triggered ability resolves, X is considered to be 0."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let k = khenra_enters(&mut t, bears);
    modify_until_eot(&mut t, k, vec![Modification::ModifyPT(Value::c(-5), Value::c(0))]);
    assert_eq!(t.pt(k).0, -3);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}

// --- Ironwill Forger ------------------------------------------------------------------------

#[test]
fn ironwill_forger_s_myriad_tokens_enter_as_the_copied_creature() {
    cr!("702.116a", "707.2", "603.6a", "614.1c");
    ruling!(
        "Ironwill Forger",
        "Any “enters” abilities of the copied creature will trigger when the tokens enter. Any “as [this creature] enters” or “[this creature] enters with” abilities of the copied creature will also work."
    );
    supported("Ironwill Forger");
    // "Lieutenant — At the beginning of combat on your turn, if you control your commander,
    // target nonlegendary creature you control gains myriad until end of turn."
    for (name, draws) in [("Elvish Visionary", 1), ("Chronozoa", 0)] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Ironwill Forger");
        let cmdr = t.battlefield(P0, "Grizzly Bears");
        t.g.objects[cmdr.0 as usize].is_commander = true;
        let c = t.battlefield(P0, name);
        t.answer_targets(P0, &[obj(c)]);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.resolve_all();
        let hand = t.hand_size(P0);
        t.answer_yes(P0, true);
        attack_with(&mut t, &[(c, Entity::Player(P1))]);
        t.resolve_all();
        let toks = tokens_named(&t, P0, name);
        assert_eq!(toks.len(), 1, "{name}");
        assert_eq!(t.hand_size(P0), hand + draws);
        if name == "Chronozoa" {
            assert_eq!(t.counters(toks[0], counters::TIME), 3);
        }
    }
}
