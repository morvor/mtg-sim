//! Rulings batch P140 — removing things from the stack and from combat, and ending the
//! turn: Reconnaissance, Sundial of the Infinite, Obeka, Brute Chronologist and Nivmagus
//! Elemental.

use crate::r_p140_common::*;
use crate::r_s09_common::declare;
use mtg_engine::decision::{Agent, Answer, Decision};
use mtg_engine::game::Game;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::counters;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

#[test]
fn reconnaissance_doesnt_undo_attack_triggers() {
    cr!("506.4", "603.2");
    ruling!(
        "Reconnaissance",
        "Does not undo any effects which triggered on declaration of attackers or blockers."
    );
    supported("Reconnaissance");
    supported("Roxanne, Starfall Savant");
    let mut t = TestGame::new(2);
    let recon = t.battlefield(P0, "Reconnaissance");
    let rox = t.battlefield(P0, "Roxanne, Starfall Savant");
    declare(&mut t, P0, &[(rox, Entity::Player(P1))]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Meteorite"), 1, "the attack trigger");
    activate_lots(&mut t, P0, recon, 0, &[obj(rox)]);
    t.resolve();
    assert!(!t.obj(rox).tapped, "untapped");
    assert!(
        crate::r_s09_common::attacks_now(&t).is_empty(),
        "removed from combat"
    );
    t.resolve_all();
    assert_eq!(
        controlled_named(&t, P0, "Meteorite").len(),
        1,
        "the trigger still resolves"
    );
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
}

/// Both players pass priority in turn, starting with `first`: the top of the stack
/// resolves as it does in the priority loop.
fn both_pass(t: &mut TestGame, first: PlayerId) {
    let other = if first == P0 { P1 } else { P0 };
    t.g.turn.passes = 0;
    t.g.pass_priority(first);
    t.g.pass_priority(other);
}

#[test]
fn sundial_ends_the_turn_in_order_exiling_even_uncounterable_spells() {
    cr!("724.1", "724.1b", "724.1c", "724.1d", "724.1f", "514.1");
    ruling!(
        "Sundial of the Infinite",
        "Ending the turn this way means the following things happen in order: 1) All spells and abilities on the stack are exiled."
    );
    ruling!(
        "Sundial of the Infinite",
        "Though spells and abilities that are exiled won't get a chance to resolve, they don't count as being \"countered.\""
    );
    supported("Sundial of the Infinite");
    supported("Doomed Traveler");
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Sundial of the Infinite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    let traveler = t.battlefield(P0, "Doomed Traveler");
    crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    // P1 casts Abrupt Decay (can't be countered) at the Sundial; P0 ends the turn.
    let decay = cast_lots(&mut t, P1, "Abrupt Decay", &[obj(dial)]);
    for _ in 0..9 {
        t.hand(P0, "Island");
    }
    activate_lots(&mut t, P0, dial, 0, &[]);
    assert_eq!(t.stack_len(), 2);
    // Lethal damage is marked on Doomed Traveler before the ability resolves (state-based
    // actions haven't been checked since), so the check during the process kills it.
    t.g.objects[traveler.0 as usize].damage = 1;
    both_pass(&mut t, P0);
    // 1) The stack is exiled, the uncounterable spell included, without countering it.
    assert_eq!(zone_of(&t, decay), Zone::Exile, "exiled, not resolved");
    assert!(t.on_battlefield(dial));
    assert!(t.stack.is_empty());
    assert!(!anything_countered(&t));
    // 2) Attacking and blocking creatures are removed from combat.
    assert!(
        t.g.combat
            .as_ref()
            .is_none_or(|c| c.attackers.is_empty() && c.blockers.is_empty()),
        "no attackers or blockers remain"
    );
    // 3) State-based actions are checked, but no triggered ability is put onto the stack
    // and no one gets priority: the Traveler's dies trigger waits.
    assert!(t.in_graveyard(P0, "Doomed Traveler"));
    assert_eq!(t.g.pending_triggers.len(), 1, "the dies trigger waits");
    // 4) Straight to the cleanup step: P0 discards down to seven there and the waiting
    // trigger is put onto the stack, with priority, before another cleanup step.
    let ok = t.g.run_until(50, |g| {
        g.turn.step == Step::Cleanup && g.turn.stage == Stage::Priority && !g.stack.is_empty()
    });
    assert!(ok, "the trigger goes on the stack in the cleanup step");
    assert_eq!(t.g.turn.step_log.last(), Some(&Step::Cleanup));
    assert!(
        !t.g.turn.step_log.contains(&Step::CombatDamage) && !t.g.turn.step_log.contains(&Step::End),
        "{:?}",
        t.g.turn.step_log
    );
    assert_eq!(t.hand_size(P0), 7);
    // No combat damage was dealt.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.named_on_battlefield("Spirit Token").len(), 1);
    assert_eq!(t.obj(t.g.current(bears)).damage, 0);
    assert!(t.on_battlefield(t.g.current(bears)));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn ending_the_turn_early_delays_beginning_of_step_triggers_to_the_next_turn() {
    cr!("724.1e", "603.7b", "504.1");
    ruling!(
        "Sundial of the Infinite",
        "If Sundial of the Infinite's ability is activated before the end step, any \"at the beginning of the end step\"-triggered abilities won't get the chance to trigger that turn because the end step is skipped."
    );
    supported("Chrome Dome");
    supported("Howling Mine");
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Sundial of the Infinite");
    let dome = t.battlefield(P0, "Chrome Dome");
    let thopter = t.battlefield(P0, "Ornithopter");
    activate_lots(&mut t, P0, dome, 0, &[obj(thopter)]);
    t.resolve_all();
    let copies = controlled_named(&t, P0, "Ornithopter");
    assert_eq!(copies.len(), 2);
    let token = *copies.iter().find(|c| t.obj(**c).is_token()).unwrap();
    activate_lots(&mut t, P0, dial, 0, &[]);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(token), "no end step this turn");
    // Those abilities trigger at the beginning of the end step of the next turn (P1's).
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(token), "sacrificed at the next end step");

    // The same for other steps, except upkeep: ending the turn in the upkeep skips the
    // draw step and its "at the beginning of each player's draw step" trigger, which
    // triggers again in the next turn's draw step; the upkeep trigger did trigger.
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Sundial of the Infinite");
    t.battlefield(P0, "Phyrexian Arena");
    t.battlefield(P0, "Howling Mine");
    crate::r_p141_common::into_upkeep(&mut t, P0);
    assert_eq!(
        triggers_on_stack(&t, "draw a card"),
        1,
        "the upkeep trigger"
    );
    let (hand0, hand1) = (t.hand_size(P0), t.hand_size(P1));
    activate_lots(&mut t, P0, dial, 0, &[]);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.hand_size(P0), hand0, "no draw step, no Howling Mine draw");
    assert_eq!(t.life(P0), 20);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(
        t.hand_size(P1),
        hand1 + 2,
        "P1's draw step: the draw and Howling Mine's"
    );
}

#[test]
fn obeka_ending_the_turn_after_next_end_step_triggers_doesnt_retrigger_them() {
    cr!("724.1b", "603.7b");
    ruling!(
        "Obeka, Brute Chronologist",
        "If the turn ends during the end step after \"At the beginning of the next end step\" abilities have triggered but before they've resolved, those abilities won't trigger again next turn."
    );
    supported("Obeka, Brute Chronologist");
    let mut t = TestGame::new(2);
    let obeka = t.battlefield(P0, "Obeka, Brute Chronologist");
    let dome = t.battlefield(P0, "Chrome Dome");
    let thopter = t.battlefield(P0, "Ornithopter");
    activate_lots(&mut t, P0, dome, 0, &[obj(thopter)]);
    t.resolve_all();
    let token = *controlled_named(&t, P0, "Ornithopter")
        .iter()
        .find(|c| t.obj(**c).is_token())
        .unwrap();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "the delayed sacrifice trigger");
    t.answer_yes(P0, true);
    activate_lots(&mut t, P0, obeka, 0, &[]);
    t.resolve();
    assert!(t.stack.is_empty(), "the turn ended: the trigger was exiled");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    assert!(t.on_battlefield(token), "never sacrificed");
}

/// What the game looked like each time a player was asked for a priority action.
#[derive(Clone, Debug)]
struct PrioritySeen {
    player: PlayerId,
    active: PlayerId,
    step: Step,
    stack: usize,
}

/// Records every priority decision of the game (who, whose turn, which step, how many
/// objects were on the stack), then answers as the scripted agent would.
fn record_priority(t: &mut TestGame) -> Arc<Mutex<Vec<PrioritySeen>>> {
    struct Recorder {
        inner: Box<dyn Agent>,
        seen: Arc<Mutex<Vec<PrioritySeen>>>,
    }
    impl Agent for Recorder {
        fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
            if let Decision::Priority { .. } = d {
                self.seen.lock().unwrap().push(PrioritySeen {
                    player: p,
                    active: g.turn.active,
                    step: g.turn.step,
                    stack: g.stack.len(),
                });
            }
            self.inner.decide(g, p, d)
        }
    }
    let seen = Arc::new(Mutex::new(vec![]));
    let mut agents = t.g.agents.0.lock().unwrap();
    for a in agents.iter_mut() {
        let inner = std::mem::replace(a, Box::new(mtg_engine::decision::PassiveAgent));
        *a = Box::new(Recorder {
            inner,
            seen: seen.clone(),
        });
    }
    seen
}

#[test]
fn sundial_earliest_in_your_upkeep_after_upkeep_triggers() {
    cr!("502.4", "503.1", "724.1b");
    ruling!(
        "Sundial of the Infinite",
        "The earliest that you can activate Sundial of the Infinite's ability is during your upkeep step, after abilities that trigger \"at the beginning of your upkeep\" have been put onto the stack but before they resolve."
    );
    let mut t = TestGame::new(2);
    let dial = t.battlefield(P0, "Sundial of the Infinite");
    t.battlefield(P0, "Phyrexian Arena");
    // From P1's end step, the game runs into P0's turn; P0's first chance to act is the
    // first priority decision of that turn.
    t.set_step(P1, Step::End);
    let seen = record_priority(&mut t);
    let ok = t.g.run_until(10_000, |g| {
        g.turn.active == P0 && seen_in_turn(&seen, P0) > 0
    });
    assert!(ok);
    let first = seen
        .lock()
        .unwrap()
        .iter()
        .find(|s| s.active == P0)
        .cloned()
        .unwrap();
    assert!(
        seen.lock().unwrap().iter().all(|s| s.step != Step::Untap),
        "no one gets priority in the untap step"
    );
    assert_eq!((first.player, first.step), (P0, Step::Upkeep), "{first:?}");
    assert_eq!(
        first.stack, 1,
        "the Arena's trigger is already on the stack"
    );
    // Then P0 ends the turn: the trigger never resolves.
    lots(&mut t, P0);
    assert!(can_activate(&mut t, P0, dial));
    activate_lots(&mut t, P0, dial, 0, &[]);
    t.resolve();
    assert!(t.stack.is_empty());
    assert_eq!(t.life(P0), 20, "the Arena's trigger never resolved");
}

/// How many priority decisions were recorded during `p`'s turns.
fn seen_in_turn(seen: &Arc<Mutex<Vec<PrioritySeen>>>, p: PlayerId) -> usize {
    seen.lock()
        .unwrap()
        .iter()
        .filter(|s| s.active == p)
        .count()
}

#[test]
fn nivmagus_exiles_an_uncounterable_spell_without_countering_it() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Nivmagus Elemental",
        "Exiling a spell is not the same as countering it, although the spell won’t resolve in both cases. You may exile a spell that can’t be countered."
    );
    supported("Nivmagus Elemental");
    let mut t = TestGame::new(2);
    let niv = t.battlefield(P0, "Nivmagus Elemental");
    let stone = t.battlefield(P1, "Millstone");
    let decay = cast_lots(&mut t, P0, "Abrupt Decay", &[obj(stone)]);
    t.answer_choose(P0, &[obj(decay)]);
    activate_lots(&mut t, P0, niv, 0, &[]);
    assert_eq!(zone_of(&t, decay), Zone::Exile, "exiled as a cost");
    t.resolve_all();
    assert_eq!(t.counters(niv, counters::PLUS1), 2);
    assert!(t.on_battlefield(stone), "Abrupt Decay didn't resolve");
    assert!(!anything_countered(&t));
}

#[test]
fn nivmagus_can_exile_a_copy_you_control_but_not_cards() {
    cr!("707.10", "118.3");
    ruling!(
        "Nivmagus Elemental",
        "You can exile a copy of an instant or sorcery spell to activate Nivmagus Elemental’s ability if you control the copy."
    );
    ruling!(
        "Nivmagus Elemental",
        "You exile an instant or sorcery spell you control from the stack, not an instant or sorcery card in any other zone."
    );
    // Cards in hand and graveyard don't count.
    let mut t = TestGame::new(2);
    let niv = t.battlefield(P0, "Nivmagus Elemental");
    t.hand(P0, "Lightning Bolt");
    t.graveyard(P0, "Lightning Bolt");
    lots(&mut t, P0);
    assert!(!can_activate(&mut t, P0, niv));
    // A copy of Lightning Bolt that P0 controls.
    let bolt = cast_lots(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.answer_yes(P0, false);
    cast_lots(&mut t, P0, "Twincast", &[obj(bolt)]);
    t.resolve();
    let copy = *t
        .stack
        .iter()
        .find(|s| t.obj(**s).kind == ObjKind::SpellCopy)
        .expect("a copy");
    assert_eq!(t.obj(copy).controller, P0);
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![obj(copy)]),
    );
    activate_lots(&mut t, P0, niv, 0, &[]);
    assert!(!t.stack.contains(&copy), "the copy was exiled");
    t.resolve_all();
    assert_eq!(t.counters(niv, counters::PLUS1), 2);
    assert_eq!(t.life(P1), 17, "only the original Bolt resolved");
}
