//! Rulings batch P064 — continuous effects that make things fly: animation effects and
//! their timestamps (CR 613.1d, 613.4b, 613.7), "can't have or gain" (Archetype of
//! Imagination), conditional statics re-checked continuously (Benalish Commander, Aeon
//! Chronicler), and other characteristic rulings.

use crate::r_p064_common::*;
use crate::r_s01_common::supported;
use crate::r_s14_common::triggers_from;
use crate::r_s29_common::cast_and_resolve;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::{Mana, ManaRestriction, ManaType};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Animation effects and timestamps
// ---------------------------------------------------------------------------------------

#[test]
fn chimeric_sphere_later_animation_wins_and_loses_flying() {
    cr!("613.7", "613.4b", "613.1f");
    ruling!(
        "Chimeric Sphere",
        "If the first ability is activated and resolves, then the second ability is activated and resolves, Chimeric Sphere will be a 3/2 creature. It won’t have flying."
    );
    supported("Chimeric Sphere");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Chimeric Sphere");
    pool(&mut t, P0, ManaType::C, 4);
    activate_and_resolve(&mut t, P0, s, "2/1");
    assert!(flies(&mut t, s));
    activate_and_resolve(&mut t, P0, s, "3/2");
    assert_eq!(t.pt(s), (3, 2));
    assert!(!flies(&mut t, s));
}

/// P0's Synthesizer Labship with two charge counters (its 2+ ability is active), and P0
/// moves to the beginning of combat with its trigger aimed at `target` on the stack.
fn labship_targets(t: &mut TestGame, target: ObjectId) -> ObjectId {
    let l = t.battlefield(P0, "Synthesizer Labship");
    counters(t, l, "charge", 2);
    t.answer_targets(P0, &[obj(target)]);
    crate::r_p057_common::into_beginning_of_combat(t, P0);
    assert_eq!(triggers_from(t, l), 1);
    l
}

#[test]
fn synthesizer_labship_sets_base_pt_of_a_creature_counters_still_apply() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Synthesizer Labship",
        "If the target of Synthesizer Labship’s second ability was already a creature, its base power and toughness will become 2/2 until end of turn."
    );
    supported("Synthesizer Labship");
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Ornithopter");
    counters(&mut t, o, "+1/+1", 1);
    labship_targets(&mut t, o);
    t.resolve_all();
    assert_eq!(t.pt(o), (3, 3), "base 2/2 plus its +1/+1 counter");
    // A P/T-setting effect that starts later overwrites it.
    cast_and_resolve(&mut t, P0, "Diminish", &[obj(o)]);
    assert_eq!(t.pt(o), (2, 2), "base 1/1 plus the counter");
}

#[test]
fn synthesizer_labship_animating_a_vehicle_doesnt_crew_it() {
    cr!("702.122b", "702.122c");
    ruling!(
        "Synthesizer Labship",
        "it doesn’t count as “crewing” that Vehicle for any ability that would trigger due to a Vehicle becoming crewed."
    );
    supported("Ghost Ark");
    let mut t = TestGame::new(2);
    let ark = t.battlefield(P0, "Ghost Ark");
    labship_targets(&mut t, ark);
    t.resolve_all();
    assert!(crate::r_s20_common::creature_now(&mut t, ark));
    assert_eq!(t.pt(ark), (2, 2));
    assert_eq!(triggers_from(&t, ark), 0, "it didn't become crewed");
}

#[test]
fn daxos_s_torment_overrides_earlier_pt_setting_effects() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Daxos's Torment",
        "If the triggered ability resolves while Daxos’s Torment is already a creature, that ability will override any effects that set its power or toughness to a specific value."
    );
    supported("Daxos's Torment");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Daxos's Torment");
    t.enter(P0, "Glorious Anthem");
    t.resolve_all();
    assert_eq!(t.pt(d), (6, 6), "5/5 plus the Anthem");
    cast_and_resolve(&mut t, P0, "Diminish", &[obj(d)]);
    cast_and_resolve(&mut t, P0, "Giant Growth", &[obj(d)]);
    assert_eq!(t.pt(d), (5, 5));
    t.enter(P0, "Glorious Anthem");
    t.resolve_all();
    assert_eq!(
        t.pt(d),
        (10, 10),
        "5/5 again; Giant Growth and two Anthems still apply"
    );
}

#[test]
fn answered_prayers_keeps_gaining_life_once_its_a_creature() {
    cr!("613.1d", "603.2");
    ruling!(
        "Answered Prayers",
        "Once Answered Prayers becomes a creature, you’ll still gain life whenever a creature enters the battlefield under your control."
    );
    supported("Answered Prayers");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Answered Prayers");
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert!(crate::r_s20_common::creature_now(&mut t, a));
    assert_eq!(t.life(P0), 21);
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.pt(a), (3, 3));
}

// ---------------------------------------------------------------------------------------
// Archetype of Imagination
// ---------------------------------------------------------------------------------------

#[test]
fn archetype_of_imagination_applies_to_creatures_entering_later() {
    cr!("611.3a", "613.1f");
    ruling!(
        "Archetype of Imagination",
        "The Archetype's second ability applies to each creature controlled by any of your opponents, no matter when it entered the battlefield."
    );
    supported("Archetype of Imagination");
    let mut t = TestGame::new(2);
    let before = t.battlefield(P1, "Serra Angel");
    t.battlefield(P0, "Archetype of Imagination");
    let after = t.enter(P1, "Serra Angel");
    t.settle();
    assert!(!flies(&mut t, before));
    assert!(!flies(&mut t, after));
}

#[test]
fn archetype_of_imagination_stops_grants_from_being_created_at_all() {
    cr!("611.2a", "613.1f", "101.2");
    ruling!(
        "Archetype of Imagination",
        "While you control an Archetype, continuous effects generated by the resolution of spells and abilities that would give the specified ability to creatures your opponents control aren't created."
    );
    let mut t = TestGame::new(2);
    let arch = t.battlefield(P0, "Archetype of Imagination");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P1's Jump gives the Bears flying and nothing else: no effect is created.
    t.set_step(P1, Step::PrecombatMain);
    cast_and_resolve(&mut t, P1, "Jump", &[obj(bears)]);
    assert!(!flies(&mut t, bears));
    crate::r_s02_common::destroy(&mut t, arch);
    assert!(
        !flies(&mut t, bears),
        "the Jump effect was never created, so the Bears don't fly later in the turn"
    );
    // Other effects of such a spell apply as normal: Mighty Leap's +2/+2.
    let mut t = TestGame::new(2);
    let arch = t.battlefield(P0, "Archetype of Imagination");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_and_resolve(&mut t, P1, "Mighty Leap", &[obj(bears)]);
    assert_eq!(t.pt(bears), (4, 4));
    crate::r_s02_common::destroy(&mut t, arch);
    assert!(!flies(&mut t, bears));
    assert_eq!(t.pt(bears), (4, 4));
}

// ---------------------------------------------------------------------------------------
// Ozai, the Phoenix King
// ---------------------------------------------------------------------------------------

#[test]
fn ozai_turns_restricted_mana_red_keeping_its_restriction() {
    cr!("106.4", "106.6", "614.1a");
    ruling!(
        "Ozai, the Phoenix King",
        "If unspent mana has any restrictions or riders associated with it"
    );
    supported("Ozai, the Phoenix King");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ozai, the Phoenix King");
    let mut m = Mana::new(ManaType::G);
    m.restriction = Some(ManaRestriction::SpellOfType(CardType::Creature));
    t.g.players[0].mana_pool.add(m);
    t.advance_to(P0, Step::BeginningOfCombat);
    let pool = &t.g.player(P0).mana_pool.mana;
    assert_eq!(pool.len(), 1, "the mana wasn't lost");
    assert_eq!(pool[0].ty, ManaType::R);
    assert_eq!(
        pool[0].restriction,
        Some(ManaRestriction::SpellOfType(CardType::Creature))
    );
}

// ---------------------------------------------------------------------------------------
// Characteristic-defining and conditional abilities
// ---------------------------------------------------------------------------------------

#[test]
fn benalish_commander_counts_itself_and_marked_damage_can_become_lethal() {
    cr!("604.3", "704.5g", "120.6");
    ruling!(
        "Benalish Commander",
        "As long as Benalish Commander is on the battlefield, its first ability will count itself."
    );
    ruling!(
        "Benalish Commander",
        "nonlethal damage dealt to Benalish Commander may become lethal if other Soldiers you control leave the battlefield during that turn."
    );
    supported("Benalish Commander");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Benalish Commander");
    t.g.recompute();
    assert_eq!(t.pt(b), (1, 1));
    let s1 = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    let s2 = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    t.g.recompute();
    assert_eq!(t.pt(b), (3, 3));
    let src = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, src, 1, b);
    assert!(t.on_battlefield(b));
    crate::r_s02_common::destroy(&mut t, s1);
    assert!(t.on_battlefield(b), "1 damage on a 2/2");
    crate::r_s02_common::destroy(&mut t, s2);
    assert!(!t.on_battlefield(b), "the marked damage became lethal");
}

#[test]
fn aeon_chronicler_marked_damage_becomes_lethal_as_cards_leave_your_hand() {
    cr!("604.3", "704.5g", "120.6");
    ruling!(
        "Aeon Chronicler",
        "nonlethal damage dealt to Aeon Chronicler may become lethal if cards leave your hand during that turn."
    );
    supported("Aeon Chronicler");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Aeon Chronicler");
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    let f = t.hand(P0, "Forest");
    t.g.recompute();
    let src = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, src, 2, a);
    assert!(t.on_battlefield(a));
    t.play_land(P0, f).unwrap();
    t.g.flush_events();
    t.settle();
    assert!(!t.on_battlefield(a), "a 2/2 with 2 damage");
}

#[test]
fn urzas_avenger_effects_last_until_end_of_turn() {
    cr!("514.2", "611.2a");
    ruling!(
        "Urza's Avenger",
        "The -1/-1 is not permanent. It lasts until the end of the turn as do the abilities that are granted."
    );
    supported("Urza's Avenger");
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Urza's Avenger");
    crate::r_s35_common::pick_options(&mut t, P0, &["flying"]);
    activate_and_resolve(&mut t, P0, u, "-1/-1");
    assert_eq!(t.pt(u), (3, 3));
    assert!(flies(&mut t, u));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(u), (4, 4));
    assert!(!flies(&mut t, u));
}

#[test]
fn sarcomite_myr_is_a_blue_artifact_creature() {
    cr!("105.2", "202.2");
    ruling!(
        "Sarcomite Myr",
        "Sarcomite Myr is an artifact creature, just like Ornithopter."
    );
    supported("Sarcomite Myr");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Sarcomite Myr");
    let o = t.obj_now(m);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert_eq!(crate::r_s05_common::colors(&t, m), ColorSet::single(Color::Blue));
}

#[test]
fn spacecraft_is_a_type_with_no_intrinsic_abilities() {
    cr!("205.3g");
    ruling!(
        "Atmospheric Greenhouse",
        "Spacecraft is an artifact type with no special meaning."
    );
    supported("Atmospheric Greenhouse");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Atmospheric Greenhouse");
    assert!(is_a(&mut t, g, "Spacecraft"));
    assert!(!t.obj_now(g).is(CardType::Creature));
    assert!(!flies(&mut t, g));
}

// ---------------------------------------------------------------------------------------
// Redundant instances
// ---------------------------------------------------------------------------------------

#[test]
fn multiple_instances_of_granted_keywords_are_redundant() {
    cr!("702.9c", "702.15f");
    ruling!(
        "Triton Waverider",
        "Multiple instances of flying on the same creature are redundant."
    );
    ruling!(
        "Multiform Wonder",
        "Multiple instances of flying, vigilance, or lifelink on the same creature are redundant."
    );
    ruling!(
        "Majestic Myriarch",
        "Multiple instances of any of the abilities Majestic Myriarch can gain are redundant."
    );
    supported("Triton Waverider");
    supported("Multiform Wonder");
    // Triton Waverider: two enchantments enter, two triggers resolve; it attacks as a
    // creature with flying (one instance's worth: a Bears with reach can block it).
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Triton Waverider");
    t.enter(P0, "Glorious Anthem");
    t.enter(P0, "Glorious Anthem");
    t.resolve_all();
    assert!(flies(&mut t, w));
    // Multiform Wonder: lifelink twice still gains life once per damage.
    let mut t = TestGame::new(2);
    let m = t.enter(P0, "Multiform Wonder");
    t.resolve_all();
    let cur = t.g.current(m);
    t.g.objects[cur.0 as usize].summoning_sick = false;
    for _ in 0..2 {
        crate::r_s35_common::pick_options(&mut t, P0, &["lifelink"]);
        activate_and_resolve(&mut t, P0, m, "your choice of flying");
    }
    assert!(has(&mut t, m, KeywordKind::Lifelink));
    t.attack(&[(m, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P0), 23, "3 damage, 3 life: not 6");
    // Majestic Myriarch with two flying creatures gains one flying (redundant anyway).
    let mut t = TestGame::new(2);
    let my = t.battlefield(P0, "Majestic Myriarch");
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P0, "Wind Drake");
    crate::r_p057_common::into_beginning_of_combat(&mut t, P0);
    t.resolve_all();
    assert!(flies(&mut t, my));
    t.attack(&[(my, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 14, "a 6/6 dealing damage once");
}
