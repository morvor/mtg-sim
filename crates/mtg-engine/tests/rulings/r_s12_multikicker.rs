//! Rulings batch S12 — multikicker (CR 702.33c): "You may pay an additional [cost] any
//! number of times as you cast this spell." Comet Storm and Strength of the Tajuru have
//! one target plus another for each time they were kicked.

use crate::r_s01_common::*;
use crate::r_s04_common::untapped_lands;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The targets of the spell `spell` on the stack.
fn spell_targets(t: &TestGame, spell: ObjectId) -> Vec<Entity> {
    t.obj(spell)
        .stack
        .as_ref()
        .map(|si| {
            si.chosen
                .iter()
                .flat_map(|c| c.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_target_for_each_kick_each_one_different() {
    cr!("702.33c", "601.2b", "601.2c", "115.3");
    ruling!("Comet Storm", "Each target you choose must be different.");
    ruling!("Strength of the Tajuru", "Each target you choose must be different.");
    supported("Comet Storm");
    supported("Strength of the Tajuru");
    // Comet Storm ({X}{R}{R}, multikicker {1}) kicked twice with X = 2: three targets.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 6);
    let storm = t.hand(P0, "Comet Storm");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    let from = t.asked().len();
    let spell = t
        .cast(P0, storm)
        .x(2)
        .targets(&[
            Entity::Object(bears),
            Entity::Object(giant),
            Entity::Player(P1),
        ])
        .go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // One choice of up to three targets.
    let maxes: Vec<u32> = asked_since(&t, from)
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![3]);
    assert_eq!(spell_targets(&t, spell).len(), 3);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.life(P1), 18);
    // The same target can't be chosen twice: an answer naming the Bears twice isn't
    // legal, and the spell doesn't get the Bears as two of its targets.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let storm = t.hand(P0, "Comet Storm");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    let spell = t
        .cast(P0, storm)
        .x(2)
        .targets(&[Entity::Object(bears), Entity::Object(bears)])
        .go();
    let chosen = spell_targets(&t, spell);
    let mut distinct = chosen.clone();
    distinct.sort();
    distinct.dedup();
    assert!(!chosen.is_empty() && chosen.len() < 2);
    assert_eq!(distinct, chosen);
    // Strength of the Tajuru ({X}{G}{G}, multikicker {1}) kicked once with X = 2: two
    // different target creatures get two +1/+1 counters each.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Forest", 5);
    let sot = t.hand(P0, "Strength of the Tajuru");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(1));
    t.cast(P0, sot)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert_eq!(t.counters(b, counters::PLUS1), 2);
}
