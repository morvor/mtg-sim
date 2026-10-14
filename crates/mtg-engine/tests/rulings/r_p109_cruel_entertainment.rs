//! Rulings batch P109 — Cruel Entertainment: "Choose target player and another target
//! player. The first player controls the second player during the second player's next
//! turn, and the second player controls the first player during the first player's next
//! turn." Controlling another player (CR 723).

use crate::r_p009_common::pl;
use crate::r_s01_common::supported;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::ability::StepKind;
use mtg_engine::decision::Decision;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::player_control::{can_see, decider};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

/// (turn, agent's player, player the decision was for, the decision).
type Log = Arc<Mutex<Vec<(u32, PlayerId, PlayerId, Decision)>>>;

struct Recorder {
    inner: ScriptedAgent,
    log: Log,
}

impl Agent for Recorder {
    fn name(&self) -> &str {
        "recorder"
    }
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        self.log
            .lock()
            .unwrap()
            .push((g.turn.number, self.inner.player, p, d.clone()));
        self.inner.decide(g, p, d)
    }
}

/// A game of `n` players whose agents record who made each decision.
fn recorded(n: usize) -> (TestGame, Log) {
    let mut t = TestGame::new(n);
    let log: Log = Arc::default();
    for i in 0..n {
        let p = PlayerId(i as u8);
        let agent = Recorder {
            inner: ScriptedAgent {
                player: p,
                script: t.script.clone(),
            },
            log: log.clone(),
        };
        t.g.set_agent(p, Box::new(agent));
    }
    (t, log)
}

/// The players whose agents made decisions for `p` during turn `turn`.
fn deciders(log: &Log, turn: u32, p: PlayerId) -> Vec<PlayerId> {
    let mut v: Vec<PlayerId> = log
        .lock()
        .unwrap()
        .iter()
        .filter(|(t, _, q, _)| *t == turn && *q == p)
        .map(|(_, a, _, _)| *a)
        .collect();
    v.sort();
    v.dedup();
    v
}

/// P0 casts Cruel Entertainment targeting `first` and `second`, and it resolves.
fn entertain(t: &mut TestGame, first: PlayerId, second: PlayerId) {
    supported("Cruel Entertainment");
    lands_for_cost(t, P0, "Cruel Entertainment");
    let ce = t.hand(P0, "Cruel Entertainment");
    t.answer_targets(P0, &[pl(first)]);
    t.answer_targets(P0, &[pl(second)]);
    t.cast(P0, ce).go();
    t.clear_answers();
    t.resolve_all();
}

#[test]
fn each_player_controls_the_other_during_the_others_next_turn() {
    cr!("723.1", "723.3", "723.8", "115.3");
    ruling!(
        "Cruel Entertainment",
        "Cruel Entertainment’s controller may be one of its targets, even if it’s less entertaining this way."
    );
    ruling!(
        "Cruel Entertainment",
        "The player being controlled is still the active player during their turn."
    );
    ruling!(
        "Cruel Entertainment",
        "The controlling player only controls the player. The controlling player doesn’t control any of that player’s permanents, spells, or abilities."
    );
    ruling!(
        "Cruel Entertainment",
        "While controlling another player, the controlling player also continues to make their own choices and decisions."
    );
    ruling!(
        "Cruel Entertainment",
        "While controlling another player, the controlling player makes all choices and decisions that the player being controlled is allowed to make or is told to make during that turn."
    );
    let (mut t, log) = recorded(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P0 targets itself and P1.
    entertain(&mut t, P0, P1);
    assert_eq!(decider(&t.g, P1), P1, "not yet");
    assert_eq!(decider(&t.g, P0), P0);
    // P1's turn: P0 controls P1. P1 is the active player and still controls its Bears.
    t.advance_to(P1, Step::Upkeep);
    let turn = t.g.turn.number;
    assert_eq!(decider(&t.g, P1), P0);
    assert_eq!(decider(&t.g, P0), P0, "P0 keeps making its own decisions");
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(t.obj(bears).controller, P1);
    // P0's next turn: P1 controls P0.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(deciders(&log, turn, P1), vec![P0]);
    assert_eq!(deciders(&log, turn, P0), vec![P0]);
    assert_eq!(decider(&t.g, P0), P1);
    assert_eq!(decider(&t.g, P1), P1);
    assert_eq!(t.g.turn.active, P0);
    // Then nobody is controlled any more.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P1);
    assert_eq!(decider(&t.g, P0), P0);
}

#[test]
fn a_skipped_turn_waits_for_the_next_turn_taken() {
    cr!("723.1b");
    ruling!(
        "Cruel Entertainment",
        "If a player who would be controlled skips their next turn, the other player will control the next turn the affected player actually takes."
    );
    let mut t = TestGame::new(3);
    entertain(&mut t, P1, P2);
    t.g.players[2].skips.push(StepKind::Turn);
    // P1's turn: P2 controls P1.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P2);
    // P2's turn is skipped; P1 controls P2 during the next one P2 takes.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(decider(&t.g, P2), P2);
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(decider(&t.g, P2), P1);
}

#[test]
fn a_player_who_left_the_game_controls_nobody() {
    cr!("723.1", "800.4a");
    ruling!(
        "Cruel Entertainment",
        "If one player loses the game before controlling the other player, the other player takes their turn normally."
    );
    let mut t = TestGame::new(3);
    entertain(&mut t, P1, P2);
    // P1's turn (controlled by P2); then P1 loses before P2's turn.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P2);
    t.g.lose_game(P1);
    t.g.flush_events();
    t.settle();
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(decider(&t.g, P2), P2);
}

#[test]
fn the_controlled_players_resources_and_legal_choices_only() {
    cr!("723.5", "723.5a", "723.6");
    ruling!(
        "Cruel Entertainment",
        "The controlling player can use only the resources of the player being controlled (cards, mana, and so on) to pay costs for that player; the controlling player can’t use their own resources to pay costs for the player being controlled."
    );
    ruling!(
        "Cruel Entertainment",
        "The controlling player can’t make any illegal decisions or illegal choices—that player can’t do anything that the player being controlled couldn’t do."
    );
    let mut t = TestGame::new(2);
    let mountain = t.battlefield(P1, "Mountain");
    let bolt = t.hand(P1, "Lightning Bolt");
    let spike = t.hand(P1, "Lava Spike");
    entertain(&mut t, P0, P1);
    let mine = t.lands(P0, "Mountain", 3);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(decider(&t.g, P1), P0);
    // P0 has P1 cast Lightning Bolt: only P1's Mountain pays.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P0, &[pl(P1)]);
    t.g.advance();
    assert_eq!(t.g.stack.len(), 1);
    assert!(t.g.obj(mountain).tapped);
    assert!(mine.iter().all(|m| !t.g.obj(*m).tapped));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // P1 has no more mana: P0 can't have P1 cast Lava Spike with P0's untapped lands.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: spike,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P0, &[pl(P1)]);
    t.g.advance();
    t.clear_answers();
    assert!(t.g.stack.is_empty());
    assert!(t.in_hand(P1, "Lava Spike"));
    assert!(mine.iter().all(|m| !t.g.obj(*m).tapped));
}

#[test]
fn the_controller_cant_concede_for_the_player() {
    cr!("723.6");
    ruling!(
        "Cruel Entertainment",
        "The controlling player can’t make the player being controlled concede. A player may choose to concede at any time, even while under another player’s control."
    );
    let mut t = TestGame::new(2);
    entertain(&mut t, P0, P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    t.answer(P0, DecisionKind::Priority, Answer::Action(Action::Concede));
    t.g.advance();
    assert!(!t.has_lost(P1));
    t.take_action(P1, Action::Concede);
    assert!(t.has_lost(P1));
}

#[test]
fn the_controller_sees_what_the_player_sees_but_not_their_sideboard() {
    cr!("723.4");
    ruling!(
        "Cruel Entertainment",
        "While controlling another player, the controlling player can see all cards in the game that the player being controlled can see. This includes cards in that player’s hand, face-down cards that player controls, and any cards in that player’s library that they may look at."
    );
    ruling!(
        "Cruel Entertainment",
        "Controlling a player doesn’t allow the controlling player to look at the controlled player’s sideboard. If an effect instructs that player to choose a card from outside the game, the controlling player can’t have that player choose a card."
    );
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P1, "Grizzly Bears");
    let outside = t.custom(P1, (*card("Lava Spike")).clone(), Zone::Outside(P1));
    let wish = t.hand(P1, "Burning Wish");
    t.lands(P1, "Mountain", 2);
    assert!(!can_see(&t.g, P0, in_hand));
    entertain(&mut t, P0, P1);
    t.advance_to(P1, Step::PrecombatMain);
    assert!(can_see(&t.g, P0, in_hand));
    assert!(!can_see(&t.g, P0, outside));
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: wish,
            method: CastMethod::Normal,
        }),
    );
    t.g.advance();
    assert_eq!(t.g.stack.len(), 1);
    t.resolve_all();
    assert!(!t.in_hand(P1, "Lava Spike"));
    assert_eq!(t.zone(outside), Zone::Outside(P1));
}
