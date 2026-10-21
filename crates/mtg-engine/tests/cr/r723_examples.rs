//! The worked examples of CR 723 (controlling another player), played out as tests.

use mtg_engine::decision::Decision;
use mtg_engine::facedown;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::player_control::{can_see, decider};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

/// (turn, agent's player, player the decision was for, the decision).
type Log = Arc<Mutex<Vec<(u32, PlayerId, PlayerId, Decision)>>>;

/// A scripted agent that records which player's agent made each decision.
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

/// A game whose agents record who made each decision.
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

/// The agents that made the decisions for `p` matching `f`.
fn agents_for(log: &Log, p: PlayerId, f: impl Fn(&Decision) -> bool) -> Vec<PlayerId> {
    log.lock()
        .unwrap()
        .iter()
        .filter(|(_, _, q, d)| *q == p && f(d))
        .map(|(_, a, _, _)| *a)
        .collect()
}

/// P0 activates a Mindslaver targeting P1 ("{4}, {T}, Sacrifice Mindslaver: You control
/// target player during that player's next turn."), and P1's next turn begins.
fn control_p1_next_turn(t: &mut TestGame, step: Step) {
    let slaver = t.battlefield(P0, "Mindslaver");
    t.lands(P0, "Island", 4);
    t.activate(P0, slaver, 0, &[Entity::Player(P1)])
        .expect("activate Mindslaver");
    t.resolve_all();
    t.advance_to(P1, step);
    assert_eq!(decider(&t.g, P1), P0);
}

fn cast_for_p1(t: &mut TestGame, card: ObjectId) {
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card,
            method: CastMethod::Normal,
        }),
    );
}

#[test]
fn the_controller_sees_the_players_hand_and_face_down_creatures() {
    cr!("723.4", "708.5");
    cr_example!("723.4", "face-down creatures they control");
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P1, "Grizzly Bears");
    let creature = t.battlefield(P1, "Hill Giant");
    assert!(facedown::turn_face_down(&mut t.g, creature));
    t.g.recompute();
    let face_down = t.g.current(creature);
    assert!(t.obj(face_down).face_down);
    // Before: P0 sees neither.
    assert!(!can_see(&t.g, P0, in_hand));
    assert!(!can_see(&t.g, P0, face_down));
    assert!(can_see(&t.g, P1, face_down));
    control_p1_next_turn(&mut t, Step::Upkeep);
    // While controlling P1, P0 sees P1's hand and the face of P1's face-down creature.
    assert!(can_see(&t.g, P0, in_hand));
    assert!(can_see(&t.g, P0, face_down));
}

#[test]
fn the_controller_casts_the_players_spells_chooses_targets_and_resolution_choices() {
    cr!("723.5", "723.3");
    cr_example!("723.5", "decides which spells that player casts");
    let (mut t, log) = recorded(2);
    // P1 holds Careful Study ("Draw two cards, then discard two cards."), a Lightning
    // Bolt, a Hill Giant and an Elite Vanguard.
    let study = t.hand(P1, "Careful Study");
    let bolt = t.hand(P1, "Lightning Bolt");
    let hill = t.hand(P1, "Hill Giant");
    let vanguard = t.hand(P1, "Elite Vanguard");
    t.lands(P1, "Island", 1);
    t.lands(P1, "Mountain", 1);
    let my_bears = t.battlefield(P0, "Grizzly Bears");
    control_p1_next_turn(&mut t, Step::PrecombatMain);
    // P0 has P1 cast Lightning Bolt, targeting P1 itself rather than P0's creature.
    cast_for_p1(&mut t, bolt);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.advance();
    assert_eq!(t.g.stack.len(), 1);
    assert_eq!(t.obj(t.g.stack[0]).controller, P1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.on_battlefield(my_bears));
    // P0 has P1 cast Careful Study, and chooses P1's discards as it resolves.
    cast_for_p1(&mut t, study);
    t.g.advance();
    assert_eq!(t.g.stack.len(), 1);
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Object(hill), Entity::Object(vanguard)]),
    );
    t.resolve_all();
    let discard = |d: &Decision| {
        matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("discard"))
    };
    assert_eq!(agents_for(&log, P1, discard), vec![P0]);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Elite Vanguard"));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert!(t.in_graveyard(P1, "Careful Study"));
    // Every decision for P1 this turn was made by P0's agent.
    let turn = t.g.turn.number;
    assert!(log
        .lock()
        .unwrap()
        .iter()
        .filter(|(n, _, q, _)| *n == turn && *q == P1)
        .all(|(_, a, _, _)| *a == P0));
}

#[test]
fn the_controller_declares_the_players_attacks_and_assigns_their_damage() {
    cr!("723.5", "508.1", "510.1c");
    cr_example!("723.5", "which of that player’s creatures attack");
    let (mut t, log) = recorded(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P0, "Jace Beleren");
    let b1 = t.battlefield(P0, "Llanowar Elves");
    let b2 = t.battlefield(P0, "Elvish Mystic");
    control_p1_next_turn(&mut t, Step::PrecombatMain);
    // P0 has Hill Giant attack Jace Beleren and Grizzly Bears attack P0; as themselves,
    // P0 blocks the Bears with both elves. Then P0 assigns all of the Bears' 2 damage to
    // the first one listed.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![
            (giant, Entity::Object(jace)),
            (bears, Entity::Player(P0)),
        ]),
    );
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(b1, bears), (b2, bears)]),
    );
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 0]));
    t.advance_to(P1, Step::EndOfCombat);
    let attacks = agents_for(&log, P1, |d| matches!(d, Decision::DeclareAttackers { .. }));
    assert_eq!(attacks, vec![P0]);
    let damage: Vec<(PlayerId, Vec<Entity>)> = log
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(_, a, p, d)| match d {
            Decision::AssignCombatDamage { recipients, .. } if *p == P1 => {
                Some((*a, recipients.clone()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(damage.len(), 1);
    assert_eq!(damage[0].0, P0);
    let first = damage[0].1[0].object().unwrap();
    let other = if first == b1 { b2 } else { b1 };
    assert!(!t.on_battlefield(first), "dealt 2 damage");
    assert!(t.on_battlefield(other), "dealt no damage");
    // Hill Giant attacked Jace Beleren (3 loyalty): it's gone. P0 took no damage.
    assert!(!t.on_battlefield(jace));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_discard_cost_paid_for_the_controlled_player_comes_from_their_hand() {
    cr!("723.5a", "601.2h");
    cr_example!("723.5a", "additional cost of discarding cards");
    let (mut t, log) = recorded(2);
    // Thrill of Possibility: "As an additional cost to cast this spell, discard a card.
    // Draw two cards."
    let thrill = t.hand(P1, "Thrill of Possibility");
    let fodder = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 2);
    let mine = t.hand(P0, "Hill Giant");
    control_p1_next_turn(&mut t, Step::PrecombatMain);
    let p0_hand = t.hand_size(P0);
    cast_for_p1(&mut t, thrill);
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Object(fodder)]),
    );
    t.g.advance();
    assert_eq!(t.g.stack.len(), 1);
    // P0 chose the discard; only P1's cards could be chosen.
    let offered: Vec<(PlayerId, Vec<Entity>)> = log
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(_, a, p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P1 => {
                Some((*a, candidates.clone()))
            }
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0].0, P0);
    assert!(offered[0].1.contains(&Entity::Object(fodder)));
    assert!(!offered[0].1.contains(&Entity::Object(mine)));
    assert_eq!(t.zone(fodder), Zone::Graveyard(P1));
    assert_eq!(t.zone(mine), Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), p0_hand);
}

#[test]
fn tournament_style_decisions_stay_with_the_controlled_player() {
    cr!("723.5b");
    cr_example!("723.5b", "agree to an intentional draw");
    let (mut t, log) = recorded(2);
    control_p1_next_turn(&mut t, Step::Upkeep);
    // P0 would agree to an intentional draw for P1; P1, asked themselves, doesn't.
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    assert!(!t.g.propose_intentional_draw());
    assert_eq!(t.g.result, None);
    let yes_no = agents_for(&log, P1, |d| matches!(d, Decision::YesNo { .. }));
    assert_eq!(yes_no, vec![P1]);
}
