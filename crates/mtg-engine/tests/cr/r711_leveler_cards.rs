//! CR 711: leveler cards.

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, Layout};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Student of Warfare ({W}, 1/1): "Level up {W}. LEVEL 2-6: 3/3, first strike. LEVEL 7+:
/// 4/4, double strike."
const STUDENT: &str = "Student of Warfare";

fn set_levels(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.objects[id.0 as usize]
        .counters
        .insert(counters::LEVEL.into(), n);
    t.g.recompute();
}

fn has(t: &TestGame, id: ObjectId, k: KeywordKind) -> bool {
    t.obj(id).chars.has_keyword(k)
}

#[test]
fn a_leveler_has_two_level_symbols_and_three_power_toughness_boxes() {
    cr!("711.1");
    supported(STUDENT);
    let def = card(STUDENT);
    assert_eq!(def.layout, Layout::Leveler);
    let abilities = &def.front().chars.abilities;
    // Level up, and one static ability for each of the two level symbols.
    assert!(abilities
        .iter()
        .any(|a| a.keyword().is_some_and(|k| k.kind == KeywordKind::LevelUp)));
    let level_statics = abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Static(s) if s.condition.is_some()))
        .count();
    assert_eq!(level_statics, 2);
    // Three power/toughness boxes: 1/1, 3/3 and 4/4.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, STUDENT);
    let mut seen = Vec::new();
    for n in [0, 2, 7] {
        set_levels(&mut t, s, n);
        seen.push(t.pt(s));
    }
    assert_eq!(seen, vec![(1, 1), (3, 3), (4, 4)]);
}

#[test]
fn a_level_range_symbol_applies_from_n1_to_n2_counters() {
    cr!("711.2", "711.2a", "711.3");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, STUDENT);
    for n in [2, 4, 6] {
        set_levels(&mut t, s, n);
        assert_eq!(t.pt(s), (3, 3), "level {n}");
        assert!(has(&t, s, KeywordKind::FirstStrike), "level {n}");
        assert!(!has(&t, s, KeywordKind::DoubleStrike), "level {n}");
    }
    // It sets the base power and toughness: modifications apply on top of it.
    t.g.add_counters(Entity::Object(s), counters::PLUS1, 1, None);
    t.g.recompute();
    assert_eq!(t.pt(s), (4, 4));
}

#[test]
fn a_level_plus_symbol_applies_from_n3_counters_on() {
    cr!("711.2", "711.2b", "711.3");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, STUDENT);
    for n in [7, 8, 20] {
        set_levels(&mut t, s, n);
        assert_eq!(t.pt(s), (4, 4), "level {n}");
        assert!(has(&t, s, KeywordKind::DoubleStrike), "level {n}");
        // The first striation's abilities don't apply any more.
        assert!(!has(&t, s, KeywordKind::FirstStrike), "level {n}");
    }
}

#[test]
fn below_the_first_level_symbol_it_has_its_uppermost_power_and_toughness() {
    cr!("711.5");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, STUDENT);
    for n in [0, 1] {
        set_levels(&mut t, s, n);
        assert_eq!(t.pt(s), (1, 1), "level {n}");
        assert!(!has(&t, s, KeywordKind::FirstStrike));
        assert!(!has(&t, s, KeywordKind::DoubleStrike));
    }
}

#[test]
fn in_other_zones_it_has_its_uppermost_power_and_toughness() {
    cr!("711.6");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, STUDENT);
    let in_gy = t.graveyard(P0, STUDENT);
    // Even with level counters on it (as a card in exile could have).
    let in_exile = t.exile(P0, STUDENT);
    t.g.objects[in_exile.0 as usize]
        .counters
        .insert(counters::LEVEL.into(), 7);
    t.g.recompute();
    for id in [in_hand, in_gy, in_exile] {
        let c = &t.obj(id).chars;
        assert_eq!((c.power, c.toughness), (Some(1), Some(1)));
        assert!(!c.has_keyword(KeywordKind::DoubleStrike));
    }
}
