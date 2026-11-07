//! Rulings batch P226 — Isengard Unleashed: "Damage can't be prevented this turn. If a
//! source you control would deal damage this turn to an opponent or a permanent an
//! opponent controls, it deals triple that damage instead." Flashback (CR 702.34), and
//! trample assigning unmodified damage (CR 702.19b, 510.1c).

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn isengard_unleashed_triples_damage_to_opponents_and_trample_assigns_unmodified_damage() {
    cr!("702.19b", "510.1c", "614.1a", "615.12");
    ruling!(
        "Isengard Unleashed",
        "If a creature with trample you control would deal combat damage to a blocking creature this turn, you must assign its unmodified damage. For example, a 3/3 creature with trample blocked by a 2/2 creature can have 1 damage assigned to the defending player. It will then deal 6 damage to the blocking creature (2 tripled) and 3 to the defending player (1 tripled)."
    );
    supported("Isengard Unleashed");
    let mut t = TestGame::new(2);
    let mammoth = t.battlefield(P0, "War Mammoth");
    let mine = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Isengard Unleashed");
    let card = t.hand(P0, "Isengard Unleashed");
    t.cast(P0, card).go();
    t.resolve_all();
    // Not to P0's own permanents or P0: Shock deals 2.
    t.lands(P0, "Mountain", 2);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(mine).go();
    t.resolve_all();
    assert_eq!(t.obj_now(mine).damage, 2);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P0).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // P1's Holy Day doesn't prevent the combat damage. Lethal damage to the Bears is
    // assigned as 2; 1 tramples over: 6 to the Bears, 3 to P1.
    t.set_step(P0, Step::BeginningOfCombat);
    t.lands(P1, "Plains", 1);
    let holy = t.hand(P1, "Holy Day");
    t.cast(P1, holy).go();
    t.resolve_all();
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 1]));
    t.attack(&[(mammoth, Entity::Player(P1))], &[(bears, mammoth)]);
    let lethal: Vec<Vec<u32>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::AssignCombatDamage { lethal, .. } => Some(lethal),
            _ => None,
        })
        .collect();
    assert_eq!(lethal, vec![vec![2]]);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
    assert!(t.g.log.iter().any(|l| l.text.contains("deals 6 damage")));
    // Next turn it's over: Shock deals 2 to P1.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn isengard_unleashed_has_flashback() {
    cr!("702.34a");
    // Flashback {4}{R}{R}{R}.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Isengard Unleashed");
    t.lands(P0, "Mountain", 6);
    let flashback = CastMethod::Keyword(KeywordKind::Flashback);
    assert!(t.cast(P0, card).method(flashback.clone()).try_go().is_err());
    t.lands(P0, "Mountain", 1);
    t.cast(P0, card).method(flashback).go();
    t.resolve_all();
    assert!(t.in_exile("Isengard Unleashed"));
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
}
