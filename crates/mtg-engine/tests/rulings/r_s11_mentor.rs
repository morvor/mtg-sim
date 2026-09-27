//! Rulings batch S11 — mentor (CR 702.134): "Whenever this creature attacks, put a +1/+1
//! counter on target attacking creature with lesser power." The target's power is
//! compared as the ability is put on the stack and again as it resolves (CR 608.2b), using
//! the mentoring creature's last known information if it has left the battlefield.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s11_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

/// The candidates offered for the targets of triggered abilities of `src` since decision
/// `from`.
fn trigger_target_candidates(t: &TestGame, src: ObjectId, from: usize) -> Vec<Vec<Entity>> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                source, candidates, ..
            } if matches!(
                t.g.obj(*source).stack.as_deref().map(|s| &s.kind),
                Some(mtg_engine::object::StackKind::Triggered { source: s, .. }) if *s == src
            ) =>
            {
                Some(candidates.clone())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_second_mentor_trigger_does_nothing_once_the_target_isnt_smaller() {
    cr!("702.134a", "608.2b");
    ruling!(
        "Legion Warboss",
        "If the target creature's power is no longer less than the attacking creature's power as the ability resolves, mentor doesn't add a +1/+1 counter. For example, if two 3/3 creatures with mentor attack and both mentor triggers target the same 2/2 creature, the first to resolve puts a +1/+1 counter on it and the second does nothing."
    );
    supported("Legion Warboss");
    // Two Legion Warbosses (2/2 mentor) attack with Llanowar Elves (1/1), both mentor
    // triggers targeting the Elves.
    let mut t = TestGame::new(2);
    let w1 = t.battlefield(P0, "Legion Warboss");
    let w2 = t.battlefield(P0, "Legion Warboss");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_targets(P0, &[Entity::Object(elves)]);
    attack_with(
        &mut t,
        &[
            (w1, Entity::Player(P1)),
            (w2, Entity::Player(P1)),
            (elves, Entity::Player(P1)),
        ],
    );
    assert_eq!(triggered_from(&t, w1), 1);
    assert_eq!(triggered_from(&t, w2), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, elves), 1);
    assert_eq!(t.pt(elves), (2, 2));
}

#[test]
fn mentor_compares_power_when_put_on_the_stack_and_when_it_resolves() {
    cr!("702.134a", "603.3d", "608.2b", "507.1");
    ruling!(
        "Legion Warboss",
        "Mentor compares the power of the creature with mentor with that of the target creature at two different times: once as the triggered ability is put onto the stack, and once as the triggered ability resolves. If you wish to raise a creature's power so its mentor ability can target a bigger creature, the last chance you have to do so is during the beginning of combat step."
    );
    ruling!(
        "Truefire Captain",
        "Mentor compares the power of the creature with mentor with that of the target creature at two different times: once as the triggered ability is put onto the stack, and once as the triggered ability resolves. If you wish to raise a creature’s power so its mentor ability can target a bigger creature, the last chance you have to do so is during the beginning of combat step."
    );
    supported("Truefire Captain");
    supported("Tributary Instructor");
    // Truefire Captain (4/3 mentor) attacks with Tributary Instructor (4/4, mentor too):
    // neither can target the other.
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Truefire Captain");
    let instructor = t.battlefield(P0, "Tributary Instructor");
    let from = t.asked().len();
    attack_with(
        &mut t,
        &[
            (captain, Entity::Player(P1)),
            (instructor, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    assert!(trigger_target_candidates(&t, captain, from)
        .iter()
        .all(|c| c.is_empty()));
    assert_eq!(plus1(&t, instructor), 0);
    // Giant Growth on the Captain in the beginning of combat step: it's 7/6 when mentor
    // triggers, so it can target the Instructor.
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Truefire Captain");
    let instructor = t.battlefield(P0, "Tributary Instructor");
    t.set_step(P0, Step::BeginningOfCombat);
    let gg = in_hand_with_mana(&mut t, P0, "Giant Growth");
    t.cast(P0, gg).target(captain).go();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(instructor)]);
    attack_with(
        &mut t,
        &[
            (captain, Entity::Player(P1)),
            (instructor, Entity::Player(P1)),
        ],
    );
    t.resolve_all();
    assert_eq!(plus1(&t, instructor), 1);
    // The target grows in response: as the ability resolves its power isn't less, so no
    // counter.
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Truefire Captain");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(
        &mut t,
        &[(captain, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    assert_eq!(triggered_from(&t, captain), 1);
    let gg = in_hand_with_mana(&mut t, P0, "Giant Growth");
    t.cast(P0, gg).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(plus1(&t, bears), 0);
}

#[test]
fn mentor_uses_the_last_known_power_of_a_mentor_that_left_the_battlefield() {
    cr!("702.134a", "608.2b", "608.2h", "113.7a");
    ruling!(
        "Nyxborn Unicorn",
        "If the creature with mentor leaves the battlefield with the mentor ability on the stack, use its power as that creature last existed on the battlefield to determine whether the target creature has lesser power."
    );
    supported("Nyxborn Unicorn");
    // Nyxborn Unicorn (2/2 mentor), made 5/5 by Giant Growth before combat, attacks with
    // Hill Giant (3/3); its mentor ability targets the Giant, and the Unicorn is destroyed
    // before it resolves. As it last existed, its power was 5.
    let mut t = TestGame::new(2);
    let unicorn = t.battlefield(P0, "Nyxborn Unicorn");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    let gg = in_hand_with_mana(&mut t, P0, "Giant Growth");
    t.cast(P0, gg).target(unicorn).go();
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Object(giant)]);
    attack_with(
        &mut t,
        &[(unicorn, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    assert_eq!(triggered_from(&t, unicorn), 1);
    destroy(&mut t, unicorn);
    assert!(t.in_graveyard(P0, "Nyxborn Unicorn"));
    t.resolve_all();
    assert_eq!(plus1(&t, giant), 1);
    assert_eq!(t.pt(giant), (4, 4));
}
