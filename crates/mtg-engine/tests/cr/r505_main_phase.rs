//! CR 505: the main phase — precombat and postcombat main phases, counting main phases,
//! the turn-based actions of the precombat main phase, and sorcery-speed actions.

use crate::r114_common::{free_sorcery, probe_lines, spy};
use crate::r500_common::*;
use crate::r703_common::{
    add_scheme_deck, archenemy_game, event_index, oracle_card, run_effect, step_began,
    to_step_start,
};
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::variants::{self, ROLLED_TO_VISIT, SET_IN_MOTION};
use mtg_engine::*;

fn main_trigger(name: &str, when: &str) -> mtg_engine::card::CardDef {
    oracle_card(
        name,
        "Creature — Faerie",
        "{0}",
        Some((1, 1)),
        &format!("At the beginning of your {when}, you gain 1 life."),
    )
}

/// Casts Relentless Assault in P0's precombat main phase (an additional combat phase and
/// main phase follow it) and plays out the turn; returns its steps.
fn turn_with_relentless_assault(t: &mut TestGame) -> Vec<Step> {
    let assault = t.hand(P0, "Relentless Assault");
    t.lands(P0, "Mountain", 4);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, assault).go();
    t.resolve_all();
    next_turn_steps(t, P0)
}

#[test]
fn two_main_phases_separated_by_combat() {
    cr!("505.1");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let steps = next_turn_steps(&mut t, P0);
    let mains: Vec<usize> = (0..steps.len()).filter(|i| steps[*i].is_main()).collect();
    assert_eq!(mains.len(), 2);
    assert_eq!(steps[mains[0]], Step::PrecombatMain);
    assert_eq!(steps[mains[1]], Step::PostcombatMain);
    assert!(steps[mains[0] + 1..mains[1]].iter().all(|s| s.is_combat()));
    assert!(mains[1] > mains[0] + 1);
}

#[test]
fn only_the_first_main_phase_is_precombat() {
    cr!("505.1a");
    ruling!(
        "Sphinx of the Second Sun",
        "each main phase other than the first one is a postcombat main phase"
    );
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        main_trigger("Morning Sprite", "precombat main phase"),
        Zone::Battlefield,
    );
    t.custom(
        P0,
        main_trigger("Evening Sprite", "postcombat main phase"),
        Zone::Battlefield,
    );
    let steps = turn_with_relentless_assault(&mut t);
    let mains: Vec<Step> = steps.iter().copied().filter(|s| s.is_main()).collect();
    // The main phase created by Relentless Assault is a postcombat main phase too.
    assert_eq!(mains, vec![Step::PostcombatMain, Step::PostcombatMain]);
    // The postcombat trigger triggered in both; (the precombat one already happened
    // before the spell was cast, and not again).
    assert_eq!(t.life(P0), 22);
}

#[test]
fn the_second_main_phase_after_a_skipped_combat_is_postcombat() {
    cr!("505.1a");
    let mut t = TestGame::new(2);
    t.custom(
        P0,
        main_trigger("Evening Sprite", "postcombat main phase"),
        Zone::Battlefield,
    );
    t.set_step(P0, Step::Upkeep);
    run_effect(
        &mut t,
        P0,
        None,
        Effect::Skip {
            who: PlayerRef::You,
            step: StepKind::Combat,
        },
        &[],
    );
    let steps = next_turn_steps(&mut t, P0);
    assert!(!steps.iter().any(|s| s.is_combat()));
    let mains: Vec<Step> = steps.iter().copied().filter(|s| s.is_main()).collect();
    assert_eq!(mains, vec![Step::PrecombatMain, Step::PostcombatMain]);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn second_main_phase_counts_main_phases_this_turn() {
    cr!("505.1b");
    ruling!(
        "Savior of the Small",
        "They won't trigger during your third, fourth, or other additional main phases"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ninja Pizza");
    let steps = turn_with_relentless_assault(&mut t);
    assert_eq!(steps.iter().filter(|s| s.is_main()).count(), 2);
    // Three main phases this turn: "At the beginning of your second main phase, create a
    // Food token" triggered only in the second one.
    let food = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.chars.has_subtype("Food"))
        .count();
    assert_eq!(food, 1);
}

#[test]
fn a_main_phase_ends_when_all_players_pass_with_an_empty_stack() {
    cr!("505.2");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let seen = spy(&mut t, P1, |g, _, d| {
        matches!(d, Decision::Priority { .. }).then(|| format!("{:?}", g.turn.step))
    });
    // It has no steps: P0 passes, P1 passes, and the combat phase begins.
    t.g.advance();
    assert_eq!(t.g.turn.step, Step::PrecombatMain);
    t.g.advance();
    assert_eq!(probe_lines(&seen), vec!["PrecombatMain".to_string()]);
    assert_eq!(t.g.turn.stage, Stage::End);
    t.g.advance();
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
}

#[test]
fn the_archenemy_sets_a_scheme_in_motion_only_in_the_precombat_main_phase() {
    cr!("505.3");
    let mut t = archenemy_game();
    let deck = add_scheme_deck(
        &mut t,
        P0,
        vec![
            (*card("Roots of All Evil")).clone(),
            (*card("Look Skyward and Despair")).clone(),
            (*card("Roots of All Evil")).clone(),
        ],
    );
    t.set_step(P0, Step::Draw);
    to_step_start(&mut t, P0, Step::PrecombatMain);
    assert!(!t.obj(deck[0]).face_down);
    assert!(t.stack_len() == 0 && !t.g.pending_triggers.is_empty());
    t.resolve_all();
    // An additional main phase later this turn isn't a precombat main phase: no scheme.
    run_effect(
        &mut t,
        P0,
        None,
        Effect::AddTurnParts {
            parts: vec![TurnPart::MainPhase],
            after_phase: true,
            n: Value::c(1),
            who: None,
        },
        &[],
    );
    run_to(&mut t, "P0's end step", |g| g.turn.step == Step::End);
    let mains = t
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::StepBegan { step, .. } if step.is_main()))
        .count();
    assert_eq!(mains, 3);
    let set = t
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::Custom { name, .. } if name.as_str() == SET_IN_MOTION))
        .count();
    assert_eq!(set, 1);
    assert_eq!(variants::scheme_deck(&t.g, P0).first(), Some(&deck[1]));
}

#[test]
fn sagas_get_lore_counters_in_the_precombat_main_phase_only() {
    cr!("505.4");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "History of Benalia");
    t.g.add_counters(Entity::Object(saga), counters::LORE, 1, None);
    t.g.turn.number = 3;
    t.set_step(P0, Step::Draw);
    to_step_start(&mut t, P0, Step::PrecombatMain);
    // The lore counter was put on before anyone received priority (the chapter ability
    // waits to be put on the stack).
    assert_eq!(t.counters(saga, counters::LORE), 2);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    // Relentless Assault: an additional (postcombat) main phase adds no lore counter.
    let assault = t.hand(P0, "Relentless Assault");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, assault).go();
    t.resolve_all();
    run_to(&mut t, "P1's turn", |g| g.turn.active == P1);
    assert_eq!(t.counters(saga, counters::LORE), 2);
}

#[test]
fn attractions_are_visited_after_lore_counters_then_priority() {
    cr!("505.5", "505.6");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "History of Benalia");
    t.battlefield(P0, "Information Booth");
    t.g.dice.loaded.push_back(2);
    t.set_step(P0, Step::Draw);
    let seen = spy(&mut t, P0, |g, _, d| match d {
        Decision::Priority { .. } if g.turn.step == Step::PrecombatMain => {
            Some(format!("stack:{}", g.stack.len()))
        }
        _ => None,
    });
    to_step_start(&mut t, P0, Step::PrecombatMain);
    let began = event_index(&t, step_began(Step::PrecombatMain)).unwrap();
    let lore = event_index(&t, |e| {
        matches!(e, Event::CountersAdded { target: Entity::Object(o), .. } if *o == saga)
    })
    .unwrap();
    let visited = event_index(
        &t,
        |e| matches!(e, Event::Custom { name, .. } if name.as_str() == ROLLED_TO_VISIT),
    )
    .unwrap();
    assert!(began < lore && lore < visited);
    // Fourth, the active player gets priority (after the triggered abilities go on the
    // stack).
    t.g.advance();
    assert_eq!(probe_lines(&seen).first().map(String::as_str), Some("stack:2"));
}

#[test]
fn sorcery_speed_spells_only_in_the_active_players_main_phase() {
    cr!("505.6a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let theirs = t.custom(P1, free_sorcery("Their Sorcery"), Zone::Hand(P1));
    // Not in the upkeep, not in combat.
    t.set_step(P0, Step::Upkeep);
    assert!(t.cast(P0, bears).try_go().is_err());
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.cast(P0, bears).try_go().is_err());
    // The nonactive player can't cast a sorcery during the active player's main phase.
    t.set_step(P0, Step::PostcombatMain);
    assert!(t.cast(P1, theirs).try_go().is_err());
    // The active player may, in either main phase.
    assert!(t.cast(P0, bears).try_go().is_ok());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn playing_a_land_is_a_main_phase_action_that_doesnt_use_the_stack() {
    cr!("505.6b");
    let mut t = TestGame::new(2);
    let f1 = t.hand(P0, "Forest");
    let f2 = t.hand(P0, "Forest");
    let theirs = t.hand(P1, "Forest");
    t.set_step(P0, Step::Upkeep);
    assert!(t.play_land(P0, f1).is_err());
    t.set_step(P0, Step::PrecombatMain);
    // Not while the stack isn't empty.
    let spell = t.custom(
        P0,
        crate::r114_common::free_instant("Quick Thought"),
        Zone::Hand(P0),
    );
    t.cast(P0, spell).go();
    assert!(t.play_land(P0, f1).is_err());
    t.resolve_all();
    // Not by the nonactive player.
    assert!(t.play_land(P1, theirs).is_err());
    t.play_land(P0, f1).unwrap();
    // It happened immediately: nothing went on the stack to respond to.
    assert!(t.on_battlefield(f1));
    assert_eq!(t.stack_len(), 0);
    // Only one per turn.
    assert!(t.play_land(P0, f2).is_err());
    t.set_step(P0, Step::PostcombatMain);
    assert!(t.play_land(P0, f2).is_err());
}
