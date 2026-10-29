//! CR 702.43 Modular.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_038_051::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn p1p1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

#[test]
fn modular_enters_with_n_counters_and_moves_them_when_it_dies() {
    cr!("702.43", "702.43a");
    assert_supported("Arcbound Worker");
    assert_supported("Arcbound Bruiser");
    let mut t = TestGame::new(2);
    // Arcbound Bruiser: 0/0, modular 3. Cast it.
    t.lands(P0, "Wastes", 5);
    let bruiser = t.hand(P0, "Arcbound Bruiser");
    t.cast(P0, bruiser).go();
    t.resolve_all();
    let bruiser = t.named_on_battlefield("Arcbound Bruiser")[0];
    assert_eq!(p1p1(&t, bruiser), 3);
    assert_eq!(t.pt(bruiser), (3, 3));
    // It gets a fourth counter; when it dies, all four move to target artifact creature.
    t.g.add_counters(Entity::Object(bruiser), counters::PLUS1, 1, None);
    let worker = t.enter(P0, "Arcbound Worker");
    assert_eq!(p1p1(&t, worker), 1);
    t.answer_targets(P0, &[Entity::Object(worker)]);
    t.g.destroy(bruiser, None);
    t.settle();
    assert_eq!(triggers_named(&t, "Modular 3").len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(p1p1(&t, worker), 5);
}

#[test]
fn modular_can_target_only_an_artifact_creature_and_is_optional() {
    cr!("702.43a");
    let mut t = TestGame::new(2);
    let worker = t.enter(P0, "Arcbound Worker");
    let other = t.enter(P0, "Arcbound Worker");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.enter(P1, "Arcbound Worker");
    t.answer_targets(P0, &[Entity::Object(other)]);
    t.g.destroy(worker, None);
    t.settle();
    let cands = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap();
    assert!(cands.contains(&Entity::Object(other)));
    assert!(cands.contains(&Entity::Object(theirs)));
    assert!(!cands.contains(&Entity::Object(bears)));
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(p1p1(&t, other), 1);
}

#[test]
fn sacrificing_a_modular_creature_to_arcbound_ravager() {
    cr!("702.43a");
    assert_supported("Arcbound Ravager");
    let mut t = TestGame::new(2);
    let ravager = t.enter(P0, "Arcbound Ravager");
    let worker = t.enter(P0, "Arcbound Worker");
    assert_eq!(p1p1(&t, ravager), 1);
    // "Sacrifice an artifact: Put a +1/+1 counter on this creature."
    t.answer_choose(P0, &[Entity::Object(worker)]);
    t.answer_targets(P0, &[Entity::Object(ravager)]);
    t.activate(P0, ravager, 0, &[]).unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    // One counter from the Worker's modular, one from Ravager's ability.
    assert_eq!(p1p1(&t, ravager), 3);
}

#[test]
fn modular_counts_the_counters_it_had_before_minus_one_counters_killed_it() {
    cr!("702.43a");
    ruling!(
        "Arcbound Worker",
        "modular will put a number of +1/+1 counters on the target artifact creature equal to the number of +1/+1 counters on this creature before it left the battlefield"
    );
    let mut t = TestGame::new(2);
    let worker = t.enter(P0, "Arcbound Worker");
    let stinger = t.enter(P0, "Arcbound Stinger");
    // Skinrender puts three -1/-1 counters on the Worker.
    t.answer_targets(P1, &[Entity::Object(worker)]);
    t.answer_targets(P0, &[Entity::Object(stinger)]);
    t.answer_yes(P0, true);
    t.enter(P1, "Skinrender");
    t.resolve_all();
    assert!(!t.on_battlefield(worker));
    assert_eq!(p1p1(&t, stinger), 2);
}

#[test]
fn modular_enters_with_counters_however_it_enters() {
    cr!("702.43a");
    let mut t = TestGame::new(2);
    // Put onto the battlefield from the graveyard rather than cast.
    let card = t.graveyard(P0, "Arcbound Bruiser");
    t.g.move_object(card, Zone::Battlefield, events::MoveCause::Effect, Some(P0));
    let b = t.named_on_battlefield("Arcbound Bruiser")[0];
    assert_eq!(p1p1(&t, b), 3);
}

#[test]
fn each_instance_of_modular_works_separately() {
    cr!("702.43b");
    let def = custom_card(
        "Double Modular Golem",
        "Artifact Creature — Golem",
        Some((0, 0)),
        "Modular 1\nModular 2",
    );
    let mut t = TestGame::new(2);
    let golem = t.g.create_card_object(std::sync::Arc::new(def), P0, Zone::Nowhere);
    t.g.move_object(golem, Zone::Battlefield, events::MoveCause::Effect, Some(P0));
    let golem = t.g.current(golem);
    // Each instance's replacement effect applies: 1 + 2 counters.
    assert_eq!(p1p1(&t, golem), 3);
    let worker = t.enter(P0, "Arcbound Worker");
    t.answer_targets(P0, &[Entity::Object(worker)]);
    t.answer_targets(P0, &[Entity::Object(worker)]);
    t.g.destroy(golem, None);
    t.settle();
    assert_eq!(triggers_named(&t, "Modular 1").len(), 1);
    assert_eq!(triggers_named(&t, "Modular 2").len(), 1);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    // Each trigger counts all three counters it had.
    assert_eq!(p1p1(&t, worker), 1 + 3 + 3);
}
