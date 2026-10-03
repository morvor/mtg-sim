//! CR 500.7–500.11: extra turns, additional phases and steps, and skipping.

use crate::r500_common::*;
use crate::r703_common::{oracle_card, run_effect};
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Phase, Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

fn upkeep_sprite() -> mtg_engine::card::CardDef {
    oracle_card(
        "Upkeep Sprite",
        "Creature — Faerie",
        "{0}",
        Some((1, 1)),
        "At the beginning of your upkeep, you gain 1 life.",
    )
}

// ---------------------------------------------------------------------------
// CR 500.7: extra turns
// ---------------------------------------------------------------------------

#[test]
fn extra_turns_are_added_one_at_a_time_directly_after_this_turn() {
    cr!("500.7");
    ruling!("Time Stretch", "the most recently created extra turn is taken first");
    let mut t = TestGame::new(2);
    let stretch = t.hand(P0, "Time Stretch");
    t.lands(P0, "Island", 10);
    t.cast(P0, stretch).target(P0).go();
    t.resolve_all();
    assert_eq!(t.g.extra_turns, vec![P0, P0]);
    // Both extra turns come directly after this turn, before P1's turn.
    assert_eq!(next_turns(&mut t, 3), vec![P0, P0, P1]);
}

#[test]
fn the_most_recently_created_extra_turn_is_taken_first() {
    cr!("500.7");
    let mut t = TestGame::new(3);
    // P0's extra turn is created first, then P1's: P1's turn comes first.
    for p in [P0, P1] {
        run_effect(
            &mut t,
            p,
            None,
            Effect::ExtraTurn {
                who: PlayerRef::You,
            },
            &[],
        );
    }
    assert_eq!(next_turns(&mut t, 4), vec![P1, P0, P1, P2]);
}

#[test]
fn extra_turns_for_several_players_are_added_in_apnap_order() {
    cr!("500.7", "101.4");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    // During P1's turn, each player is given an extra turn: they're added P1, P2, P0
    // (APNAP), so P0's (the most recent) is taken first.
    run_effect(
        &mut t,
        P1,
        None,
        Effect::ExtraTurn {
            who: PlayerRef::EachPlayer,
        },
        &[],
    );
    assert_eq!(next_turns(&mut t, 4), vec![P0, P2, P1, P2]);
}

// ---------------------------------------------------------------------------
// CR 500.8: additional phases
// ---------------------------------------------------------------------------

#[test]
fn additional_phases_follow_the_phase_the_most_recent_first() {
    cr!("500.8", "500.1");
    ruling!(
        "Sphinx of the Second Sun",
        "the most recently created phase happens first"
    );
    let mut t = TestGame::new(2);
    t.g.turn.number = 3; // not the starting player's first turn (CR 103.8a)
    t.battlefield(P0, "Sphinx of the Second Sun");
    let assault = t.hand(P0, "Relentless Assault");
    t.lands(P0, "Mountain", 4);
    t.set_step(P0, Step::EndOfCombat);
    // The Sphinx's trigger resolves first in the postcombat main phase: a beginning phase
    // is added after this phase. Then Relentless Assault adds a combat phase and a main
    // phase after this main phase: those come first.
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    t.cast(P0, assault).go();
    t.resolve_all();
    let steps = next_turn_steps(&mut t, P0);
    let after: Vec<Step> = steps
        .iter()
        .copied()
        .skip_while(|s| *s != Step::PostcombatMain)
        .collect();
    assert_eq!(
        after,
        vec![
            Step::PostcombatMain,
            Step::BeginningOfCombat,
            Step::DeclareAttackers,
            Step::EndOfCombat,
            Step::PostcombatMain,
            // The Sphinx triggers again at the beginning of this postcombat main phase
            // too (CR 505.1a), adding another beginning phase after it.
            Step::Untap,
            Step::Upkeep,
            Step::Draw,
            Step::Untap,
            Step::Upkeep,
            Step::Draw,
            Step::End,
            Step::Cleanup,
        ]
    );
}

#[test]
fn an_additional_combat_phase_only_after_a_main_phase() {
    cr!("500.8");
    ruling!(
        "Relentless Assault",
        "only if it resolves during a main phase"
    );
    // "After this main phase, ..." outside a main phase adds nothing.
    let mut t = TestGame::new(2);
    let assault = oracle_card(
        "Instant Assault",
        "Instant",
        "{0}",
        None,
        "After this main phase, there is an additional combat phase followed by an additional main phase.",
    );
    let c = t.custom(P0, assault, Zone::Hand(P0));
    t.set_step(P0, Step::End);
    t.cast(P0, c).go();
    t.resolve_all();
    assert_eq!(t.g.turn.schedule, vec![Step::Cleanup]);
}

// ---------------------------------------------------------------------------
// CR 500.9: additional steps
// ---------------------------------------------------------------------------

#[test]
fn an_additional_upkeep_step_comes_directly_after_this_step() {
    cr!("500.9");
    ruling!(
        "Paradox Haze",
        "will trigger at the beginning of each additional upkeep as well"
    );
    let mut t = TestGame::new(2);
    let haze = t.battlefield(P0, "Paradox Haze");
    t.g.objects[haze.0 as usize].attached_to = Some(Entity::Player(P0));
    t.custom(P0, upkeep_sprite(), Zone::Battlefield);
    t.set_step(P1, Step::End);
    let steps = next_turn_steps(&mut t, P0);
    assert_eq!(
        &steps[..4],
        &[Step::Untap, Step::Upkeep, Step::Upkeep, Step::Draw]
    );
    // The upkeep trigger triggered in both upkeep steps; the Haze only in the first.
    assert_eq!(t.life(P0), 22);
}

// ---------------------------------------------------------------------------
// CR 500.10: a step added after a phase comes with its own phase
// ---------------------------------------------------------------------------

#[test]
fn upkeep_steps_added_after_combat_get_beginning_phases_of_their_own() {
    cr!("500.10", "500.8");
    ruling!(
        "Obeka, Splitter of Seconds",
        "the untap and draw steps will be skipped"
    );
    ruling!(
        "Obeka, Splitter of Seconds",
        "will trigger at the beginning of each additional upkeep step as well"
    );
    let mut t = TestGame::new(2);
    let obeka = t.battlefield(P0, "Obeka, Splitter of Seconds");
    t.custom(P0, upkeep_sprite(), Zone::Battlefield);
    let land = t.battlefield(P0, "Island");
    t.g.objects[land.0 as usize].tapped = true;
    t.set_step(P0, Step::BeginningOfCombat);
    let hand = t.hand_size(P0);
    t.attack(&[(obeka, Entity::Player(P1))], &[]);
    // Obeka dealt 2 combat damage: two beginning phases, each with only an upkeep step,
    // follow the combat phase.
    let steps = next_turn_steps(&mut t, P0);
    let after: Vec<Step> = steps
        .iter()
        .copied()
        .skip_while(|s| *s != Step::EndOfCombat)
        .collect();
    assert_eq!(
        after,
        vec![
            Step::EndOfCombat,
            Step::Upkeep,
            Step::Upkeep,
            Step::PostcombatMain,
            Step::End,
            Step::Cleanup,
        ]
    );
    // No untap or draw steps happened in them: the land stayed tapped and no card was
    // drawn; the upkeep trigger triggered in each upkeep.
    assert!(t.obj_now(land).tapped);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn a_draw_step_added_after_a_main_phase() {
    cr!("500.10");
    let mut t = TestGame::new(2);
    t.g.turn.number = 3; // not the starting player's first turn (CR 103.8a)
    let uud = t.hand(P0, "Untap, Upkeep, Draw");
    let lands = t.lands(P0, "Island", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, uud).modes(&[2]).go();
    t.resolve_all();
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    let hand = t.hand_size(P0);
    // After this main phase: a beginning phase with only a draw step, then combat.
    let steps = next_turn_steps(&mut t, P0);
    assert_eq!(&steps[..2], &[Step::Draw, Step::BeginningOfCombat]);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
}

#[test]
fn you_get_adds_nothing_to_another_players_turn() {
    cr!("500.10a");
    let extra = || {
        oracle_card(
            "Extra Upkeep",
            "Instant",
            "{0}",
            None,
            "You get an additional upkeep step after this step.",
        )
    };
    let mut t = TestGame::new(2);
    let mine = t.custom(P0, extra(), Zone::Hand(P0));
    let theirs = t.custom(P1, extra(), Zone::Hand(P1));
    t.set_step(P0, Step::Upkeep);
    // P1 "gets" an upkeep step during P0's turn: nothing is added.
    t.cast(P1, theirs).go();
    t.resolve_all();
    assert_eq!(t.g.turn.schedule.first(), Some(&Step::Draw));
    // P0 gets one during their own turn.
    t.cast(P0, mine).go();
    t.resolve_all();
    assert_eq!(t.g.turn.schedule.first(), Some(&Step::Upkeep));
}

// ---------------------------------------------------------------------------
// CR 500.11: skipping
// ---------------------------------------------------------------------------

#[test]
fn a_skipped_step_is_passed_as_though_it_didnt_exist() {
    cr!("500.11");
    ruling!("Eon Hub", "The turn proceeds from untap step to draw step");
    ruling!("Eon Hub", "Upkeep-triggered abilities don't trigger");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eon Hub");
    t.custom(P0, upkeep_sprite(), Zone::Battlefield);
    t.set_step(P1, Step::End);
    let steps = next_turn_steps(&mut t, P0);
    assert_eq!(&steps[..3], &[Step::Untap, Step::Draw, Step::PrecombatMain]);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_skipped_turn_is_passed_as_though_it_didnt_exist() {
    cr!("500.11");
    let mut t = TestGame::new(2);
    let vapors = t.battlefield(P1, "Lethal Vapors");
    // "Any player may activate this ability": P0 destroys it and skips their next turn.
    t.activate(P0, vapors, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lethal Vapors"));
    assert_eq!(next_turns(&mut t, 3), vec![P1, P1, P0]);
}

#[test]
fn skipping_the_next_draw_step() {
    cr!("500.11");
    let mut t = TestGame::new(2);
    let fatigue = t.hand(P0, "Fatigue");
    t.lands(P0, "Island", 2);
    t.cast(P0, fatigue).target(P1).go();
    t.resolve_all();
    let hand = t.hand_size(P1);
    let steps = next_turn_steps(&mut t, P1);
    assert!(!steps.contains(&Step::Draw));
    assert_eq!(t.hand_size(P1), hand);
    // Only the next one: the following draw step happens.
    let steps = next_turn_steps(&mut t, P1);
    assert!(steps.contains(&Step::Draw));
}

#[test]
fn skipped_upkeep_holds_untap_step_triggers_until_the_draw_step() {
    cr!("500.11", "502.4");
    ruling!(
        "Eon Hub",
        "Any triggered abilities that triggered during the untap step will go onto the stack at the start of the draw step"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eon Hub");
    let untapper = oracle_card(
        "Untap Watcher",
        "Creature — Faerie",
        "{0}",
        Some((1, 1)),
        "Whenever this creature becomes untapped, you gain 1 life.",
    );
    let w = t.custom(P0, untapper, Zone::Battlefield);
    t.g.objects[w.0 as usize].tapped = true;
    t.set_step(P1, Step::End);
    run_to(&mut t, "P0's draw step", |g| {
        g.turn.active == P0 && g.turn.step == Step::Draw && g.turn.stage == Stage::Priority
    });
    // It untapped in the untap step and triggered then; with no upkeep, the ability is
    // put on the stack the next time a player would receive priority: in the draw step.
    assert_eq!(t.g.pending_triggers.len(), 1);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}
