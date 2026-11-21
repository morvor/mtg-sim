//! CR 703: turn-based actions.

use crate::r114_common::{probe_lines, spy};
use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

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
fn turn_based_actions_happen_automatically_without_using_the_stack() {
    cr!("703.1", "703.2", "703.4d");
    let mut t = TestGame::new(2);
    t.g.turn.number = 3; // not the starting player's first turn (CR 103.8a)
    t.set_step(P0, Step::Upkeep);
    let hand = t.hand_size(P0);
    let asked_before = t.asked().len();
    // The draw happens as the draw step begins: no player chooses to perform it, and
    // nothing is put on the stack to be responded to.
    to_step_start(&mut t, P0, Step::Draw);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.stack_len(), 0);
    let draw = event_index(
        &t,
        |e| matches!(e, Event::Drew { player, .. } if *player == P0),
    )
    .expect("the active player drew");
    let began = event_index(&t, step_began(Step::Draw)).unwrap();
    assert!(began < draw);
    // Only the upkeep's priority passes were asked; nobody was asked about the draw.
    let asked: Vec<_> = t.asked()[asked_before..].to_vec();
    assert!(asked
        .iter()
        .all(|(_, d)| matches!(d, Decision::Priority { .. })));
    assert!(t.turn_events.iter().all(|e| !matches!(
        e,
        Event::AbilityTriggeredOnStack { .. } | Event::SpellCast { .. }
    )));
}

#[test]
fn abilities_that_watch_for_a_step_to_begin_are_triggered_abilities() {
    cr!("703.1a");
    let mut t = TestGame::new(2);
    t.custom(P0, upkeep_sprite(), Zone::Battlefield);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    // The ability uses the stack: it's waiting to resolve when P0 gets priority.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn turn_based_actions_come_before_triggers_state_based_actions_and_priority() {
    cr!("703.3");
    let mut t = TestGame::new(2);
    let watcher = oracle_card(
        "Draw Watcher",
        "Creature — Faerie",
        "{0}",
        Some((1, 1)),
        "At the beginning of your draw step, target player gains 1 life.",
    );
    t.custom(P0, watcher, Zone::Battlefield);
    t.g.turn.number = 3; // not the starting player's first turn (CR 103.8a)
    let seen = spy(&mut t, P0, |g, p, d| match d {
        Decision::ChooseTargets { .. } => Some(format!("targets:{}", g.player(p).hand.len())),
        Decision::Priority { .. } if g.turn.step == Step::Draw => {
            Some(format!("priority:{}", g.player(p).hand.len()))
        }
        _ => None,
    });
    t.set_step(P0, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::Draw);
    t.g.advance();
    // The card was drawn before the trigger was put on the stack (its target was chosen
    // then) and before P0 received priority.
    let lines = probe_lines(&seen);
    assert_eq!(
        lines.first().map(String::as_str),
        Some(&*format!("targets:{}", hand + 1))
    );
    assert!(lines.iter().any(|l| l == &format!("priority:{}", hand + 1)));

    // State-based actions are checked after the turn-based action: a player whose draw
    // step draw fails loses before anyone gets priority.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    let lib = t.player(P1).library.clone();
    for c in lib {
        t.g.move_object(c, Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    t.g.run_until(100, |g| g.result.is_some() || g.turn.step != Step::Upkeep);
    t.g.run_until(100, |g| {
        g.result.is_some() || g.turn.stage == Stage::Priority
    });
    t.g.advance();
    assert!(t.has_lost(P1));
}

#[test]
fn the_turn_based_actions_happen_in_their_steps() {
    cr!("703.4", "703.4c", "703.4d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(bears);
    t.g.tap(theirs);
    t.set_step(P1, Step::End);
    let hand = t.hand_size(P0);
    let their_hand = t.hand_size(P1);
    to_step_start(&mut t, P0, Step::Upkeep);
    // Untap step: the active player's permanents untapped, before the upkeep began.
    assert!(!t.obj_now(bears).tapped);
    assert!(t.obj_now(theirs).tapped);
    assert!(
        event_index(
            &t,
            |e| matches!(e, Event::Untapped { obj } if *obj == bears)
        )
        .unwrap()
            < event_index(&t, step_began(Step::Upkeep)).unwrap()
    );
    assert_eq!(t.hand_size(P0), hand);
    // Draw step: the active player drew.
    to_step_start(&mut t, P0, Step::Draw);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.hand_size(P1), their_hand);
}

#[test]
fn untap_step_phasing_happens_before_untapping() {
    cr!("703.4a", "703.4c");
    supported("Breezekeeper");
    let mut t = TestGame::new(2);
    // One phased-in phasing creature, and one tapped creature that phased out.
    let a = t.battlefield(P0, "Breezekeeper");
    let b = t.battlefield(P0, "Breezekeeper");
    t.g.tap(a);
    t.g.tap(b);
    mtg_engine::kw::phasing::phase_out(&mut t.g, vec![b]);
    t.g.recompute();
    assert!(t.obj(b).phased_out);
    // An opponent's phased-out permanent doesn't phase in during P0's untap step.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    mtg_engine::kw::phasing::phase_out(&mut t.g, vec![theirs]);
    t.set_step(P1, Step::End);
    to_step_start(&mut t, P0, Step::Upkeep);
    // Simultaneously: the phased-in one phased out; the phased-out one phased in.
    assert!(t.obj(a).phased_out);
    assert!(!t.obj(b).phased_out);
    // Untapping came after phasing: the one that phased in untapped, while the one that
    // phased out was treated as though it didn't exist and stayed tapped.
    assert!(!t.obj(b).tapped);
    assert!(t.obj(a).tapped);
    assert!(t.obj(theirs).phased_out);
}

#[test]
fn untap_step_checks_day_and_night_after_phasing() {
    cr!("703.4b");
    let mut t = TestGame::new(2);
    t.g.day = Some(true);
    // P1 casts no spells during their turn: at P0's untap step it becomes night.
    t.set_step(P1, Step::Upkeep);
    to_step_start(&mut t, P0, Step::Upkeep);
    assert_eq!(t.g.day, Some(false));
    let changed = event_index(&t, |e| matches!(e, Event::DayNightChanged { .. })).unwrap();
    assert!(changed > event_index(&t, step_began(Step::Untap)).unwrap());
    assert!(changed < event_index(&t, step_began(Step::Upkeep)).unwrap());
    // If it's neither day nor night, the check doesn't happen.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::Upkeep);
    to_step_start(&mut t, P0, Step::Upkeep);
    assert_eq!(t.g.day, None);
}

#[test]
fn precombat_main_phase_adds_lore_counters_to_sagas() {
    cr!("703.4f");
    let mut t = TestGame::new(2);
    let saga = oracle_card(
        "Test Saga",
        "Enchantment — Saga",
        "{0}",
        None,
        "(As this Saga enters and after your draw step, add a lore counter.)\nI — You gain 1 life.\nII — You gain 2 life.\nIII — You gain 3 life.",
    );
    let s = t.custom(P0, saga, Zone::Battlefield);
    t.g.add_counters(Entity::Object(s), counters::LORE, 1, None);
    let theirs = t.custom(
        P1,
        oracle_card(
            "Their Saga",
            "Enchantment — Saga",
            "{0}",
            None,
            "I — You gain 1 life.\nII — You gain 2 life.\nIII — You gain 3 life.",
        ),
        Zone::Battlefield,
    );
    t.g.add_counters(Entity::Object(theirs), counters::LORE, 1, None);
    t.set_step(P0, Step::Draw);
    to_step_start(&mut t, P0, Step::PrecombatMain);
    assert_eq!(t.counters(s, counters::LORE), 2);
    // Only the active player's Sagas, and only in the precombat main phase.
    assert_eq!(t.counters(theirs, counters::LORE), 1);
    t.resolve_all();
    to_step_start(&mut t, P0, Step::PostcombatMain);
    assert_eq!(t.counters(s, counters::LORE), 2);
}

#[test]
fn beginning_of_combat_chooses_a_defending_player_in_multiplayer() {
    cr!("703.4h");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            attack_multiple_players: false,
            ..Default::default()
        },
    );
    t.answer_choose(P0, &[Entity::Player(P2)]);
    to_step_start(&mut t, P0, Step::BeginningOfCombat);
    assert_eq!(t.g.combat.as_ref().unwrap().defending_players, vec![P2]);
    // With all opponents automatically defending, no choice is made.
    let mut t = TestGame::new(3);
    let asked = t.asked().len();
    to_step_start(&mut t, P0, Step::BeginningOfCombat);
    assert_eq!(t.g.combat.as_ref().unwrap().defending_players, vec![P1, P2]);
    assert!(t.asked()[asked..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseEntities { .. })));
}

#[test]
fn combat_steps_declare_attackers_blockers_and_deal_damage_simultaneously() {
    cr!("703.4i", "703.4j", "703.4k", "703.4m");
    let mut t = TestGame::new(2);
    let attacker = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P1))]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(b1, attacker), (b2, attacker)]),
    );
    to_step_start(&mut t, P0, Step::DeclareAttackers);
    assert!(t.obj(attacker).tapped);
    let asked = t.asked();
    let attack_q = asked
        .iter()
        .position(|(p, d)| *p == P0 && matches!(d, Decision::DeclareAttackers { .. }))
        .unwrap();
    assert!(asked[attack_q + 1..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::DeclareBlockers { .. })));
    to_step_start(&mut t, P0, Step::DeclareBlockers);
    let asked = t.asked();
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::DeclareBlockers { .. })));
    to_step_start(&mut t, P0, Step::CombatDamage);
    // The attacking player announced the assignment; all damage was dealt at once, so the
    // blockers dealt their damage even though the Giant's damage destroys one of them.
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::AssignCombatDamage { .. })));
    let dmg: Vec<_> = t
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::Damage { combat: true, .. }))
        .collect();
    assert_eq!(dmg.len(), 4);
    t.settle();
    // The Giant divided its 3 damage: lethal damage to one Bears, 1 to the other.
    assert_eq!([b1, b2].iter().filter(|b| t.on_battlefield(**b)).count(), 1);
    // Its 3 toughness took 4 damage from the two Bears.
    assert!(!t.on_battlefield(attacker));
}

#[test]
fn cleanup_discard_then_damage_removal_and_end_of_turn_effects_together() {
    cr!("703.4n", "703.4p");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Giant Growth: 5/5 until end of turn, with 3 damage marked on it. If the effect ended
    // before the damage was removed, it would be destroyed.
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(bears).go();
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    t.g.obj_mut(bears).damage = 3;
    for _ in 0..9 {
        t.hand(P0, "Hill Giant");
    }
    to_step_start(&mut t, P0, Step::End);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.hand_size(P0), 7);
    assert_eq!(t.graveyard_size(P0), 2 + 1); // two discarded Giants and Giant Growth
}

#[test]
fn mana_empties_as_each_step_and_phase_ends() {
    cr!("703.4q");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Upkeep);
    t.g.players[0]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::G, 3);
    assert_eq!(t.player(P0).mana_pool.total(), 3);
    to_step_start(&mut t, P0, Step::Draw);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    t.g.players[1]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::R, 1);
    to_step_start(&mut t, P0, Step::PrecombatMain);
    assert_eq!(t.player(P1).mana_pool.total(), 0);
}
