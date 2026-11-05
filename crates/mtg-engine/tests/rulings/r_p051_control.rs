//! Rulings batch P051 — Sorin cards: controlling a player (CR 723), control-changing
//! effects with no duration (CR 611.2a, 800.4a), and a planeswalker's loyalty abilities
//! the turn it enters (CR 606.3).

use crate::r_p051_common::*;
use crate::r_p179_common::battlefield_back;
use crate::r_s01_common::*;
use crate::r_s05_common::move_to;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::player_control::decider;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

/// `who` activates Sorin Markov's "−7: You control target player during that player's
/// next turn." targeting `target`, and it resolves.
fn sorin_markov(t: &mut TestGame, who: PlayerId, target: PlayerId) {
    supported("Sorin Markov");
    let sorin = t.battlefield(who, "Sorin Markov");
    t.g.add_counters(obj(sorin), counters::LOYALTY, 3, None);
    let active = t.g.turn.active;
    t.set_step(who, Step::PrecombatMain);
    t.activate(who, sorin, 2, &[pl(target)]).expect("−7");
    t.resolve_all();
    t.set_step(active, Step::PrecombatMain);
}

#[test]
fn sorin_markov_controls_only_the_player_who_stays_the_active_player() {
    cr!("723.3", "723.5");
    ruling!(
        "Sorin Markov",
        "The player who is being controlled is still the active player."
    );
    ruling!(
        "Sorin Markov",
        "You only control the player. You don’t control any of the other player’s permanents, spells, or abilities."
    );
    ruling!(
        "Sorin Markov",
        "While controlling another player, you make all choices and decisions that player is allowed to make or is told to make during that turn."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, BEARS);
    t.battlefield(P1, "Mountain");
    let bolt = t.hand(P1, "Lightning Bolt");
    sorin_markov(&mut t, P0, P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    assert_eq!(t.g.turn.active, P1);
    // P0 has P1 cast P1's Lightning Bolt at P1's own Bears, choosing the target.
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: bolt,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P0, &[obj(bears)]);
    t.g.advance();
    let spell = *t.g.stack.last().expect("the Bolt was cast");
    assert_eq!(t.g.obj(spell).controller, P1);
    assert_eq!(t.g.obj(bears).controller, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, BEARS));
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn sorin_markov_cant_make_the_player_concede() {
    cr!("723.6", "104.3a");
    ruling!(
        "Sorin Markov",
        "You can’t make the affected player concede. That player may choose to concede at any time, even while you’re controlling their turn."
    );
    let mut t = TestGame::new(2);
    sorin_markov(&mut t, P0, P1);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(decider(&t.g, P1), P0);
    t.answer(P0, DecisionKind::Priority, Answer::Action(Action::Concede));
    t.g.advance();
    assert!(!t.has_lost(P1));
    assert!(!t.has_lost(P0));
    t.take_action(P1, Action::Concede);
    assert!(t.has_lost(P1));
}

/// A 3-player game in which P0 controls P1 during P1's turn (Sorin Markov) and P2 casts
/// Master Warcraft ("Cast this spell only before attackers are declared. You choose which
/// creatures attack this turn. You choose which creatures block this turn and how those
/// creatures block.") in P1's beginning of combat step: P2, not P0, chooses whether P1's
/// Bears attack.
#[test]
fn sorin_markov_controller_cant_make_illegal_decisions_and_yields_to_master_warcraft() {
    cr!("723.5", "723.5a", "508.1");
    ruling!(
        "Sorin Markov",
        "You can’t make any illegal decisions or illegal choices — you can’t do anything that player couldn’t do."
    );
    // P1 has no mana: P0 can't have P1 cast P1's Lightning Bolt, not even with P0's own
    // untapped Mountains.
    let mut t = TestGame::new(2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let mine = t.lands(P0, "Mountain", 2);
    sorin_markov(&mut t, P0, P1);
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(decider(&t.g, P1), P0);
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
    t.clear_answers();
    assert!(t.g.stack.is_empty());
    assert!(t.in_hand(P1, "Lightning Bolt"));
    assert!(mine.iter().all(|m| !t.obj_now(*m).tapped));
    assert_eq!(t.life(P1), 20);
    // Master Warcraft takes precedence over controlling the player.
    supported("Master Warcraft");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, BEARS);
    t.lands(P2, "Mountain", 4);
    let mw = t.hand(P2, "Master Warcraft");
    sorin_markov(&mut t, P0, P1);
    t.advance_to(P1, Step::BeginningOfCombat);
    assert_eq!(decider(&t.g, P1), P0);
    t.cast(P2, mw).go();
    t.resolve_all();
    // P0 (deciding for P1) would declare no attackers; P2 makes the Bears attack P0.
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.answer(
        P2,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, pl(P0))]),
    );
    let turn = t.g.turn.number;
    assert!(t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers && g.turn.stage == mtg_engine::turn::Stage::Priority)
            || g.turn.number != turn
    }));
    t.settle();
    assert_eq!(crate::r_p015_common::attacking(&t, bears), Some(pl(P0)));
}

#[test]
fn sorin_markov_on_yourself_only_overwrites_another_control_effect() {
    cr!("723.1a", "723.9");
    ruling!(
        "Sorin Markov",
        "You could gain control of yourself using Sorin’s third ability, but unless you do so to overwrite someone else’s player-controlling effect, this doesn’t do anything."
    );
    // Alone, it changes nothing.
    let mut t = TestGame::new(2);
    sorin_markov(&mut t, P0, P0);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(decider(&t.g, P0), P0);
    // After P1's Sorin targeted P0, P0's own Sorin overwrites it.
    let mut t = TestGame::new(2);
    sorin_markov(&mut t, P1, P0);
    sorin_markov(&mut t, P0, P0);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(decider(&t.g, P0), P0);
    // Without the overwrite, P1 controls P0's next turn.
    let mut t = TestGame::new(2);
    sorin_markov(&mut t, P1, P0);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(decider(&t.g, P0), P1);
}

#[test]
fn sorin_ravenous_neonate_control_lasts_indefinitely() {
    cr!("611.2a");
    ruling!(
        "Sorin of House Markov // Sorin, Ravenous Neonate",
        "The control effect created by Sorin, Ravenous Neonate's last ability lasts indefinitely. It doesn't wear off during the cleanup step or if Sorin, Ravenous Neonate leaves the battlefield."
    );
    supported("Sorin of House Markov // Sorin, Ravenous Neonate");
    let mut t = TestGame::new(2);
    let sorin = battlefield_back(
        &mut t,
        P0,
        "Sorin of House Markov // Sorin, Ravenous Neonate",
    );
    t.g.objects[sorin.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    let bears = t.battlefield(P1, BEARS);
    assert!(activate_text(
        &mut t,
        P0,
        sorin,
        "Gain control",
        &[obj(bears)]
    ));
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(t.obj_now(bears).chars.has_subtype("Vampire"));
    // Past the cleanup step.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bears).controller, P0);
    // Sorin leaves the battlefield.
    assert!(t.on_battlefield(sorin));
    move_to(&mut t, sorin, Zone::Graveyard(P0));
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn sorin_vampire_lord_granted_control_lasts_until_you_leave_the_game() {
    cr!("611.2a", "800.4a");
    ruling!(
        "Sorin, Vampire Lord",
        "The control-change effect from ability granted by Sorin's last ability lasts indefinitely. It doesn't wear off during the cleanup step, and it doesn't expire if Sorin or a Vampire whose ability was activated leaves the battlefield. In a multiplayer game, it does expire if you leave the game."
    );
    supported("Sorin, Vampire Lord");
    let mut t = TestGame::new(3);
    let sorin = t.battlefield(P0, "Sorin, Vampire Lord");
    t.g.objects[sorin.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 8);
    let vamp = t.battlefield(P0, "Vampire Interloper");
    let bears = t.battlefield(P1, BEARS);
    t.activate(P0, sorin, 2, &[]).unwrap();
    t.resolve_all();
    assert!(activate_text(
        &mut t,
        P0,
        vamp,
        "Gain control",
        &[obj(bears)]
    ));
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    t.advance_to(P1, Step::Upkeep);
    for id in [sorin, vamp] {
        if t.on_battlefield(id) {
            move_to(&mut t, id, Zone::Graveyard(P0));
        }
    }
    assert_eq!(t.obj_now(bears).controller, P0);
    t.take_action(P0, Action::Concede);
    t.settle();
    assert!(t.has_lost(P0));
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn sorin_ravenous_neonate_can_activate_the_turn_he_transforms_at_sorcery_speed() {
    cr!("606.3");
    ruling!(
        "Sorin of House Markov // Sorin, Ravenous Neonate",
        "You can activate one of Sorin, Ravenous Neonate's loyalty abilities the turn he enters the battlefield. However, you may do so only during one of your main phases when the stack is empty."
    );
    supported("Sorin of House Markov // Sorin, Ravenous Neonate");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sorin of House Markov // Sorin, Ravenous Neonate");
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.set_step(P0, Step::EndOfCombat);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    let neonate = t.named_on_battlefield("Sorin, Ravenous Neonate");
    assert_eq!(neonate.len(), 1);
    let neonate = neonate[0];
    // With a spell on the stack: no.
    let bolt = t.hand(P1, "Lightning Bolt");
    crate::r_p046_common::fill_pool(&mut t, P1, 1);
    t.cast(P1, bolt).target(pl(P1)).go();
    assert!(!activate_text(&mut t, P0, neonate, "Food", &[]));
    t.resolve_all();
    // With an empty stack in P0's main phase: yes.
    assert!(activate_text(&mut t, P0, neonate, "Food", &[]));
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Food").len(), 1);
}
