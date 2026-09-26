//! CR 702.80 Wither.

use crate::common_k702_011_017::{assert_supported, attack_with, bf, custom_card};
use crate::common_k702_018_026::declare_blocks;
use crate::common_k702_052_066::{destroy, run_effect};
use crate::k702_001_010_common::grant;
use mtg_engine::ability::*;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Deals `n` damage from `source` to `target` (as an effect would).
fn deal(t: &mut TestGame, source: ObjectId, target: Entity, n: i32) {
    let to = match target {
        Entity::Object(_) => Sel::Target(1),
        Entity::Player(p) => Sel::Players(PlayerRef::Player(p)),
    };
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(source)], vec![target]];
    t.g.exec(
        &Effect::DealDamage {
            source: Sel::Target(0),
            amount: Value::c(n),
            to,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

#[test]
fn wither_damage_to_a_creature_is_minus_one_counters() {
    cr!("702.80", "702.80a", "120.3d");
    assert_supported("Boggart Ram-Gang");
    let mut t = TestGame::new(2);
    // Boggart Ram-Gang: 3/3 haste, wither. Craw Wurm: 6/4.
    let gang = t.battlefield(P0, "Boggart Ram-Gang");
    let wurm = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(gang, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(wurm, gang)]);
    t.advance_to(P0, Step::EndOfCombat);
    // No damage marked: three -1/-1 counters instead.
    assert_eq!(t.obj_now(wurm).damage, 0);
    assert_eq!(t.counters(wurm, counters::MINUS1), 3);
    assert_eq!(t.pt(wurm), (3, 1));
    // They stay after the turn ends (damage would have been removed).
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.pt(wurm), (3, 1));
    // The Ram-Gang was dealt ordinary damage and died.
    assert!(!t.on_battlefield(gang));
}

#[test]
fn wither_damage_to_players_and_planeswalkers_is_normal() {
    cr!("702.80a");
    ruling!(
        "Puncture Blast",
        "if it deals damage to a player or planeswalker, the damage behaves normally"
    );
    let mut t = TestGame::new(2);
    let gang = t.battlefield(P0, "Boggart Ram-Gang");
    let jace = t.battlefield(P1, "Jace Beleren");
    deal(&mut t, gang, Entity::Player(P1), 3);
    assert_eq!(t.life(P1), 17);
    deal(&mut t, gang, Entity::Object(jace), 2);
    assert_eq!(t.counters(jace, counters::LOYALTY), 1);
    assert_eq!(t.counters(jace, counters::MINUS1), 0);
}

#[test]
fn a_wither_source_that_left_its_zone_uses_last_known_information() {
    cr!("702.80b");
    let def = custom_card(
        "Spiteful Wretch",
        "Creature — Elemental",
        Some((1, 1)),
        "Wither\nWhen this creature dies, it deals 2 damage to target creature.",
    );
    let mut t = TestGame::new(2);
    let wretch = bf(&mut t, P0, def);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    destroy(&mut t, wretch);
    t.resolve_all();
    // It's in the graveyard as it deals the damage: as it last existed it had wither.
    assert_eq!(t.counters(wurm, counters::MINUS1), 2);
    assert_eq!(t.obj_now(wurm).damage, 0);
}

#[test]
fn wither_works_from_any_zone() {
    cr!("702.80c");
    ruling!(
        "Puncture Blast",
        "If it deals damage to a creature, the damage results in -1/-1 counters"
    );
    assert_supported("Puncture Blast");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 3);
    let blast = t.hand(P0, "Puncture Blast");
    t.cast(P0, blast).target(wurm).go();
    t.resolve_all();
    // An instant on the stack with wither.
    assert_eq!(t.counters(wurm, counters::MINUS1), 3);
    assert_eq!(t.obj_now(wurm).damage, 0);
    assert_eq!(t.pt(wurm), (3, 1));
}

#[test]
fn multiple_instances_of_wither_are_redundant() {
    cr!("702.80d");
    let mut t = TestGame::new(2);
    let gang = t.battlefield(P0, "Boggart Ram-Gang");
    grant(&mut t, gang, Keyword::new(KeywordKind::Wither));
    assert_eq!(t.obj_now(gang).chars.keyword_count(KeywordKind::Wither), 2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    deal(&mut t, gang, Entity::Object(wurm), 3);
    assert_eq!(t.counters(wurm, counters::MINUS1), 3);
}

#[test]
fn wither_damage_can_kill_by_reducing_toughness() {
    cr!("702.80a", "704.5f");
    let mut t = TestGame::new(2);
    let gang = t.battlefield(P0, "Boggart Ram-Gang");
    let bears = t.battlefield(P1, "Grizzly Bears");
    deal(&mut t, gang, Entity::Object(bears), 2);
    t.settle();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn all_damage_can_be_dealt_as_though_its_source_had_wither() {
    cr!("702.80a", "120.3d");
    assert_supported("Everlasting Torment");
    let mut t = TestGame::new(2);
    // "All damage is dealt as though its source had wither."
    t.battlefield(P1, "Everlasting Torment");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    deal(&mut t, giant, Entity::Object(wurm), 3);
    assert_eq!(t.counters(wurm, counters::MINUS1), 3);
    assert_eq!(t.obj_now(wurm).damage, 0);
    deal(&mut t, giant, Entity::Player(P1), 3);
    assert_eq!(t.life(P1), 17);
}
