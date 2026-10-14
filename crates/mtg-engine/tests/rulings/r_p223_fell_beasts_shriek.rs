//! Rulings batch P223 — Fell Beast's Shriek ("Each opponent chooses a creature they
//! control. Tap and goad the chosen creatures. Splice onto instant or sorcery {2}{U}{R}")
//! and the "[player] chooses a creature they control" pattern (Imperial Edict, Wei
//! Assassins); splice (CR 702.47).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::{choice_candidates, in_hand_with_mana};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

fn goaded_by(t: &TestGame, id: ObjectId) -> Vec<PlayerId> {
    t.obj_now(id).goaded_by.clone()
}

fn splice(t: &mut TestGame, p: PlayerId, yes: bool) {
    t.answer(p, DecisionKind::OptionalCost, Answer::Bool(yes));
}

/// Mana for Lightning Bolt with `n` Fell Beast's Shrieks spliced onto it.
fn bolt_with_shrieks(t: &mut TestGame, n: usize) -> ObjectId {
    let bolt = in_hand_with_mana(t, P0, "Lightning Bolt");
    for _ in 0..n {
        t.lands(P0, "Island", 1);
        t.lands(P0, "Mountain", 1);
        t.lands(P0, "Wastes", 2);
        t.hand(P0, "Fell Beast's Shriek");
        splice(t, P0, true);
    }
    bolt
}

#[test]
fn each_opponent_chooses_a_creature_which_is_tapped_and_goaded() {
    cr!("701.15a", "701.26a", "101.4", "115.10a");
    supported("Fell Beast's Shriek");
    let mut t = TestGame::new(3);
    let a1 = t.battlefield(P1, "Grizzly Bears");
    let a2 = t.battlefield(P1, "Hill Giant");
    let b1 = t.battlefield(P2, "Grizzly Bears");
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(a2)]);
    let shriek = in_hand_with_mana(&mut t, P0, "Fell Beast's Shriek");
    let from = t.asked().len();
    t.cast(P0, shriek).go();
    // Not targeted: no target is chosen as it's cast.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })));
    t.resolve_all();
    // P1 chose among their own creatures, then P2 (APNAP order).
    let choices: Vec<(PlayerId, Vec<Entity>)> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some((*p, candidates.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].0, P1);
    assert_eq!(choices[1].0, P2);
    assert_eq!(choices[0].1.len(), 2);
    assert_eq!(choices[1].1, vec![Entity::Object(b1)]);
    assert!(t.obj_now(a2).tapped && goaded_by(&t, a2) == vec![P0]);
    assert!(!t.obj_now(a1).tapped && goaded_by(&t, a1).is_empty());
    assert!(t.obj_now(b1).tapped && goaded_by(&t, b1) == vec![P0]);
    assert!(!t.obj_now(mine).tapped);
}

#[test]
fn target_opponent_chooses_a_creature_to_destroy() {
    cr!("115.10a", "701.8a");
    supported("Imperial Edict");
    supported("Wei Assassins");
    // Imperial Edict: "Target opponent chooses a creature they control. Destroy that
    // creature."
    let mut t = TestGame::new(3);
    let a1 = t.battlefield(P1, "Grizzly Bears");
    let a2 = t.battlefield(P1, "Hill Giant");
    let b1 = t.battlefield(P2, "Grizzly Bears");
    t.answer_choose(P1, &[Entity::Object(a2)]);
    let edict = in_hand_with_mana(&mut t, P0, "Imperial Edict");
    t.cast(P0, edict).target(P1).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(a1) && t.on_battlefield(b1));
    // Wei Assassins: "When ~ enters, target opponent chooses a creature they control.
    // Destroy that creature."
    let mut t = TestGame::new(2);
    let a1 = t.battlefield(P1, "Grizzly Bears");
    let a2 = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[Entity::Object(a1)]);
    t.enter(P0, "Wei Assassins");
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(a2));
}

#[test]
fn spliced_cards_stay_in_your_hand() {
    cr!("702.47a", "702.47e");
    ruling!("Fell Beast's Shriek", "Any cards you splice onto a spell remain in your hand, regardless of what happens to the spell after you cast it.");
    for countered in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let bolt = bolt_with_shrieks(&mut t, 1);
        let spell = t.cast(P0, bolt).target(P1).go();
        assert!(t.in_hand(P0, "Fell Beast's Shriek"));
        if countered {
            let cancel = in_hand_with_mana(&mut t, P1, "Cancel");
            t.cast(P1, cancel).target(spell).go();
        }
        t.resolve_all();
        assert!(t.in_hand(P0, "Fell Beast's Shriek"));
        assert_eq!(t.life(P1), if countered { 20 } else { 17 });
        assert_eq!(t.obj_now(bears).tapped, !countered);
    }
}

#[test]
fn a_spell_with_all_targets_illegal_does_nothing_spliced_text_included() {
    cr!("608.2b", "702.47d");
    ruling!("Fell Beast's Shriek", "If all of the spell's targets are illegal when the spell tries to resolve, it won't resolve and none of its effects will happen, including those from cards spliced onto it.");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let bolt = bolt_with_shrieks(&mut t, 1);
    t.cast(P0, bolt).target(bears).go();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(!t.obj_now(giant).tapped);
    assert!(goaded_by(&t, giant).is_empty());
}

#[test]
fn spliced_text_happens_after_the_spells_own_effects() {
    cr!("702.47a", "608.2c");
    ruling!("Fell Beast's Shriek", "The abilities spliced onto the spell happen last, after all of that spell's other effects.");
    // Unsummon returns the Bears before P1 chooses a creature: only the Giant can be
    // chosen.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let unsummon = in_hand_with_mana(&mut t, P0, "Unsummon");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    t.hand(P0, "Fell Beast's Shriek");
    splice(&mut t, P0, true);
    t.cast(P0, unsummon).target(bears).go();
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert_eq!(
        choice_candidates(&t, from, "Choose"),
        vec![vec![Entity::Object(giant)]]
    );
    assert!(t.obj_now(giant).tapped);
}

#[test]
fn several_cards_are_revealed_together_and_each_is_spliced_once() {
    cr!("702.47b", "601.2b");
    ruling!("Fell Beast's Shriek", "You reveal all cards you intend to splice at the same time. Each individual card can be spliced only once onto any spell, although multiple cards with the same name may be spliced onto one spell.");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let bolt = bolt_with_shrieks(&mut t, 2);
    let from = t.asked().len();
    t.cast(P0, bolt).target(P1).go();
    let asked = t.asked();
    let offers: Vec<usize> = asked[from..]
        .iter()
        .enumerate()
        .filter(|(_, (_, d))| {
            matches!(d, Decision::OptionalCost { name, repeatable: false, .. } if name == "splice")
        })
        .map(|(i, _)| i)
        .collect();
    let target = asked[from..]
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .expect("target chosen");
    // Two different Shrieks offered, once each, both before the target is chosen.
    assert_eq!(offers.len(), 2);
    assert!(offers.iter().all(|i| *i < target));
    // Both spliced: P1 chooses a creature twice.
    t.answer_choose(P1, &[Entity::Object(a)]);
    t.answer_choose(P1, &[Entity::Object(b)]);
    t.resolve_all();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert_eq!(t.life(P1), 17);
}
