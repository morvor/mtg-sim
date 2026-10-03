//! CR 724: ending turns and phases.

use crate::r506_common::custom_card;
use mtg_engine::end_turn::end_the_combat_phase;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

/// Both players pass priority in turn, starting with `first` (the top of the stack
/// resolves).
fn both_pass(t: &mut TestGame, first: PlayerId) {
    let other = if first == P0 { P1 } else { P0 };
    t.g.turn.passes = 0;
    t.g.pass_priority(first);
    t.g.pass_priority(other);
}

/// Advances the game until `active`'s turn begins, returning the steps that began and
/// whether any player received priority during the rest of the current turn.
fn run_to_turn_of(t: &mut TestGame, active: PlayerId) -> (Vec<Step>, bool) {
    let mut steps = Vec::new();
    let mut priority = false;
    let turn = t.g.turn.number;
    for _ in 0..500 {
        if t.g.turn.number != turn && t.g.turn.active == active {
            return (steps, priority);
        }
        let before = t.g.turn.stage;
        t.g.advance();
        if t.g.turn.number == turn {
            if before == Stage::Begin {
                steps.push(t.g.turn.step);
            }
            if t.g.turn.stage == Stage::Priority && t.g.turn.priority.is_some() {
                priority = true;
            }
        }
    }
    panic!("{active}'s turn never began");
}

#[test]
fn ending_the_turn_exiles_the_stack_and_skips_to_the_cleanup_step() {
    cr!("724.1", "724.1b", "724.1d", "724.1e");
    ruling!(
        "Time Stop",
        "Ending the turn this way means the following things happen in order: 1) All spells and abilities on the stack are exiled. This includes Time Stop, though it will continue to resolve."
    );
    ruling!(
        "Time Stop",
        "Unless Time Stop is cast during the Ending phase, any \"at the beginning of the end step\"-triggered abilities don't get the chance to trigger on the turn Time Stop is cast."
    );
    let mut t = TestGame::new(2);
    // "At the beginning of your end step, you gain 1 life for each creature you control
    // with vigilance."
    t.battlefield(P0, "Alert Heedbonder");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 6);
    t.lands(P0, "Mountain", 2);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(giant).go();
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    let stop = t.hand(P0, "Time Stop");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.cast(P0, stop).go();
    assert_eq!(t.g.stack.len(), 2);
    both_pass(&mut t, P0);
    // Every object on the stack was exiled, including Time Stop itself: Lightning Bolt
    // never resolves.
    assert!(t.g.stack.is_empty());
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.in_exile("Time Stop"));
    assert!(!t.in_graveyard(P0, "Time Stop"));
    assert_eq!(t.life(P1), 20);
    let life = t.life(P0);
    let (steps, priority) = run_to_turn_of(&mut t, P1);
    // The game skipped straight to the cleanup step: no combat, no end step (so the
    // Heedbonder's ability didn't trigger), and no one received priority.
    assert_eq!(steps, vec![Step::Cleanup]);
    assert!(!priority);
    assert_eq!(t.life(P0), life);
    // The cleanup step happened in its entirety: damage wore off.
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn abilities_that_triggered_before_the_turn_ends_cease_to_exist() {
    cr!("724.1a");
    ruling!(
        "Day's Undoing",
        "If any abilities trigger while players are shuffling cards into their library or drawing seven cards, those abilities cease to exist when the turn ends. They won’t be put on the stack."
    );
    let mut t = TestGame::new(2);
    // "Whenever an opponent draws a card, Underworld Dreams deals 1 damage to that player."
    t.battlefield(P1, "Underworld Dreams");
    t.lands(P0, "Island", 3);
    let undoing = t.hand(P0, "Day's Undoing");
    t.cast(P0, undoing).go();
    both_pass(&mut t, P0);
    // P0 drew seven cards, then (it's P0's turn) the turn ended.
    assert_eq!(t.hand_size(P0), 7);
    assert!(t.g.pending_triggers.is_empty());
    assert!(t.g.stack.is_empty());
    let (steps, _) = run_to_turn_of(&mut t, P1);
    assert_eq!(steps, vec![Step::Cleanup]);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn abilities_that_trigger_during_the_process_are_put_on_the_stack_in_the_cleanup_step() {
    cr!("724.1c", "724.1f");
    ruling!(
        "Sundial of the Infinite",
        "If any triggered abilities do trigger during this process, they're put onto the stack during the cleanup step. If this happens, players will have a chance to cast spells and activate abilities, then there will be another cleanup step before the turn finally ends."
    );
    let mut t = TestGame::new(2);
    let sundial = t.battlefield(P0, "Sundial of the Infinite");
    t.lands(P0, "Plains", 1);
    // "When Doomed Traveler dies, create a 1/1 white Spirit creature token with flying."
    let traveler = t.battlefield(P0, "Doomed Traveler");
    t.activate(P0, sundial, 0, &[]).unwrap();
    // Lethal damage is marked on the Traveler before the ability resolves (state-based
    // actions haven't been checked yet).
    t.g.objects[traveler.0 as usize].damage = 1;
    both_pass(&mut t, P0);
    // The state-based action check during the process put the Traveler into the
    // graveyard; its ability waits, as no player gets priority.
    assert!(t.in_graveyard(P0, "Doomed Traveler"));
    assert!(t.g.stack.is_empty());
    assert_eq!(t.g.pending_triggers.len(), 1);
    // In the cleanup step the ability is put onto the stack and P0 receives priority.
    let ok = t.g.run_until(50, |g| {
        g.turn.step == Step::Cleanup && g.turn.stage == Stage::Priority && !g.stack.is_empty()
    });
    assert!(ok);
    assert_eq!(t.g.turn.priority, Some(P0));
    let (steps, _) = run_to_turn_of(&mut t, P1);
    // The Spirit was created, then another cleanup step followed before the turn ended.
    assert_eq!(t.named_on_battlefield("Spirit Token").len(), 1);
    assert_eq!(steps, vec![Step::Cleanup]);
}

#[test]
fn ending_the_turn_during_the_cleanup_step_begins_a_new_cleanup_step() {
    cr!("724.1d", "724.1b");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P0, "Sundial of the Infinite");
    let second = t.battlefield(P0, "Sundial of the Infinite");
    t.lands(P0, "Plains", 2);
    let traveler = t.battlefield(P0, "Doomed Traveler");
    t.activate(P0, first, 0, &[]).unwrap();
    t.g.objects[traveler.0 as usize].damage = 1;
    both_pass(&mut t, P0);
    let ok = t.g.run_until(50, |g| {
        g.turn.step == Step::Cleanup && g.turn.stage == Stage::Priority && !g.stack.is_empty()
    });
    assert!(ok);
    let cleanups = |t: &TestGame| {
        t.g.turn
            .step_log
            .iter()
            .filter(|s| **s == Step::Cleanup)
            .count()
    };
    assert_eq!(cleanups(&t), 1);
    // In response to the Traveler's ability, P0 ends the turn again.
    t.activate(P0, second, 0, &[]).unwrap();
    both_pass(&mut t, P0);
    // The Traveler's ability was exiled with the stack; a new cleanup step begins.
    assert!(t.g.stack.is_empty());
    t.g.advance();
    t.g.advance();
    assert_eq!(t.g.turn.step, Step::Cleanup);
    assert_eq!(cleanups(&t), 2);
    run_to_turn_of(&mut t, P1);
    assert!(t.named_on_battlefield("Spirit Token").is_empty());
}

#[test]
fn ending_the_turn_during_combat_removes_creatures_from_combat() {
    cr!("724.1d");
    ruling!(
        "Glorious End",
        "2) If there are any attacking and blocking creatures, they’re removed from combat."
    );
    let mut t = TestGame::new(2);
    let sundial = t.battlefield(P0, "Sundial of the Infinite");
    t.lands(P0, "Plains", 1);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().is_some_and(|c| c.attacker(bear).is_some()));
    t.activate(P0, sundial, 0, &[]).unwrap();
    both_pass(&mut t, P0);
    assert!(t.g.combat.is_none());
    let (steps, _) = run_to_turn_of(&mut t, P1);
    assert_eq!(steps, vec![Step::Cleanup]);
    // No combat damage was dealt.
    assert_eq!(t.life(P1), 20);
}

#[test]
fn obeka_lets_the_player_whose_turn_it_is_end_the_turn() {
    cr!("724.1");
    ruling!(
        "Obeka, Brute Chronologist",
        "If the turn ends before the end step, any \"At the beginning of the next end step\" triggered abilities won't get the chance to trigger that turn because the end step has been skipped."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let obeka = t.battlefield(P0, "Obeka, Brute Chronologist");
    // P1, whose turn it is, declines: the turn goes on.
    t.answer_yes(P1, false);
    t.activate(P0, obeka, 0, &[]).unwrap();
    both_pass(&mut t, P1);
    assert_eq!(t.g.turn.stage, Stage::Priority);
    assert_eq!(t.g.turn.step, Step::PrecombatMain);
    t.g.objects[obeka.0 as usize].tapped = false;
    t.answer_yes(P1, true);
    t.activate(P0, obeka, 0, &[]).unwrap();
    both_pass(&mut t, P1);
    let (steps, _) = run_to_turn_of(&mut t, P0);
    assert_eq!(steps, vec![Step::Cleanup]);
}

/// A custom instant for the combat-phase tests: "Target player draws a card. End the
/// combat phase."
fn draw_and_end_combat() -> CardDef {
    let mut c = custom_card(
        "Parley of Peace",
        "Instant",
        None,
        "Target player draws a card. End the combat phase.",
    );
    let cost = mtg_engine::mana::ManaCost::parse("{W}").unwrap();
    c.faces[0].chars.colors = cost.colors();
    c.faces[0].chars.mana_cost = Some(cost);
    c
}

#[test]
fn ending_the_combat_phase_skips_to_the_postcombat_main_phase() {
    cr!("724.2", "724.2b", "724.2d", "724.2e");
    ruling!(
        "Mandate of Peace",
        "Ending the combat phase this way means the following things happen in order: 1) All spells and abilities on the stack are exiled."
    );
    ruling!(
        "Mandate of Peace",
        "Any “at end of combat” triggered abilities won’t get the chance to trigger that combat because the end of combat step is skipped."
    );
    let mut t = TestGame::new(2);
    // "At end of combat, destroy all creatures blocking or blocked by this creature."
    let frostbeast = t.battlefield(P0, "Kjeldoran Frostbeast");
    // "Whenever this creature attacks and isn't blocked, it gets +2/+0 until end of
    // combat."
    let dwellers = t.battlefield(P0, "Murk Dwellers");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Plains", 2);
    t.lands(P0, "Forest", 1);
    let mandate = t.hand(P1, "Mandate of Peace");
    let growth = t.hand(P0, "Giant Growth");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (frostbeast, Entity::Player(P1)),
            (dwellers, Entity::Player(P1)),
        ]),
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bear, frostbeast)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
    assert_eq!(t.pt(dwellers).0, 4);
    t.cast(P0, growth).target(dwellers).go();
    t.cast(P1, mandate).go();
    both_pass(&mut t, P1);
    // Everything on the stack was exiled; the creatures were removed from combat and the
    // "until end of combat" effect ended.
    assert!(t.g.stack.is_empty());
    assert!(t.in_exile("Giant Growth"));
    assert!(t.in_exile("Mandate of Peace"));
    assert!(t.g.combat.is_none());
    assert_eq!(t.pt(dwellers).0, 2);
    // The game skips to the postcombat main phase: no combat damage, no end of combat
    // step, so the Frostbeast's ability doesn't trigger.
    let ok = t.g.run_until(50, |g| g.turn.stage == Stage::Priority);
    assert!(ok);
    assert_eq!(t.g.turn.step, Step::PostcombatMain);
    assert!(!t.g.turn.step_log.contains(&Step::CombatDamage));
    assert!(!t.g.turn.step_log.contains(&Step::EndOfCombat));
    assert!(t.g.stack.is_empty());
    assert!(t.on_battlefield(bear));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn abilities_triggering_while_ending_the_combat_phase() {
    cr!("724.2a", "724.2c", "724.2f");
    ruling!(
        "Mandate of Peace",
        "If any triggered abilities trigger during this process, they’re put onto the stack after the game skips to the appropriate phase or step."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Underworld Dreams");
    let traveler = t.battlefield(P0, "Doomed Traveler");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P1, "Plains", 1);
    let parley = t.custom(P1, draw_and_end_combat(), Zone::Hand(P1));
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.cast(P1, parley).target(Entity::Player(P0)).go();
    t.g.objects[traveler.0 as usize].damage = 1;
    both_pass(&mut t, P1);
    // Underworld Dreams triggered on the draw before the process began: that ability
    // ceased to exist. The Traveler died to the state-based action check during the
    // process: its ability waits for the next phase.
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.in_graveyard(P0, "Doomed Traveler"));
    assert!(t.g.stack.is_empty());
    assert_eq!(t.g.pending_triggers.len(), 1);
    let ok = t.g.run_until(50, |g| g.turn.stage == Stage::Priority);
    assert!(ok);
    assert_eq!(t.g.turn.step, Step::PostcombatMain);
    t.settle();
    assert_eq!(t.g.stack.len(), 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Spirit Token").len(), 1);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn ending_the_combat_phase_outside_combat_does_nothing() {
    cr!("724.2g");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    end_the_combat_phase(&mut t.g);
    assert_eq!(t.g.stack.len(), 1);
    assert_eq!(t.g.turn.step, Step::PrecombatMain);
    assert_eq!(t.g.turn.stage, Stage::Priority);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
