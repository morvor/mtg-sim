//! CR 512–514: the ending phase — end step and cleanup step.

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn ending_phase_is_end_step_then_cleanup_step() {
    cr!("512.1");
    let mut t = TestGame::new(2);
    t.g.run_until(10_000, |g| {
        g.turn.step == Step::Cleanup && g.turn.stage == mtg_engine::turn::Stage::End
    });
    let log = steps_this_turn(&t);
    let n = log.len();
    assert_eq!(&log[n - 3..], &[Step::PostcombatMain, Step::End, Step::Cleanup]);
    assert_eq!(Step::End.phase(), Step::Cleanup.phase());
    assert_eq!(Step::End.phase(), mtg_engine::turn::Phase::Ending);
}

#[test]
fn end_step_has_no_turn_based_actions_and_active_player_gets_priority() {
    cr!("513.1");
    let mut t = TestGame::new(2);
    go_to(&mut t, Step::PostcombatMain);
    let before = t.g.turn_events.len();
    let order = priority_order_in(&mut t, Step::End);
    assert_eq!(order, vec![P0, P1]);
    let evs: Vec<_> = t.g.turn_events[before..]
        .iter()
        .filter(|e| !matches!(e, mtg_engine::events::Event::StepBegan { .. }))
        .collect();
    assert!(evs.is_empty(), "{evs:?}");
}

#[test]
fn at_end_of_turn_errata_triggers_at_the_beginning_of_the_end_step() {
    cr!("513.1a");
    // Ball Lightning was printed with "At end of turn, sacrifice Ball Lightning"; its Oracle
    // text reads "At the beginning of the end step, sacrifice this creature."
    let c = card("Ball Lightning");
    assert!(c.unsupported_text().is_empty(), "{:?}", c.unsupported_text());
    let mut t = TestGame::new(2);
    let ball = t.battlefield(P0, "Ball Lightning");
    go_to(&mut t, Step::End);
    assert!(t.on_battlefield(ball));
    t.resolve_all();
    assert!(!t.on_battlefield(ball));
    assert!(t.in_graveyard(P0, "Ball Lightning"));
}

#[test]
fn end_step_doesnt_back_up_for_new_triggers() {
    cr!("513.2");
    let mut t = TestGame::new(2);
    go_to(&mut t, Step::End);
    // A permanent with "at the beginning of the end step" enters during the end step.
    let ball = t.enter(P0, "Ball Lightning");
    // A delayed trigger "at the beginning of the next end step" is created during it.
    apply(
        &mut t,
        P0,
        Effect::AtNext {
            step: TriggerStep::End,
            effect: Box::new(gain(5)),
        },
        &[],
    );
    // But "until end of turn" effects created now still end this turn.
    let bears = t.battlefield(P0, "Grizzly Bears");
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
            duration: Duration::EndOfTurn,
        },
        &[bears],
    );
    t.resolve_all();
    assert!(t.on_battlefield(ball));
    assert_eq!(t.life(P0), 20);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(ball), "didn't trigger this turn");
    assert_eq!(t.pt(bears), (2, 2), "until end of turn effect ended");
    // Next turn's end step: both trigger.
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(ball));
    assert_eq!(t.life(P0), 25);
}

#[test]
fn active_player_discards_down_to_maximum_hand_size() {
    cr!("514.1");
    let mut t = TestGame::new(2);
    let keep: Vec<ObjectId> = (0..7).map(|_| t.hand(P0, "Grizzly Bears")).collect();
    let extra1 = t.hand(P0, "Hill Giant");
    let extra2 = t.hand(P0, "Craw Wurm");
    // The non-active player's hand isn't affected.
    for _ in 0..9 {
        t.hand(P1, "Grizzly Bears");
    }
    t.answer_choose(P0, &[Entity::Object(extra1), Entity::Object(extra2)]);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), 7);
    assert!(keep.iter().all(|c| t.zone(*c) == Zone::Hand(P0)));
    assert!(t.in_graveyard(P0, "Hill Giant") && t.in_graveyard(P0, "Craw Wurm"));
    // P1 discarded nothing during P0's cleanup.
    assert_eq!(t.hand_size(P1), 9);
    // The discard was a turn-based action: it asked P0, and nothing used the stack.
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { min: 2, max: 2, .. })));
}

#[test]
fn damage_removal_and_end_of_turn_effects_end_simultaneously() {
    cr!("514.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // +0/+2 until end of turn and 3 damage: if the effect ended before the damage was
    // removed, the bears would die.
    apply(
        &mut t,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(0), Value::c(2))],
            duration: Duration::EndOfTurn,
        },
        &[bears],
    );
    t.g.objects[bears.0 as usize].damage = 3;
    // A phased-out permanent's damage is removed too.
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.objects[giant.0 as usize].damage = 2;
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![giant]);
    t.settle();
    assert!(t.on_battlefield(bears));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.g.obj(giant).damage, 0);
    // "This turn" rule effects end too.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    apply(
        &mut t,
        P1,
        Effect::AddRestriction {
            restriction: Restriction::CantBlock(Filter::In(Box::new(Sel::Target(0)))),
            duration: Duration::ThisTurn,
        },
        &[bears],
    );
    assert!(!t.g.can_block_at_all(bears));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.g.can_block_at_all(bears));
}

#[test]
fn nobody_gets_priority_in_a_normal_cleanup_step() {
    cr!("514.3");
    let mut t = TestGame::new(2);
    go_to(&mut t, Step::End);
    t.script.lock().unwrap().asked.clear();
    t.g.run_until(10_000, |g| g.turn.step == Step::Cleanup);
    // Players pass in the end step; then the cleanup step gives no one priority.
    t.script.lock().unwrap().asked.clear();
    t.g.run_until(10_000, |g| g.turn.active == P1);
    let priority_in_cleanup = t
        .asked()
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Priority { .. }))
        .count();
    assert_eq!(priority_in_cleanup, 0);
    assert_eq!(
        t.g.turn.step,
        Step::Untap,
        "the next turn began without anyone getting priority"
    );
}

#[test]
fn triggers_during_cleanup_give_priority_and_another_cleanup_step() {
    cr!("514.3a");
    let mut t = TestGame::new(2);
    // "Whenever you discard a card, you gain 1 life."
    bf(
        &mut t,
        P0,
        custom_card(
            "Scrap Collector",
            "Enchantment",
            None,
            "Whenever you discard a card, you gain 1 life.",
        ),
    );
    for _ in 0..8 {
        t.hand(P0, "Grizzly Bears");
    }
    go_to(&mut t, Step::End);
    t.g.run_until(10_000, |g| g.turn.step == Step::Cleanup);
    t.g.run_until(10_000, |g| {
        g.turn.step == Step::Cleanup && g.turn.stage == mtg_engine::turn::Stage::Priority
    });
    // The discard trigger is waiting; the active player gets priority after it's stacked.
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(t.stack_len(), 1);
    // Create a delayed trigger for "the next cleanup step" while in this one.
    apply(
        &mut t,
        P0,
        Effect::AtNext {
            step: TriggerStep::Cleanup,
            effect: Box::new(gain(10)),
        },
        &[],
    );
    t.g.run_until(10_000, |g| {
        g.turn.step_log.iter().filter(|s| **s == Step::Cleanup).count() >= 2
            || g.turn.active == P1
    });
    assert_eq!(t.g.turn.active, P0);
    assert_eq!(t.g.turn.step, Step::Cleanup, "another cleanup step began");
    t.g.run_until(10_000, |g| g.turn.active == P1);
    assert_eq!(t.life(P0), 31, "discard trigger and the next cleanup's delayed trigger");
}
