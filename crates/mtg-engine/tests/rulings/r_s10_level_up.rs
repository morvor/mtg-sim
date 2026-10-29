//! Rulings batch S10 — level up (CR 702.87, leveler cards CR 711): "Level up [cost]
//! ([cost]: Put a level counter on this. Level up only as a sorcery.)" with level symbols
//! that give the leveler power, toughness, and abilities.

use crate::r_s01_common::{give_mana_for, supported};
use crate::r_s04_common::activate_named;
use crate::r_s05_common::run_from;
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;
use smol_str::SmolStr;

fn level_up(t: &mut TestGame, id: ObjectId) {
    activate_named(t, P0, id, "Level Up", 0).expect("level up");
    t.resolve();
}

fn levels(t: &TestGame, id: ObjectId) -> u32 {
    t.obj_now(id).counter(counters::LEVEL)
}

/// Puts `n` counters of `kind` on the permanent (as an effect other than level up does).
fn add(t: &mut TestGame, id: ObjectId, kind: &str, n: i32) {
    run_from(
        t,
        P0,
        None,
        Effect::AddCounters {
            what: Sel::Target(0),
            kind: SmolStr::new(kind),
            n: Value::c(n),
        },
        &[Entity::Object(id)],
    );
}

/// Removes `n` counters of `kind` from the permanent.
fn remove(t: &mut TestGame, id: ObjectId, kind: &str, n: i32) {
    run_from(
        t,
        P0,
        None,
        Effect::RemoveCounters {
            what: Sel::Target(0),
            kind: Some(kind.into()),
            n: Value::c(n),
        },
        &[Entity::Object(id)],
    );
}

/// P0 casts the real spell `name` targeting `target` and it resolves.
fn cast_on(t: &mut TestGame, name: &str, target: ObjectId) {
    supported(name);
    give_mana_for(t, P0, name);
    let card = t.hand(P0, name);
    t.cast(P0, card).target(target).go();
    t.resolve_all();
}

/// The permanent gets +p/+t until end of turn.
fn pump(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    run_from(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(p), Value::c(tough))],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn modifying_effects_apply_to_a_leveler_whenever_they_began() {
    cr!("702.87b", "711.2a", "613.4c", "613.4d");
    ruling!(
        "Student of Warfare",
        "Effects that modify a leveler’s power or toughness, such as the effects of Giant Growth or Glorious Anthem, will apply to it no matter when they started to take effect. The same is true for counters that change the creature’s power or toughness (such as +1/+1 counters) and effects that switch its power and toughness."
    );
    supported("Student of Warfare");
    // Student of Warfare (1/1; LEVEL 2-6: 3/3 first strike). At level 0: Giant Growth,
    // Glorious Anthem, a +1/+1 counter, and +1/+0 make it 7/6.
    let mut t = TestGame::new(2);
    let student = t.battlefield(P0, "Student of Warfare");
    cast_on(&mut t, "Giant Growth", student);
    t.battlefield(P0, "Glorious Anthem");
    add(&mut t, student, counters::PLUS1, 1);
    pump(&mut t, student, 1, 0);
    assert_eq!(t.pt(student), (7, 6));
    // Level 2: its base power and toughness are 3/3; all of them still apply.
    t.lands(P0, "Plains", 2);
    level_up(&mut t, student);
    level_up(&mut t, student);
    assert_eq!(t.pt(student), (9, 8));
    // Switching (Twisted Image) applies after them.
    cast_on(&mut t, "Twisted Image", student);
    assert_eq!(t.pt(student), (8, 9));
}

#[test]
fn modifying_effects_apply_to_a_leveler_whenever_they_began_straight() {
    cr!("702.87b", "711.2a", "613.4c", "613.4d");
    ruling!(
        "Brimstone Mage",
        "Effects that modify a leveler's power or toughness, such as the effects of Giant Growth or Glorious Anthem, will apply to it no matter when they started to take effect. The same is true for counters that change the creature's power or toughness (such as +1/+1 counters) and effects that switch its power and toughness."
    );
    supported("Brimstone Mage");
    // Brimstone Mage (2/2; LEVEL 1-2: 2/3; LEVEL 3+: 2/4).
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Brimstone Mage");
    cast_on(&mut t, "Giant Growth", mage);
    add(&mut t, mage, counters::PLUS1, 1);
    assert_eq!(t.pt(mage), (6, 6));
    cast_on(&mut t, "Twisted Image", mage);
    t.battlefield(P0, "Glorious Anthem");
    assert_eq!(t.pt(mage), (7, 7));
    // Level 3: base 2/4, then +3/+3, +1/+1 (counter), +1/+1 (Anthem): 7/9, switched: 9/7.
    add(&mut t, mage, counters::LEVEL, 3);
    assert_eq!(t.pt(mage), (9, 7));
}

/// The leveler `name` at `level` (base P/T `before`) gets base power and toughness 1/1
/// from Diminish; leveling up to `later` (base `after`) doesn't override it, since the
/// level symbol's ability has the leveler's own timestamp. At end of turn it's `after`.
fn set_effects_in_timestamp_order(
    name: &str,
    level: i32,
    before: (i32, i32),
    later: i32,
    after: (i32, i32),
) {
    supported(name);
    let mut t = TestGame::new(2);
    let leveler = t.battlefield(P0, name);
    add(&mut t, leveler, counters::LEVEL, level);
    assert_eq!(t.pt(leveler), before);
    cast_on(&mut t, "Diminish", leveler);
    assert_eq!(t.pt(leveler), (1, 1));
    add(&mut t, leveler, counters::LEVEL, later - level);
    assert_eq!(levels(&t, leveler), later as u32);
    assert_eq!(t.pt(leveler), (1, 1));
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert_eq!(t.pt(leveler), after);
}

#[test]
fn a_level_symbol_has_the_levelers_timestamp() {
    cr!("711.2a", "613.4b", "613.7a");
    ruling!(
        "Student of Warfare",
        "Effects that set a leveler’s power or toughness to a specific value, including the effects from a level symbol’s ability, apply in timestamp order. The timestamp of each level symbol’s ability is the same as the timestamp of the leveler itself, regardless of when the most recent level counter was put on it."
    );
    // Student of Warfare: LEVEL 2-6 3/3, LEVEL 7+ 4/4.
    set_effects_in_timestamp_order("Student of Warfare", 2, (3, 3), 7, (4, 4));
}

#[test]
fn a_level_symbol_has_the_levelers_timestamp_straight() {
    cr!("711.2a", "613.4b", "613.7a");
    ruling!(
        "Zulaport Enforcer",
        "Effects that set a leveler's power or toughness to a specific value, including the effects from a level symbol's ability, apply in timestamp order. The timestamp of each level symbol's ability is the same as the timestamp of the leveler itself, regardless of when the most recent level counter was put on it."
    );
    // Zulaport Enforcer: LEVEL 1-2 3/3, LEVEL 3+ 5/5.
    set_effects_in_timestamp_order("Zulaport Enforcer", 1, (3, 3), 3, (5, 5));
}

#[test]
fn a_copy_of_a_leveler_uses_its_own_level_counters() {
    cr!("702.87b", "707.2", "711.2a");
    ruling!(
        "Coralhelm Commander",
        "If another creature becomes a copy of a leveler, all of the leveler's printed abilities — including those represented by level symbols — are copied. The current characteristics of the leveler, and the number of level counters on it, are not."
    );
    supported("Coralhelm Commander");
    // Coralhelm Commander (2/2; LEVEL 2-3: 3/3 flying; LEVEL 4+: 4/4 flying, "Other
    // Merfolk creatures you control get +1/+1.").
    let mut t = TestGame::new(2);
    let commander = t.battlefield(P0, "Coralhelm Commander");
    add(&mut t, commander, counters::LEVEL, 4);
    assert_eq!(t.pt(commander), (4, 4));
    let bears = t.battlefield(P0, "Grizzly Bears");
    run_from(
        &mut t,
        P0,
        None,
        Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::All(Filter::Named("Coralhelm Commander".into())),
            duration: Duration::Permanent,
        },
        &[Entity::Object(bears)],
    );
    // A Merfolk now: the original's level 4 ability gives it +1/+1, but it has no level
    // counters of its own.
    assert_eq!(t.obj_now(bears).chars.name, "Coralhelm Commander");
    assert_eq!(levels(&t, bears), 0);
    assert_eq!(t.pt(bears), (3, 3));
    assert!(!t.obj_now(bears).has_keyword(KeywordKind::Flying));
    // Leveling the copy up gives it the level abilities.
    t.lands(P0, "Island", 2);
    level_up(&mut t, bears);
    level_up(&mut t, bears);
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
}

#[test]
fn a_levelers_level_is_its_number_of_level_counters() {
    cr!("702.87a", "711.2a", "711.5");
    ruling!(
        "Kargan Dragonlord",
        "A creature's level is based on how many level counters it has on it, not how many times its level up ability has been activated or has resolved. If a leveler gets level counters due to some other effect (such as Clockspinning) or loses level counters for some reason (such as Vampire Hexmage), its level is changed accordingly."
    );
    supported("Kargan Dragonlord");
    // Kargan Dragonlord (2/2; LEVEL 4-7: 4/4 flying; LEVEL 8+: 8/8 flying, trample).
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Kargan Dragonlord");
    add(&mut t, lord, counters::LEVEL, 4);
    assert_eq!(t.pt(lord), (4, 4));
    assert!(t.obj_now(lord).has_keyword(KeywordKind::Flying));
    add(&mut t, lord, counters::LEVEL, 4);
    assert_eq!(t.pt(lord), (8, 8));
    assert!(t.obj_now(lord).has_keyword(KeywordKind::Trample));
    remove(&mut t, lord, counters::LEVEL, 8);
    assert_eq!(t.pt(lord), (2, 2));
    assert!(!t.obj_now(lord).has_keyword(KeywordKind::Flying));
}

#[test]
fn a_levelers_abilities_dont_overwrite_level_up() {
    cr!("702.87a", "711.2b", "711.4");
    ruling!(
        "Transcendent Master",
        "The abilities a leveler grants to itself don't overwrite any other abilities it may have. In particular, they don't overwrite the creature's level up ability; it always has that."
    );
    supported("Transcendent Master");
    // Transcendent Master (LEVEL 12+: 9/9 lifelink, indestructible).
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Transcendent Master");
    add(&mut t, master, counters::LEVEL, 12);
    let o = t.obj_now(master);
    assert_eq!((o.power(), o.toughness()), (9, 9));
    assert!(o.has_keyword(KeywordKind::Lifelink));
    assert!(o.has_keyword(KeywordKind::Indestructible));
    assert!(o.has_keyword(KeywordKind::LevelUp));
    t.lands(P0, "Plains", 1);
    level_up(&mut t, master);
    assert_eq!(levels(&t, master), 13);
}
