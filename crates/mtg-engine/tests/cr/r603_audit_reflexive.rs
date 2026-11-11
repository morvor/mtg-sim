//! CR 603.12 / 603.2d audit: reflexive triggered abilities. One triggers on the action its
//! own resolving ability took — once however many times a cost was paid (603.12a), not
//! at all when the player didn't take it; its modes and targets are chosen as it goes on
//! the stack after that resolution (603.3c–d), with what the objects last looked like;
//! and effects that make abilities trigger additional times don't apply to reflexive or
//! delayed triggered abilities (603.2d).

use crate::r603_audit_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Queues the answers for "you may pay [cost] any number of times" paying it `n` times:
/// "yes" to the "may", "yes" to each payment, then "no" (just "no" for none).
fn pay_times(t: &mut TestGame, p: PlayerId, n: usize) {
    if n > 0 {
        t.answer_yes(p, true);
    }
    for _ in 0..n {
        t.answer_yes(p, true);
    }
    t.answer_yes(p, false);
}

#[test]
fn paying_a_cost_several_times_triggers_the_reflexive_ability_once() {
    cr!("603.12", "603.12a");
    ruling!(
        "Intrepid Adversary",
        "After you pay the {1}{W} cost one or more times, a second ability triggers and players may respond to it. As that ability resolves, the Intrepid Adversary gets its valor counters."
    );
    supported("Intrepid Adversary");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 8);
    let adv = t.hand(P0, "Intrepid Adversary");
    t.cast(P0, adv).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Intrepid Adversary"), 1, "the enters ability");
    pay_times(&mut t, P0, 3);
    t.resolve();
    let adv = t.g.current(adv);
    // The reflexive ability is on the stack, once; the counters aren't there yet.
    assert_eq!(triggers_of(&t, "Intrepid Adversary"), 1);
    assert_eq!(t.counters(adv, "valor"), 0);
    t.resolve();
    assert_eq!(t.counters(adv, "valor"), 3);
    assert_eq!(t.pt(adv), (6, 4));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn not_paying_the_cost_triggers_nothing() {
    cr!("603.12");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 8);
    let adv = t.hand(P0, "Intrepid Adversary");
    t.cast(P0, adv).go();
    t.resolve();
    pay_times(&mut t, P0, 0);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(t.g.current(adv), "valor"), 0);
}

#[test]
fn a_reflexive_abilitys_modes_are_chosen_as_it_goes_on_the_stack() {
    cr!("603.12", "603.3c", "603.12a");
    ruling!(
        "Tranquil Frillback",
        "You'll choose the modes and targets, if any, for that second ability at that time."
    );
    supported("Tranquil Frillback");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Hill Giant");
    let frill = t.hand(P0, "Tranquil Frillback");
    t.cast(P0, frill).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Tranquil Frillback"), 1);
    let modes_asked = |t: &TestGame| {
        t.asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseModes { .. }))
            .count()
    };
    assert_eq!(modes_asked(&t), 0, "the enters ability has no modes");
    // Pay {G} twice; then, as the reflexive ability goes on the stack, choose two modes:
    // exile P1's graveyard and gain 4 life.
    pay_times(&mut t, P0, 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1, 2]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    t.settle();
    let max: Vec<u32> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(max, vec![2], "up to that many: the number of payments");
    assert_eq!(triggers_of(&t, "Tranquil Frillback"), 1);
    assert_eq!(t.graveyard_size(P1), 2);
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.life(P0), 24);
}

#[test]
fn a_reflexive_abilitys_targets_use_the_dead_creatures_last_power() {
    cr!("603.12", "608.2h", "603.10a");
    ruling!(
        "Sandbender Scavengers",
        "Use Sandbender Scavengers's power as it last existed on the battlefield to determine which creature cards may be targeted with its last ability."
    );
    supported("Sandbender Scavengers");
    let mut t = TestGame::new(2);
    let scav = t.battlefield(P0, "Sandbender Scavengers");
    t.g.add_counters(Entity::Object(scav), counters::PLUS1, 2, None);
    t.g.recompute();
    assert_eq!(t.pt(scav).0, 3);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.g.destroy(scav, None);
    t.settle();
    assert_eq!(triggers_of(&t, "Sandbender Scavengers"), 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    t.settle();
    assert!(t.in_exile("Sandbender Scavengers"));
    let candidates: Vec<Entity> = t
        .asked()
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .last()
        .expect("targets were chosen for the reflexive ability");
    assert!(candidates.contains(&Entity::Object(bears)), "mana value 2 ≤ 3");
    assert!(
        !candidates.contains(&Entity::Object(giant)),
        "mana value 4 > 3"
    );
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)));
}

#[test]
fn an_additional_trigger_effect_doesnt_double_reflexive_abilities() {
    cr!("603.2d", "603.12");
    supported("Teysa Karlov");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    let scav = t.battlefield(P0, "Sandbender Scavengers");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Wastes");
    // Power 2: the Bears (mana value 2) can come back.
    t.g.add_counters(Entity::Object(scav), counters::PLUS1, 1, None);
    t.g.recompute();
    t.g.destroy(scav, None);
    t.settle();
    // The dies ability triggers twice (Teysa).
    assert_eq!(triggers_of(&t, "Sandbender Scavengers"), 2);
    // The first one exiles it, and its reflexive ability triggers once; the second can't
    // exile it any more, so no second reflexive ability.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_of(&t, "Sandbender Scavengers"), 2, "one dies + one reflexive");
    t.resolve();
    assert!(t.on_battlefield(t.g.current(bears)));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn an_additional_trigger_effect_doesnt_double_delayed_abilities() {
    cr!("603.2d", "603.7");
    ruling!(
        "Cryptek",
        "If the targeted artifact creature is put into a graveyard this turn, that ability triggers and resolves like any other triggered ability."
    );
    supported("Cryptek");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Teysa Karlov");
    let cryptek = t.battlefield(P0, "Cryptek");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.lands(P0, "Swamp", 2);
    t.activate(P0, cryptek, 0, &[Entity::Object(thopter)])
        .expect("activate Cryptek");
    t.resolve_all();
    t.g.destroy(thopter, None);
    t.settle();
    // The delayed ability (its source is Cryptek) triggers once: Teysa doesn't apply to it.
    assert_eq!(triggers_of(&t, "Cryptek"), 1);
    t.resolve_all();
    let back = t.g.current(thopter);
    assert!(t.on_battlefield(back));
    assert!(t.obj_now(back).tapped, "returned tapped");
}

#[test]
fn a_reflexive_ability_of_an_enters_ability_isnt_caused_by_the_permanent_entering() {
    cr!("603.12", "603.2");
    supported("Strict Proctor");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Strict Proctor");
    t.lands(P0, "Plains", 6);
    let adv = t.hand(P0, "Intrepid Adversary");
    t.cast(P0, adv).go();
    t.resolve();
    // The enters ability is caused by the Adversary entering: the Proctor triggers.
    assert_eq!(triggers_of(&t, "Intrepid Adversary"), 1);
    assert_eq!(triggers_of(&t, "Strict Proctor"), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(triggers_of(&t, "Strict Proctor"), 0);
    assert_eq!(triggers_of(&t, "Intrepid Adversary"), 1, "P0 paid {{2}}");
    // P0 pays {1}{W} once. The "When you pay this cost one or more times" ability
    // triggers on that payment, not on the Adversary entering.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(triggers_of(&t, "Intrepid Adversary"), 1, "the reflexive ability");
    assert_eq!(triggers_of(&t, "Strict Proctor"), 0);
    t.resolve();
    assert_eq!(t.counters(t.g.current(adv), "valor"), 1);
    assert_eq!(t.stack_len(), 0);
}
