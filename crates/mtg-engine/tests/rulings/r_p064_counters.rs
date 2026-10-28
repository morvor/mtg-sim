//! Rulings batch P064 — counters and life: Essence Channeler and Reluctant Role Model put
//! the counters a dying creature had (last known information, CR 603.10a) on another
//! creature; Serra Ascendant, Manor Gargoyle, Hero of Bretagard and Crystalline Giant.

use crate::r_p064_common::*;
use crate::r_s01_common::supported;
use crate::r_s14_common::triggers_from;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

const P11: &str = "+1/+1";
const M11: &str = "-1/-1";

// ---------------------------------------------------------------------------------------
// Essence Channeler
// ---------------------------------------------------------------------------------------

#[test]
fn essence_channeler_moves_its_minus_counters_too_killing_the_recipient() {
    cr!("603.10a", "704.5f", "122.1");
    ruling!(
        "Essence Channeler",
        "If Essence Channeler has -1/-1 counters on it when it dies, that ability will include those as well."
    );
    supported("Essence Channeler");
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    counters(&mut t, ec, M11, 2);
    assert!(!t.on_battlefield(ec));
    t.resolve_all();
    assert!(
        !t.on_battlefield(bears),
        "two -1/-1 counters put on the 2/2 kill it"
    );
}

#[test]
fn essence_channeler_dying_to_minus_counters_sees_both_kinds() {
    cr!("603.10a", "704.5f", "704.5q");
    ruling!(
        "Essence Channeler",
        "its last ability will see all of the +1/+1 counters it had when it died as well as the -1/-1 counters it had"
    );
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    let angel = t.battlefield(P0, "Serra Angel");
    counters(&mut t, ec, P11, 2);
    assert_eq!(t.pt(ec), (4, 3));
    t.answer_targets(P0, &[obj(angel)]);
    counters(&mut t, ec, M11, 5);
    assert!(!t.on_battlefield(ec));
    t.resolve_all();
    // Two +1/+1 and five -1/-1 counters were put on the Angel; they annihilate to three
    // -1/-1 counters (CR 704.5q).
    assert_eq!(t.counters(angel, P11), 0);
    assert_eq!(t.counters(angel, M11), 3);
    assert_eq!(t.pt(angel), (1, 1));
}

#[test]
fn essence_channeler_dealt_lethal_damage_as_you_gain_life_isnt_saved() {
    cr!("510.2", "704.5g", "603.2");
    ruling!(
        "Essence Channeler",
        "If Essence Channeler is dealt lethal damage at the same time that you gain life, it won’t receive a counter from its second ability in time to save it."
    );
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    let nighthawk = t.battlefield(P0, "Vampire Nighthawk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(
        &[(ec, Entity::Player(P1)), (nighthawk, Entity::Player(P1))],
        &[(bears, ec)],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 22, "lifelink gained life as the damage was dealt");
    assert!(!t.on_battlefield(ec), "it died before its life-gain trigger resolved");
    assert!(t.in_graveyard(P0, "Essence Channeler"));
}

#[test]
fn essence_channeler_triggers_once_for_life_gained_for_each() {
    cr!("119.9", "603.2c");
    ruling!(
        "Essence Channeler",
        "If you gain an amount of life “for each” of something, that life is gained as one event and Essence Channeler’s second ability triggers only once."
    );
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    t.lands(P0, "Forest", 4);
    crate::r_s25_common::cast_new(&mut t, P0, "Bountiful Harvest", &[]);
    t.resolve();
    assert!(t.life(P0) > 24, "gained a life for each land");
    assert_eq!(triggers_from(&t, ec), 1, "one trigger for the one life gain");
    t.resolve_all();
    assert_eq!(t.counters(ec, P11), 1);
}

#[test]
fn essence_channeler_ignores_life_your_teammate_gains() {
    cr!("810.9", "119.9");
    ruling!(
        "Essence Channeler",
        "In a Two-Headed Giant game, life gained by your teammate won’t cause Essence Channeler’s second ability to trigger"
    );
    let mut t = crate::r_s33_common::two_headed_giant();
    let ec = t.battlefield(P0, "Essence Channeler");
    t.g.gain_life(P1, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_from(&t, ec), 0);
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_from(&t, ec), 1, "your own life gain does trigger it");
}

#[test]
fn essence_channeler_and_the_ozolith_both_get_the_counters() {
    cr!("603.10a", "122.6");
    ruling!(
        "Essence Channeler",
        "if you control The Ozolith when Essence Channeler dies, you’ll put the appropriate number of each kind of counter onto both The Ozolith and the target creature"
    );
    supported("The Ozolith");
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    let oz = t.battlefield(P0, "The Ozolith");
    let bears = t.battlefield(P0, "Grizzly Bears");
    counters(&mut t, ec, P11, 2);
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s02_common::destroy(&mut t, ec);
    t.resolve_all();
    assert_eq!(t.counters(bears, P11), 2);
    assert_eq!(t.counters(oz, P11), 2);
}

#[test]
fn essence_channeler_stays_blocked_when_it_gains_flying() {
    cr!("509.1h", "506.4");
    ruling!(
        "Essence Channeler",
        "Once Essence Channeler has been blocked by a creature without flying or reach, losing life to give it flying and vigilance won’t cause it to become unblocked."
    );
    let mut t = TestGame::new(2);
    let ec = t.battlefield(P0, "Essence Channeler");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_p076_common::unblockable_after_blocked(&mut t, ec, bears, |t| {
        t.g.lose_life(P0, 1);
        t.g.flush_events();
        t.settle();
        assert!(flies(t, ec));
    });
}

// ---------------------------------------------------------------------------------------
// Reluctant Role Model
// ---------------------------------------------------------------------------------------

#[test]
fn reluctant_role_model_triggers_for_each_creature_dying_with_it() {
    cr!("603.10a", "603.2");
    ruling!(
        "Reluctant Role Model",
        "If Reluctant Role Model dies at the same time as one or more other creatures with counters on them, its last ability will trigger for each of those creatures."
    );
    supported("Reluctant Role Model");
    let mut t = TestGame::new(2);
    let rrm = t.battlefield(P0, "Reluctant Role Model");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Savannah Lions");
    let _plain = t.battlefield(P0, "Elvish Mystic");
    counters(&mut t, rrm, P11, 1);
    counters(&mut t, a, P11, 1);
    counters(&mut t, b, P11, 1);
    crate::r_s25_common::cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(
        triggers_from(&t, rrm),
        3,
        "itself and the two others with counters; not the creature without counters"
    );
}

#[test]
fn reluctant_role_model_includes_minus_counters_killing_the_target() {
    cr!("603.10a", "704.5f");
    ruling!(
        "Reluctant Role Model",
        "If the creature that died had -1/-1 counters on it when it died, Reluctant Role Model's last ability will include those as well."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reluctant Role Model");
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    counters(&mut t, bears, M11, 1);
    t.answer_targets(P0, &[obj(lions)]);
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert!(!t.on_battlefield(lions), "the 1/1 got a -1/-1 counter and died");
}

#[test]
fn reluctant_role_model_and_the_ozolith_both_get_the_counters() {
    cr!("603.10a", "122.6");
    ruling!(
        "Reluctant Role Model",
        "if you control The Ozolith when Reluctant Role Model dies, you'll put the appropriate number of each kind of counter onto both The Ozolith and the target creature"
    );
    let mut t = TestGame::new(2);
    let rrm = t.battlefield(P0, "Reluctant Role Model");
    let oz = t.battlefield(P0, "The Ozolith");
    let bears = t.battlefield(P0, "Grizzly Bears");
    counters(&mut t, rrm, "flying", 1);
    t.answer_targets(P0, &[obj(bears)]);
    crate::r_s02_common::destroy(&mut t, rrm);
    t.resolve_all();
    assert_eq!(t.counters(bears, "flying"), 1);
    assert_eq!(t.counters(oz, "flying"), 1);
}

#[test]
fn reluctant_role_model_puts_new_counters_so_hardened_scales_applies() {
    cr!("122.6", "614.1a");
    ruling!(
        "Reluctant Role Model",
        "Rather, you put the same number of each kind of counter the creature that died had when it died onto the target creature."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Reluctant Role Model");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    counters(&mut t, bears, P11, 2);
    t.battlefield(P0, "Hardened Scales");
    t.answer_targets(P0, &[obj(lions)]);
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(
        t.counters(lions, P11),
        3,
        "putting counters is modified by Hardened Scales"
    );
}

// ---------------------------------------------------------------------------------------
// Serra Ascendant, Manor Gargoyle
// ---------------------------------------------------------------------------------------

#[test]
fn serra_ascendant_lifelink_to_30_applies_before_lethal_damage_is_checked() {
    cr!("510.2", "704.3", "613.4c");
    ruling!(
        "Serra Ascendant",
        "apply its last effect before considering whether or not the damage dealt to it is lethal"
    );
    supported("Serra Ascendant");
    let mut t = TestGame::new(2);
    crate::r_s33_common::set_life(&mut t, P0, 29);
    let sa = t.battlefield(P0, "Serra Ascendant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(sa, Entity::Player(P1))], &[(bears, sa)]);
    assert_eq!(t.life(P0), 30);
    assert!(t.on_battlefield(sa), "it was a 6/6 with 2 damage");
    assert_eq!(t.pt(sa), (6, 6));
    let _ = bears;
}

#[test]
fn manor_gargoyle_losing_indestructible_after_lethal_damage_is_destroyed() {
    cr!("704.5g", "702.12b", "120.6");
    ruling!(
        "Manor Gargoyle",
        "If Manor Gargoyle loses indestructible after having been dealt lethal damage earlier in the turn, it will be destroyed."
    );
    supported("Manor Gargoyle");
    let mut t = TestGame::new(2);
    let mg = t.battlefield(P0, "Manor Gargoyle");
    let src = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, src, 5, mg);
    assert!(t.on_battlefield(mg), "indestructible");
    assert_eq!(t.obj_now(mg).damage, 5, "the damage stays marked");
    pool(&mut t, P0, ManaType::C, 1);
    activate_and_resolve(&mut t, P0, mg, "loses defender");
    assert!(!t.on_battlefield(mg), "destroyed by the marked lethal damage");
}

// ---------------------------------------------------------------------------------------
// Hero of Bretagard
// ---------------------------------------------------------------------------------------

#[test]
fn hero_of_bretagard_counts_counters_of_any_kind() {
    cr!("122.1", "613.1d", "613.1f");
    ruling!(
        "Hero of Bretagard",
        "Neither the second nor the third ability of Hero of Bretagard refers to a specific kind of counter. Any counter that's on it will do."
    );
    supported("Hero of Bretagard");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hero of Bretagard");
    counters(&mut t, h, "shield", 2);
    counters(&mut t, h, "oil", 2);
    assert!(!flies(&mut t, h));
    counters(&mut t, h, "stun", 1);
    assert!(flies(&mut t, h), "five counters of three kinds");
    assert!(is_a(&mut t, h, "Angel"));
    assert!(!has(&mut t, h, KeywordKind::Indestructible));
}

#[test]
fn hero_of_bretagard_with_ten_counters_is_an_angel_god_human_warrior() {
    cr!("613.1d", "613.1f", "205.1b");
    ruling!(
        "Hero of Bretagard",
        "With ten or more counters on it, Hero of Bretagard will have flying and indestructible"
    );
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hero of Bretagard");
    counters(&mut t, h, P11, 10);
    assert!(flies(&mut t, h));
    assert!(has(&mut t, h, KeywordKind::Indestructible));
    for s in ["Angel", "God", "Human", "Warrior"] {
        assert!(is_a(&mut t, h, s), "not a {s}");
    }
}

// ---------------------------------------------------------------------------------------
// Crystalline Giant
// ---------------------------------------------------------------------------------------

#[test]
fn crystalline_giant_puts_the_random_counter_as_its_ability_resolves() {
    cr!("608.2", "122.1b");
    ruling!(
        "Crystalline Giant",
        "You choose a kind of counter at random and put it on Crystalline Giant all while its ability is resolving."
    );
    supported("Crystalline Giant");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Crystalline Giant");
    crate::r_p057_common::into_beginning_of_combat(&mut t, P0);
    assert_eq!(triggers_from(&t, g), 1);
    let total = |t: &TestGame| t.obj_now(g).counters.values().sum::<u32>();
    assert_eq!(total(&t), 0);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(total(&t), 1, "the counter was put on as the ability resolved");
}

#[test]
fn crystalline_giant_can_get_a_flying_counter_while_it_already_flies() {
    cr!("122.1b");
    ruling!(
        "Crystalline Giant",
        "If Crystalline Giant has one of the listed keywords but not because of a counter of that kind, that kind of counter is still eligible to be chosen at random."
    );
    // Over many games with different random seeds, a Giant that flies (from Archetype of
    // Imagination, not from a counter) still sometimes gets a flying counter.
    let mut got_flying = false;
    for seed in 0..60u64 {
        let mut t = TestGame::with_config(
            2,
            mtg_engine::game::GameConfig {
                seed,
                ..Default::default()
            },
        );
        let g = t.battlefield(P0, "Crystalline Giant");
        t.battlefield(P0, "Archetype of Imagination");
        assert!(flies(&mut t, g));
        crate::r_p057_common::into_beginning_of_combat(&mut t, P0);
        t.resolve_all();
        if t.counters(g, "flying") == 1 {
            got_flying = true;
            break;
        }
    }
    assert!(got_flying, "the flying counter was never chosen");
}
