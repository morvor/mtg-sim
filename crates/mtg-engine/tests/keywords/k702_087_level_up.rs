//! CR 702.87 Level up.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::run_effect;
use mtg_engine::ability::*;
use mtg_engine::card::{card, Layout};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;
use smol_str::SmolStr;

fn level_up(t: &mut TestGame, p: PlayerId, id: ObjectId) -> Result<(), casting::Illegal> {
    activate_named(t, p, id, "Level Up", 0).map(|_| ())
}

fn levels(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).counter(counters::LEVEL)
}

fn put_counters(t: &mut TestGame, id: ObjectId, kind: &str, n: i32) {
    let id = t.g.current(id);
    run_effect(
        t,
        None,
        P0,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: SmolStr::new(kind),
            n: Value::c(n),
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn level_up_puts_a_level_counter_on_the_permanent() {
    cr!("702.87", "702.87a");
    assert_supported("Student of Warfare");
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Student of Warfare");
    t.lands(P0, "Plains", 2);
    level_up(&mut t, P0, student).expect("level up");
    // It uses the stack.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(levels(&t, student), 0);
    t.resolve();
    assert_eq!(levels(&t, student), 1);
    level_up(&mut t, P0, student).expect("level up again");
    t.resolve();
    assert_eq!(levels(&t, student), 2);
}

#[test]
fn level_up_can_be_activated_only_as_a_sorcery() {
    cr!("702.87a");
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Student of Warfare");
    t.lands(P0, "Plains", 3);
    // Not while the stack isn't empty.
    level_up(&mut t, P0, student).unwrap();
    assert!(level_up(&mut t, P0, student).is_err());
    t.resolve();
    // Not during combat.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(level_up(&mut t, P0, student).is_err());
    // Not during an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(level_up(&mut t, P0, student).is_err());
    t.set_step(P0, Step::PostcombatMain);
    assert!(level_up(&mut t, P0, student).is_ok());
}

#[test]
fn a_leveler_cards_level_symbols_set_its_power_toughness_and_abilities() {
    cr!("702.87b");
    let c = card("Student of Warfare");
    assert_eq!(c.layout, Layout::Leveler);
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Student of Warfare");
    t.lands(P0, "Plains", 7);
    assert_eq!(t.pt(student), (1, 1));
    level_up(&mut t, P0, student).unwrap();
    t.resolve();
    // Level 1: still its uppermost power/toughness box.
    assert_eq!(t.pt(student), (1, 1));
    level_up(&mut t, P0, student).unwrap();
    t.resolve();
    // LEVEL 2-6: 3/3 first strike.
    assert_eq!(t.pt(student), (3, 3));
    assert!(t.obj_now(student).has_keyword(KeywordKind::FirstStrike));
    for _ in 0..5 {
        level_up(&mut t, P0, student).unwrap();
        t.resolve();
    }
    // LEVEL 7+: 4/4 double strike, no longer first strike.
    assert_eq!(levels(&t, student), 7);
    assert_eq!(t.pt(student), (4, 4));
    assert!(t.obj_now(student).has_keyword(KeywordKind::DoubleStrike));
    assert!(!t.obj_now(student).has_keyword(KeywordKind::FirstStrike));
}

#[test]
fn level_counters_from_any_source_count_and_level_up_is_always_available() {
    cr!("702.87a", "702.87b", "711.4");
    ruling!(
        "Student of Warfare",
        "A creature’s level is based on how many level counters it has on it, not how many times its level up ability has been activated or has resolved."
    );
    ruling!(
        "Student of Warfare",
        "The abilities a leveler grants to itself don’t overwrite any other abilities it may have. In particular, they don’t overwrite the creature’s level up ability; it always has that."
    );
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Student of Warfare");
    put_counters(&mut t, student, counters::LEVEL, 7);
    assert_eq!(t.pt(student), (4, 4));
    // Still has level up at level 7.
    assert!(t
        .obj_now(student)
        .chars
        .keywords()
        .any(|k| k.kind == KeywordKind::LevelUp));
    t.lands(P0, "Plains", 1);
    level_up(&mut t, P0, student).unwrap();
    t.resolve();
    assert_eq!(levels(&t, student), 8);
    // Losing level counters lowers its level.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::RemoveCounters {
            what: Sel::Target(0),
            kind: Some(counters::LEVEL.into()),
            n: Value::c(5),
        },
        &[Entity::Object(student)],
    );
    assert_eq!(t.pt(student), (3, 3));
    // Other modifications apply on top of the level's base power and toughness.
    put_counters(&mut t, student, counters::PLUS1, 1);
    assert_eq!(t.pt(student), (4, 4));
}

#[test]
fn a_copy_of_a_leveler_has_its_level_abilities_but_not_its_counters() {
    cr!("702.87b");
    ruling!(
        "Student of Warfare",
        "If another creature becomes a copy of a leveler, all of the leveler’s printed abilities — including those represented by level symbols — are copied. The current characteristics of the leveler, and the number of level counters on it, are not."
    );
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Student of Warfare");
    put_counters(&mut t, student, counters::LEVEL, 3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(student), (3, 3));
    run_effect(
        &mut t,
        None,
        P0,
        Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::All(Filter::Named("Student of Warfare".into())),
            duration: Duration::Permanent,
        },
        &[Entity::Object(bears)],
    );
    assert_eq!(t.obj_now(bears).chars.name, "Student of Warfare");
    assert_eq!(t.pt(bears), (1, 1));
    t.lands(P0, "Plains", 2);
    level_up(&mut t, P0, bears).unwrap();
    t.resolve();
    level_up(&mut t, P0, bears).unwrap();
    t.resolve();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn class_levels_dont_interact_with_level_counters() {
    cr!("702.87c", "711.7");
    assert_supported("Barbarian Class");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Barbarian Class");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    // Level counters don't give the Class a level.
    put_counters(&mut t, class, counters::LEVEL, 3);
    assert_eq!(t.obj_now(class).class_level.max(1), 1);
    // Level 3: "Creatures you control have haste." — not gained.
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Haste));
    // Gaining a class level doesn't put level counters on it.
    t.lands(P0, "Mountain", 2);
    activate_named(&mut t, P0, class, "{1}{R}: Level 2", 0).expect("level 2");
    t.resolve();
    assert_eq!(t.obj_now(class).class_level, 2);
    assert_eq!(levels(&t, class), 3);
}
