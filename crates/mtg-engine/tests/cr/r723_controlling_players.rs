//! CR 723: controlling another player.

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

/// `p` activates a Mindslaver targeting `target`, and it resolves.
fn mindslaver(t: &mut TestGame, p: PlayerId, target: PlayerId) {
    let slaver = t.battlefield(p, "Mindslaver");
    t.lands(p, "Island", 4);
    t.activate(p, slaver, 0, &[Entity::Player(target)])
        .expect("activate Mindslaver");
    t.resolve_all();
}

#[test]
fn mindslaver_controls_the_player_during_their_next_turn() {
    cr!("723.1", "723.3", "723.5", "723.5a", "723.8");
    ruling!(
        "Mindslaver",
        "The player you're controlling is still the active player during that turn."
    );
    ruling!(
        "Mindslaver",
        "You only control the player. You don't control any of that player's permanents, spells, or abilities."
    );
    ruling!(
        "Worst Fears",
        "You can use only the affected player's resources (cards, mana, and so on) to pay costs for that player; you can't use your own."
    );
    ruling!(
        "Mindslaver",
        "While controlling another player, you also continue to make your own choices and decisions."
    );
    let (mut t, log) = recorded(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let mountain = t.battlefield(P1, "Mountain");
    let bolt = t.hand(P1, "Lightning Bolt");
    mindslaver(&mut t, P0, P1);
    let my_mountain = t.battlefield(P0, "Mountain");
    // Not yet: the effect applies to P1's next turn.
    assert_eq!(decider(&t.g, P1), P1);
    t.g.pass_priority(P0);
    t.g.run_until(1, |_| false);
    assert_eq!(deciders(&log, 1, P1), vec![P1]);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    // P0 has P1 cast P1's Lightning Bolt at P1.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.advance();
    let spell = *t.g.stack.last().expect("the Bolt was cast");
    // P1 is still the active player, and the spell and permanents are P1's.
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(t.g.obj(spell).controller, P1);
    assert_eq!(t.g.obj(bear).controller, P1);
    // Only P1's resources paid for it.
    assert!(t.g.obj(mountain).tapped);
    assert!(!t.g.obj(my_mountain).tapped);
    t.advance_to(P1, Step::Draw);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // Every decision for P1 this turn was P0's; P0 kept making their own decisions.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(deciders(&log, 2, P1), vec![P0]);
    assert_eq!(deciders(&log, 2, P0), vec![P0]);
    // The effect lasted until the beginning of the next turn.
    assert_eq!(decider(&t.g, P1), P1);
    t.advance_to(P0, Step::Draw);
    assert_eq!(deciders(&log, 3, P1), vec![P1]);
}

#[test]
fn the_last_player_controlling_effect_created_is_the_one_that_works() {
    cr!("723.1a");
    ruling!(
        "Mindslaver",
        "Multiple player-controlling effects that affect the same player overwrite each other. The last one to be created is the one that works."
    );
    let mut t = TestGame::new(3);
    mindslaver(&mut t, P0, P2);
    mindslaver(&mut t, P1, P2);
    t.advance_to(P1, Step::Upkeep);
    // P2's own next turn hasn't begun: P2 still decides.
    assert_eq!(decider(&t.g, P2), P2);
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(decider(&t.g, P2), P1);
    // The overwritten effect doesn't wait for another turn.
    t.advance_to(P2, Step::End);
    let turn = t.g.turn.number;
    let ok = t
        .g
        .run_until(500, |g| g.turn.number > turn + 2 && g.turn.active == P2);
    assert!(ok);
    assert_eq!(decider(&t.g, P2), P2);
}

#[test]
fn an_effect_may_give_a_player_control_of_themselves() {
    cr!("723.9", "723.1a");
    ruling!(
        "Worst Fears",
        "You could gain control of yourself using Worst Fears, but unless you do so to overwrite someone else's player-controlling effect, this doesn't do anything."
    );
    let mut t = TestGame::new(2);
    mindslaver(&mut t, P0, P1);
    // In response to nothing, P1 overwrites P0's effect by controlling themselves.
    mindslaver(&mut t, P1, P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P1);
}

#[test]
fn a_skipped_turn_doesnt_use_up_the_effect() {
    cr!("723.1b");
    ruling!(
        "Mindslaver",
        "If the targeted player skips their next turn, you'll control the next turn the affected player actually takes."
    );
    let mut t = TestGame::new(2);
    mindslaver(&mut t, P0, P1);
    t.g.players[1].skips.push(StepKind::Turn);
    // P1's next turn is skipped: P0 takes another turn, then P1's turn is controlled.
    let turn = t.g.turn.number;
    let ok = t.g.run_until(500, |g| g.turn.number == turn + 1);
    assert!(ok);
    assert_eq!(t.g.turn.active, P0);
    assert_eq!(decider(&t.g, P1), P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
}

#[test]
fn the_controller_cant_make_the_player_concede() {
    cr!("723.6", "723.5b");
    ruling!(
        "Mindslaver",
        "You can't make the affected player concede. That player may choose to concede at any time, even while you're controlling that player."
    );
    let (mut t, log) = recorded(2);
    mindslaver(&mut t, P0, P1);
    t.advance_to(P1, Step::Upkeep);
    // The actions offered to P0 for P1 don't include conceding, and trying anyway does
    // nothing.
    t.answer(P0, DecisionKind::Priority, Answer::Action(Action::Concede));
    t.g.advance();
    assert!(!t.has_lost(P1));
    let offered = log
        .lock()
        .unwrap()
        .iter()
        .rev()
        .find_map(|(_, a, p, d)| match d {
            Decision::Priority { actions } if *a == P0 && *p == P1 => Some(actions.clone()),
            _ => None,
        })
        .unwrap();
    assert!(!offered.contains(&Action::Concede));
    // P1 may still concede.
    t.take_action(P1, Action::Concede);
    assert!(t.has_lost(P1));
}

#[test]
fn tournament_decisions_arent_made_by_the_controller() {
    cr!("723.5b");
    ruling!(
        "Mindslaver",
        "You also can't make any choices or decisions for the player that would be called for by the tournament rules (such as whether to take an intentional draw or whether to call a judge)."
    );
    let (mut t, log) = recorded(2);
    mindslaver(&mut t, P0, P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    // P0 would agree for P1; P1 doesn't.
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    assert!(!t.g.propose_intentional_draw());
    assert_eq!(t.g.result, None);
    let asked: Vec<(PlayerId, PlayerId)> = log
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, _, _, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(_, a, p, _)| (*a, *p))
        .collect();
    assert!(asked.contains(&(P1, P1)));
    assert!(!asked.contains(&(P0, P1)));
}

#[test]
fn the_controller_sees_what_the_player_sees_but_not_cards_outside_the_game() {
    cr!("723.4");
    ruling!(
        "Mindslaver",
        "While controlling another player, you can see all cards in the game that player can see. This includes cards in that player's hand, face-down cards that player controls, and any cards in that player's library the player may look at."
    );
    ruling!(
        "Mindslaver",
        "Controlling a player doesn't allow you to look at that player's sideboard. If an effect instructs that player to choose a card from outside the game, you can't have that player choose any card."
    );
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P1, "Grizzly Bears");
    let outside = t.custom(P1, (*card("Lava Spike")).clone(), Zone::Outside(P1));
    let wish = t.hand(P1, "Burning Wish");
    t.lands(P1, "Mountain", 2);
    assert!(!can_see(&t.g, P0, in_hand));
    mindslaver(&mut t, P0, P1);
    t.advance_to(P1, Step::PrecombatMain);
    assert!(can_see(&t.g, P0, in_hand));
    assert!(!can_see(&t.g, P0, outside));
    assert!(can_see(&t.g, P1, outside));
    // P0 has P1 cast Burning Wish: P1 can't be made to choose a card from outside the
    // game.
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

#[test]
fn opposition_agent_controls_opponents_while_they_search_their_libraries() {
    cr!("723.2", "723.5");
    ruling!(
        "Opposition Agent",
        "While controlling an opponent, you make all choices and decisions for that player. However, because the control effect is limited to while they're searching their libraries, it's unlikely the player will be allowed to make any decisions other than what to find with the search."
    );
    let (mut t, log) = recorded(2);
    t.battlefield(P0, "Opposition Agent");
    t.set_step(P1, Step::PrecombatMain);
    let forest = t.library_top(P1, "Forest");
    t.library_top(P1, "Ancestral Recall");
    t.lands(P1, "Swamp", 2);
    let tutor = t.hand(P1, "Demonic Tutor");
    assert_eq!(decider(&t.g, P1), P1);
    // P0 makes P1 find the Forest.
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.cast(P1, tutor).go();
    t.resolve_all();
    assert!(t.in_hand(P1, "Forest"));
    assert!(!t.in_hand(P1, "Ancestral Recall"));
    let searches: Vec<PlayerId> = log
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, _, p, d)| *p == P1 && matches!(d, Decision::ChooseEntities { .. }))
        .map(|(_, a, _, _)| *a)
        .collect();
    assert_eq!(searches, vec![P0]);
    // Only while searching.
    assert_eq!(decider(&t.g, P1), P1);
}

#[test]
fn emrakul_controls_the_opponents_next_turn_then_they_take_an_extra_turn() {
    cr!("723.1");
    ruling!(
        "Emrakul, the Promised End",
        "If the targeted player skips their next turn, you’ll control the next turn the affected player actually takes, and the extra turn the player takes will be after that turn."
    );
    let mut t = TestGame::new(2);
    let emrakul = t.hand(P0, "Emrakul, the Promised End");
    t.lands(P0, "Island", 13);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, emrakul).go();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    let controlled = t.g.turn.number;
    // After that turn, P1 takes an extra turn, which P1 controls.
    let ok = t.g.run_until(500, |g| g.turn.number == controlled + 1);
    assert!(ok);
    assert_eq!(t.g.turn.active, P1);
    assert!(t.g.turn.extra);
    assert_eq!(decider(&t.g, P1), P1);
}

#[test]
fn secret_of_bloodbending_controls_the_opponent_during_their_next_combat_phase() {
    cr!("723.5");
    ruling!(
        "Secret of Bloodbending",
        "If the targeted player skips their next combat phase or turn, you'll control the next combat phase or turn the affected player actually takes."
    );
    let (mut t, log) = recorded(2);
    t.lands(P0, "Island", 4);
    let secret = t.hand(P0, "Secret of Bloodbending");
    let bear = t.battlefield(P1, "Grizzly Bears");
    // Without waterbending: control during P1's next combat phase only.
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, secret).target(Entity::Player(P1)).go();
    t.resolve_all();
    // P1 skips their next combat phase: the effect waits for the next one P1 has.
    t.g.players[1].skips.push(StepKind::Combat);
    t.advance_to(P1, Step::PostcombatMain);
    assert!(!t.g.turn.step_log.contains(&Step::BeginningOfCombat));
    assert_eq!(decider(&t.g, P1), P1);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(decider(&t.g, P1), P1);
    // P0 has P1's Bears attack P0.
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(decider(&t.g, P1), P0);
    // An additional combat phase follows this one: P0 controls only this one.
    t.g.add_extra_combat(false);
    let ok = t.g.run_until(200, |g| {
        g.turn.step_log.iter().filter(|s| **s == Step::BeginningOfCombat).count() == 2
    });
    assert!(ok);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    assert_eq!(decider(&t.g, P1), P1);
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(t.life(P0), 18);
    assert_eq!(decider(&t.g, P1), P1);
    let attackers: Vec<PlayerId> = log
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, _, p, d)| *p == P1 && matches!(d, Decision::DeclareAttackers { .. }))
        .map(|(_, a, _, _)| *a)
        .collect();
    assert_eq!(attackers, vec![P0]);
}

/// P0 casts Word of Command targeting P1, choosing `chosen` from P1's hand.
fn word_of_command(t: &mut TestGame, chosen: ObjectId) {
    t.lands(P0, "Swamp", 2);
    let woc = t.hand(P0, "Word of Command");
    t.answer_choose(P0, &[Entity::Object(chosen)]);
    t.cast(P0, woc).target(Entity::Player(P1)).go();
    t.g.turn.passes = 0;
    t.g.pass_priority(P0);
    t.g.pass_priority(P1);
}

#[test]
fn word_of_command_makes_the_player_play_the_card_with_lands_mana_only() {
    cr!("723.7", "723.2");
    ruling!(
        "Word of Command",
        "You must order your opponent to play the chosen card if it is possible to do so."
    );
    // P1's only untapped mana source isn't a land: they can't play Mind Stone.
    let mut t = TestGame::new(2);
    let sol_ring = t.battlefield(P1, "Sol Ring");
    let stone = t.hand(P1, "Mind Stone");
    word_of_command(&mut t, stone);
    assert!(t.in_hand(P1, "Mind Stone"));
    assert!(!t.g.obj(sol_ring).tapped);
    assert!(t.in_graveyard(P0, "Word of Command"));
    // With lands, P1 must play it.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sol Ring");
    let lands = t.lands(P1, "Wastes", 2);
    let stone = t.hand(P1, "Mind Stone");
    word_of_command(&mut t, stone);
    assert_eq!(t.g.stack.len(), 1);
    assert_eq!(t.obj_now(stone).controller, P1);
    assert!(lands.iter().all(|l| t.g.obj(*l).tapped));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Mind Stone").len(), 1);
    // The control lasted only until Word of Command finished resolving.
    assert_eq!(decider(&t.g, P1), P1);
}

/// An agent that makes another player discard the last cards offered.
struct LastPicker {
    inner: ScriptedAgent,
    log: Log,
}

impl Agent for LastPicker {
    fn name(&self) -> &str {
        "last picker"
    }
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        self.log
            .lock()
            .unwrap()
            .push((g.turn.number, self.inner.player, p, d.clone()));
        match d {
            Decision::ChooseEntities {
                prompt,
                candidates,
                min,
                ..
            } if p != self.inner.player && prompt.to_lowercase().contains("discard") => {
                Answer::Entities(candidates[candidates.len() - *min as usize..].to_vec())
            }
            _ => self.inner.decide(g, p, d),
        }
    }
}

#[test]
fn word_of_command_controls_the_player_while_the_chosen_spell_resolves() {
    cr!("723.2", "723.7");
    ruling!(
        "Word of Command",
        "During the resolution of this spell, that player plays the chosen card."
    );
    let (mut t, log) = recorded(2);
    t.g.set_agent(
        P0,
        Box::new(LastPicker {
            inner: ScriptedAgent {
                player: P0,
                script: t.script.clone(),
            },
            log: log.clone(),
        }),
    );
    // "Draw two cards, then discard two cards."
    let study = t.hand(P1, "Careful Study");
    t.hand(P1, "Grizzly Bears");
    t.library_top(P1, "Hill Giant");
    t.library_top(P1, "Savannah Lions");
    t.lands(P1, "Island", 1);
    word_of_command(&mut t, study);
    // P1 cast Careful Study during Word of Command's resolution; now P1 decides again.
    assert_eq!(t.g.stack.len(), 1);
    assert_eq!(decider(&t.g, P1), P1);
    t.resolve_all();
    // While Careful Study resolved, P0 made P1's discard choice: the two cards drawn.
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Savannah Lions"));
    let discards: Vec<PlayerId> = log
        .lock()
        .unwrap()
        .iter()
        .filter(|(_, _, p, d)| {
            *p == P1
                && matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("discard"))
        })
        .map(|(_, a, _, _)| *a)
        .collect();
    assert_eq!(discards, vec![P0]);
}

#[test]
fn secret_of_bloodbending_with_its_additional_cost_paid_controls_the_whole_turn() {
    cr!("723.1");
    ruling!(
        "Secret of Bloodbending",
        "The player you're controlling is still the active player during that turn."
    );
    // Waterbend {10} paid: control during P1's whole next turn.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 14);
    let secret = t.hand(P0, "Secret of Bloodbending");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, secret).target(Entity::Player(P1)).go();
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(decider(&t.g, P1), P0);
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(decider(&t.g, P1), P0);
    // Cast with flashback (an alternative cost) without waterbending: only P1's next
    // combat phase.
    let mut t = TestGame::new(2);
    let secret = t.graveyard(P0, "Secret of Bloodbending");
    t.answer_targets(P0, &[Entity::Object(secret)]);
    t.enter(P0, "Snapcaster Mage");
    t.resolve_all();
    t.lands(P0, "Island", 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.cast(P0, secret)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert!(t.in_exile("Secret of Bloodbending"));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P1);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(decider(&t.g, P1), P0);
}

#[test]
fn with_shared_team_turns_the_controller_controls_the_affected_players_team() {
    cr!("805.8");
    ruling!(
        "Secret of Bloodbending",
        "In a Two-Headed Giant game, gaining control of a player causes you to gain control of each player on that team."
    );
    let mut t = TestGame::with_config(
        4,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    );
    mindslaver(&mut t, P0, P2);
    t.advance_to(P2, Step::Upkeep);
    assert_eq!(decider(&t.g, P2), P0);
    assert_eq!(decider(&t.g, P3), P0);
    // P0's teammate isn't affected.
    assert_eq!(decider(&t.g, P1), P1);
}
