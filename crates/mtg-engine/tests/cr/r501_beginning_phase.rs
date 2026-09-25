//! CR 501–504: the beginning phase — untap step, upkeep step, draw step.

use crate::r114_common::{free_instant, probe_lines};
use crate::r500_common::*;
use crate::r703_common::oracle_card;
use mtg_engine::card::CardDef;
use mtg_engine::decision::Decision;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Phase, Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

fn untap_watcher() -> CardDef {
    oracle_card(
        "Untap Watcher",
        "Creature — Faerie",
        "{0}",
        Some((1, 1)),
        "Whenever this creature becomes untapped, you gain 1 life.",
    )
}

fn upkeep_sprite() -> CardDef {
    oracle_card(
        "Upkeep Sprite",
        "Creature — Faerie",
        "{0}",
        Some((1, 1)),
        "At the beginning of your upkeep, you gain 1 life.",
    )
}

#[test]
fn the_beginning_phase_is_untap_upkeep_and_draw() {
    cr!("501.1");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let steps = next_turn_steps(&mut t, P0);
    let beginning: Vec<Step> = steps
        .iter()
        .copied()
        .take_while(|s| s.phase() == Phase::Beginning)
        .collect();
    assert_eq!(beginning, vec![Step::Untap, Step::Upkeep, Step::Draw]);
}

#[test]
fn phasing_happens_first_in_the_untap_step_then_untapping() {
    cr!("502.1", "502.3");
    let mut t = TestGame::new(2);
    // A phasing creature phases out; a tapped creature that phased out earlier phases in
    // and, as untapping comes after phasing, is untapped too.
    let imp = t.battlefield(P0, "Teferi's Imp");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[imp.0 as usize].tapped = true;
    t.g.objects[bears.0 as usize].tapped = true;
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![bears]);
    t.set_step(P1, Step::End);
    run_to(&mut t, "P0's upkeep", |g| {
        g.turn.active == P0 && g.turn.step == Step::Upkeep
    });
    let imp = t.obj_now(imp);
    assert!(imp.phased_out);
    // Phased out, it isn't untapped.
    assert!(imp.tapped);
    let b = t.obj_now(bears);
    assert!(!b.phased_out);
    assert!(!b.tapped);
    // It doesn't use the stack.
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn day_becomes_night_in_the_untap_step_if_no_spells_were_cast() {
    cr!("502.2");
    let mut t = TestGame::new(2);
    t.g.turn.number = 3;
    t.g.day = Some(true);
    t.set_step(P1, Step::End);
    // P1 cast no spells during their turn: as P0's untap step begins, it becomes night.
    run_to(&mut t, "P0's upkeep", |g| {
        g.turn.active == P0 && g.turn.step == Step::Upkeep
    });
    assert_eq!(t.g.day, Some(false));
    // Night becomes day if the previous turn's active player cast two or more spells.
    let a = t.custom(P0, free_instant("First Thought"), Zone::Hand(P0));
    let b = t.custom(P0, free_instant("Second Thought"), Zone::Hand(P0));
    t.cast(P0, a).go();
    t.resolve_all();
    t.cast(P0, b).go();
    t.resolve_all();
    run_to(&mut t, "P1's upkeep", |g| {
        g.turn.active == P1 && g.turn.step == Step::Upkeep
    });
    assert_eq!(t.g.day, Some(true));
    // Neither day nor night: nothing happens.
    t.g.day = None;
    run_to(&mut t, "P0's upkeep", |g| {
        g.turn.active == P0 && g.turn.step == Step::Upkeep
    });
    assert_eq!(t.g.day, None);
}

fn two_headed_giant() -> TestGame {
    let config = GameConfig {
        variant: Variant::TwoHeadedGiant,
        teams: Some(vec![0, 0, 1, 1]),
        ..Default::default()
    };
    let mut t = TestGame::with_config(4, config);
    t.g.turn.number = 3;
    t
}

#[test]
fn day_night_with_shared_team_turns_looks_at_the_whole_team() {
    cr!("502.2a");
    // It's night. On team P0/P1's turn, P0 and P1 each cast one spell: no single player
    // cast two, so it stays night.
    let mut t = two_headed_giant();
    t.g.day = Some(false);
    t.set_step(P0, Step::PrecombatMain);
    for p in [P0, P1] {
        let c = t.custom(p, free_instant("Team Thought"), Zone::Hand(p));
        t.cast(p, c).go();
        t.resolve_all();
    }
    run_to(&mut t, "the other team's upkeep", |g| {
        g.turn.active == P2 && g.turn.step == Step::Upkeep
    });
    assert_eq!(t.g.day, Some(false));

    // P1 (not the player representing the turn) casts two spells: it becomes day.
    let mut t = two_headed_giant();
    t.g.day = Some(false);
    t.set_step(P0, Step::PrecombatMain);
    for _ in 0..2 {
        let c = t.custom(P1, free_instant("Team Thought"), Zone::Hand(P1));
        t.cast(P1, c).go();
        t.resolve_all();
    }
    run_to(&mut t, "the other team's upkeep", |g| {
        g.turn.active == P2 && g.turn.step == Step::Upkeep
    });
    assert_eq!(t.g.day, Some(true));

    // It's day and P1 cast a spell during the team's turn: it stays day.
    let mut t = two_headed_giant();
    t.g.day = Some(true);
    t.set_step(P0, Step::PrecombatMain);
    let c = t.custom(P1, free_instant("Team Thought"), Zone::Hand(P1));
    t.cast(P1, c).go();
    t.resolve_all();
    run_to(&mut t, "the other team's upkeep", |g| {
        g.turn.active == P2 && g.turn.step == Step::Upkeep
    });
    assert_eq!(t.g.day, Some(true));
}

#[test]
fn nobody_gets_priority_in_the_untap_step_and_its_triggers_wait() {
    cr!("502.4", "503.1a");
    let mut t = TestGame::new(2);
    let w = t.custom(P0, untap_watcher(), Zone::Battlefield);
    t.g.objects[w.0 as usize].tapped = true;
    t.custom(P0, upkeep_sprite(), Zone::Battlefield);
    t.set_step(P1, Step::End);
    let seen = crate::r114_common::spy(&mut t, P0, |g, _, d| match d {
        Decision::Priority { .. } if g.turn.active == P0 => {
            Some(format!("{:?}:{}", g.turn.step, g.stack.len()))
        }
        _ => None,
    });
    run_to(&mut t, "P0's draw step", |g| {
        g.turn.active == P0 && g.turn.step == Step::Draw
    });
    let lines = probe_lines(&seen);
    // No priority in the untap step; the first time P0 gets priority is in the upkeep,
    // with both the untap trigger and the upkeep trigger already on the stack.
    assert!(lines.iter().all(|l| !l.starts_with("Untap")), "{lines:?}");
    assert_eq!(lines.first().map(String::as_str), Some("Upkeep:2"));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn the_upkeep_has_no_turn_based_actions_and_the_active_player_gets_priority() {
    cr!("503.1");
    let mut t = TestGame::new(2);
    t.g.turn.number = 3;
    t.set_step(P1, Step::End);
    let hand = t.hand_size(P0);
    run_to(&mut t, "P0's upkeep", |g| {
        g.turn.active == P0 && g.turn.step == Step::Upkeep && g.turn.stage == Stage::Priority
    });
    // Nothing happened as the step began: the active player simply gets priority.
    assert_eq!(t.g.turn.priority, Some(P0));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.stack_len(), 0);
    let evs: Vec<_> = t
        .turn_events
        .iter()
        .skip_while(|e| {
            !matches!(
                e,
                mtg_engine::events::Event::StepBegan {
                    step: Step::Upkeep,
                    ..
                }
            )
        })
        .collect();
    assert_eq!(evs.len(), 1);
}

#[test]
fn after_their_upkeep_step_means_after_the_first_upkeep_ends() {
    cr!("503.2");
    let mut t = TestGame::new(2);
    let haze = t.battlefield(P0, "Paradox Haze");
    t.g.objects[haze.0 as usize].attached_to = Some(Entity::Player(P0));
    let reset = t.hand(P1, "Reset");
    t.lands(P1, "Island", 2);
    t.set_step(P1, Step::End);
    run_to(&mut t, "P0's first upkeep", |g| {
        g.turn.active == P0 && g.turn.step == Step::Upkeep && g.turn.stage == Stage::Priority
    });
    // During the first upkeep it can't be cast.
    t.settle();
    assert!(t.cast(P1, reset).try_go().is_err());
    t.resolve_all();
    // The Haze gave P0 a second upkeep step: after the first one ended, it can be.
    run_to(&mut t, "P0's second upkeep", |g| {
        g.turn.step == Step::Upkeep && g.turn.upkeeps == 2 && g.turn.stage == Stage::Priority
    });
    assert!(t.cast(P1, reset).try_go().is_ok());
    t.resolve_all();
    // Its lands were untapped by it.
    assert!(t.in_graveyard(P1, "Reset"));
}

#[test]
fn after_drawing_the_active_player_gets_priority_in_the_draw_step() {
    cr!("504.2");
    let mut t = TestGame::new(2);
    t.g.turn.number = 3;
    t.set_step(P0, Step::Upkeep);
    let hand = t.hand_size(P0);
    let seen = crate::r114_common::spy(&mut t, P0, |g, p, d| match d {
        Decision::Priority { .. } if g.turn.step == Step::Draw => {
            Some(format!("hand:{}", g.player(p).hand.len()))
        }
        _ => None,
    });
    run_to(&mut t, "the main phase", |g| g.turn.step == Step::PrecombatMain);
    assert_eq!(
        probe_lines(&seen).first().cloned(),
        Some(format!("hand:{}", hand + 1))
    );
}
