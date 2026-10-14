//! CR 500: turn structure — phases and steps, how they end, effects that last until a
//! step or phase, and "at the beginning of" triggers.

use crate::r114_common::{free_instant, probe_lines, queue_action, spy};
use crate::r105_util::add_pool;
use crate::r500_common::*;
use crate::r609_common::has_kw;
use crate::r703_common::{oracle_card, run_effect};
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Phase, Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_turn_has_five_phases_in_order_even_if_nothing_happens() {
    cr!("500.1");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let steps = next_turn_steps(&mut t, P0);
    // Nothing happens this turn (no spells, no attackers), yet every phase takes place.
    assert_eq!(
        phases(&steps),
        vec![
            Phase::Beginning,
            Phase::PrecombatMain,
            Phase::Combat,
            Phase::PostcombatMain,
            Phase::Ending,
        ]
    );
    // The beginning, combat, and ending phases are broken down into steps, in order.
    assert_eq!(&steps[..3], &[Step::Untap, Step::Upkeep, Step::Draw]);
    assert_eq!(steps[3], Step::PrecombatMain);
    assert_eq!(steps[4], Step::BeginningOfCombat);
    assert_eq!(&steps[steps.len() - 2..], &[Step::End, Step::Cleanup]);
    let mut sorted = steps.clone();
    sorted.sort();
    assert_eq!(sorted, steps);
}

#[test]
fn a_step_ends_only_when_all_players_pass_in_succession_with_an_empty_stack() {
    cr!("500.2");
    let mut t = TestGame::new(2);
    let bolt = t.custom(P0, free_instant("Quick Thought"), Zone::Hand(P0));
    t.set_step(P0, Step::Upkeep);
    let seen = record_priority(&mut t, P0);
    queue_action(
        &mut t,
        P0,
        Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        },
    );
    // P0 casts the instant; both players pass and it resolves, emptying the stack.
    run_to(&mut t, "the instant resolved", |g| {
        g.stack.is_empty() && g.turn.passes == 0 && g.history.spells_cast.len() == 1
    });
    t.g.run_until(100, |g| g.stack.is_empty() && g.turn.priority.is_some());
    // The stack becoming empty doesn't end the step: the active player gets priority
    // again in the upkeep.
    assert_eq!(t.g.turn.step, Step::Upkeep);
    assert_eq!(t.g.turn.priority, Some(P0));
    run_to(&mut t, "the draw step", |g| g.turn.step == Step::Draw);
    let lines = probe_lines(&seen);
    let upkeep_priorities = lines.iter().filter(|l| l.ends_with("Upkeep")).count();
    // P0 got priority to cast, after casting, after the spell resolved (and passed then).
    assert!(upkeep_priorities >= 3, "{lines:?}");
}

#[test]
fn steps_without_priority_end_when_their_actions_are_done() {
    cr!("500.3");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let p0 = record_priority(&mut t, P0);
    let p1 = record_priority(&mut t, P1);
    let steps = next_turn_steps(&mut t, P0);
    assert!(steps.contains(&Step::Untap) && steps.contains(&Step::Cleanup));
    // Nobody received priority during the untap step or the cleanup step.
    for l in probe_lines(&p0).iter().chain(probe_lines(&p1).iter()) {
        assert!(!l.ends_with(":Untap") && !l.ends_with(":Cleanup"), "{l}");
    }
}

#[test]
fn effects_until_a_step_expire_as_that_step_begins() {
    cr!("500.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erhnam Djinn");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[bears.into()]);
    t.resolve_all();
    assert!(has_kw(&t, bears, KeywordKind::Landwalk));
    // It lasts through the rest of this turn and the opponent's turn...
    t.advance_to(P1, Step::End);
    assert!(has_kw(&t, bears, KeywordKind::Landwalk));
    // ...and expires as P0's next upkeep begins, before its new trigger resolves.
    run_to(&mut t, "P0's next upkeep", |g| {
        g.turn.active == P0 && g.turn.step == Step::Upkeep && g.turn.stage == Stage::Priority
    });
    t.g.recompute();
    assert!(!has_kw(&t, bears, KeywordKind::Landwalk));
}

#[test]
fn unspent_mana_empties_as_each_step_ends() {
    cr!("500.5");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Upkeep);
    add_pool(&mut t, P0, &[ManaType::R, ManaType::R]);
    assert_eq!(t.player(P0).mana_pool.total(), 2);
    run_to(&mut t, "the draw step", |g| g.turn.step == Step::Draw);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
}

#[test]
fn until_end_of_combat_effects_last_through_the_end_of_combat_step() {
    cr!("500.5a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(2))],
            duration: Duration::EndOfCombat,
        },
        &[bears.into()],
    );
    assert_eq!(t.pt(bears), (4, 4));
    // Still applies in the end of combat step...
    run_to(&mut t, "end of combat", |g| {
        g.turn.step == Step::EndOfCombat && g.turn.stage == Stage::Priority
    });
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    // ...and expires at the end of the combat phase.
    run_to(&mut t, "second main phase", |g| g.turn.step == Step::PostcombatMain);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn until_end_of_turn_effects_end_in_the_cleanup_step() {
    cr!("500.5b", "514.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::End);
    let growth = t.hand(P0, "Giant Growth");
    t.lands(P0, "Forest", 1);
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // The end step ends; the effect still lasts until the cleanup step.
    run_to(&mut t, "cleanup", |g| g.turn.step == Step::Cleanup);
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
    run_to(&mut t, "next turn", |g| g.turn.active == P1);
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn at_the_beginning_of_triggers_wait_until_a_player_would_receive_priority() {
    cr!("500.6");
    let mut t = TestGame::new(2);
    let sprite = oracle_card(
        "End Sprite",
        "Creature — Faerie",
        "{0}",
        Some((1, 1)),
        "At the beginning of each end step, you gain 1 life.",
    );
    t.custom(P0, sprite, Zone::Battlefield);
    t.set_step(P1, Step::PostcombatMain);
    run_to(&mut t, "the end step began", |g| {
        g.turn.step == Step::End && g.turn.stage == Stage::Priority
    });
    // The ability triggered as the step began (it waits to be put on the stack)...
    assert_eq!(t.g.pending_triggers.len(), 1);
    assert_eq!(t.stack_len(), 0);
    // ...and is put on the stack before the active player receives priority.
    let seen = spy(&mut t, P1, |g, _, d| {
        matches!(d, Decision::Priority { .. }).then(|| format!("stack:{}", g.stack.len()))
    });
    t.g.advance();
    assert_eq!(probe_lines(&seen).first().map(String::as_str), Some("stack:1"));
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn no_game_events_happen_between_steps_phases_or_turns() {
    cr!("500.12");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erhnam Djinn");
    t.lands(P0, "Forest", 2);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    // Every event of P0's turn so far happened within a step: the turn began, then its
    // untap step began, and every later event follows a step beginning.
    let evs = &t.turn_events;
    assert!(matches!(evs.first(), Some(Event::TurnBegan { .. })));
    assert!(matches!(
        evs.get(1),
        Some(Event::StepBegan {
            step: Step::Untap,
            ..
        })
    ));
    // Between the end of one step (the last event before a StepBegan) and the next
    // step's beginning there's nothing: each step's events come after its StepBegan.
    let mut current: Option<Step> = None;
    for e in evs.iter().skip(1) {
        match e {
            Event::StepBegan { step, .. } => {
                assert!(current.is_none_or(|c| c < *step));
                current = Some(*step);
            }
            _ => assert!(current.is_some()),
        }
    }
}
