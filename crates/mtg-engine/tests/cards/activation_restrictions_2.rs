//! Activation restrictions (CR 602.2, 602.5b, 113.6): who may activate ("Only the
//! controller of the enchanted creature may activate this ability", with a cost that pays
//! "enchanted creature's mana cost"), "Activate only if you've controlled ~ continuously
//! since the beginning of your most recent turn", "Activate only if ~ is on the battlefield
//! or in your graveyard", and "only if you're attacking two or more opponents".

use crate::basic_effects_common::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether `p` may activate the `index`th activated ability of `src` now.
fn can(t: &mut TestGame, p: PlayerId, src: ObjectId, index: usize) -> bool {
    t.g.recompute();
    let uid = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .nth(index)
        .map(|a| a.uid)
        .expect("no such ability");
    let saved = t.g.turn.priority;
    t.g.turn.priority = Some(p);
    let ok = t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Activate { source, ability, .. } if *source == src && *ability == uid)
    });
    t.g.turn.priority = saved;
    ok
}

#[test]
fn merseine_only_the_enchanted_creatures_controller_pays_its_mana_cost() {
    cr!("602.2", "118.3");
    ruling!("Merseine", "includes colored mana requirements");
    ruling!("Merseine", "Can remove as many counters per turn as you want");
    assert_supported("Merseine");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let mers = t.enter(P0, "Merseine");
    t.g.attach(mers, Entity::Object(giant));
    assert_eq!(t.counters(mers, "net"), 3);
    t.lands(P0, "Mountain", 8);
    // Merseine's controller can't, though they have the mana.
    assert!(!can(&mut t, P0, mers, 0));
    // The Giant's controller needs {3}{R}: Islands don't do.
    t.lands(P1, "Island", 4);
    assert!(!can(&mut t, P1, mers, 0));
    t.lands(P1, "Mountain", 4);
    assert!(can(&mut t, P1, mers, 0));
    t.activate(P1, mers, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(mers, "net"), 2);
    t.activate(P1, mers, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(mers, "net"), 1);
}

#[test]
fn rocket_launcher_needs_control_since_your_most_recent_turn() {
    cr!("602.5b", "302.6");
    ruling!("Rocket Launcher", "you can activate the ability many times in one turn");
    ruling!("Rocket Launcher", "It is only destroyed at end of turn if the ability resolved");
    assert_supported("Rocket Launcher");
    // Came under P0's control this turn: no.
    let mut t = TestGame::new(2);
    let rl = t.battlefield_sick(P0, "Rocket Launcher");
    t.lands(P0, "Mountain", 4);
    assert!(!can(&mut t, P0, rl, 0));
    // Controlled since the turn began: twice in a turn, then destroyed at the end step.
    let mut t = TestGame::new(2);
    let rl = t.battlefield(P0, "Rocket Launcher");
    t.lands(P0, "Mountain", 4);
    t.activate(P0, rl, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    t.activate(P0, rl, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(rl));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(rl));
    // During the opponent's turn, still controlled since P0's most recent turn began.
    let mut t = TestGame::new(2);
    let rl = t.battlefield(P0, "Rocket Launcher");
    t.lands(P0, "Mountain", 2);
    t.set_step(P1, Step::PrecombatMain);
    assert!(can(&mut t, P0, rl, 0));
}

#[test]
fn skyblades_boon_returns_from_the_battlefield_or_your_graveyard() {
    cr!("113.6", "602.5b");
    assert_supported("Skyblade's Boon");
    // From the graveyard.
    let mut t = TestGame::new(2);
    let boon = t.graveyard(P0, "Skyblade's Boon");
    t.lands(P0, "Plains", 3);
    t.activate(P0, boon, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Skyblade's Boon"));
    // Not from the hand.
    let boon = t.g.player(P0).hand[t.hand_size(P0) - 1];
    t.lands(P0, "Plains", 3);
    assert!(!can(&mut t, P0, boon, 0));
    // From the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let boon = t.enter(P0, "Skyblade's Boon");
    t.g.attach(boon, Entity::Object(bears));
    t.lands(P0, "Plains", 3);
    t.activate(P0, boon, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_hand(P0, "Skyblade's Boon"));
}

#[test]
fn nemesis_phoenix_returns_attacking_when_you_attack_two_opponents() {
    cr!("602.5b", "508.1");
    assert_supported("Nemesis Phoenix");
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let phoenix = t.graveyard(P0, "Nemesis Phoenix");
    t.lands(P0, "Mountain", 3);
    // Not outside the declare attackers step.
    assert!(!can(&mut t, P0, phoenix, 0));
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P1)), (b, Entity::Player(P2))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(can(&mut t, P0, phoenix, 0));
    t.activate(P0, phoenix, 0, &[]).unwrap();
    t.resolve();
    let p = t.named_on_battlefield("Nemesis Phoenix")[0];
    assert!(t.g.obj(p).tapped);
    assert!(t.g.combat.as_ref().is_some_and(|c| c.attackers.iter().any(|x| x.id == p)));
    // Attacking only one opponent: no.
    let mut t = TestGame::new(3);
    let a = t.battlefield(P0, "Grizzly Bears");
    let phoenix = t.graveyard(P0, "Nemesis Phoenix");
    t.lands(P0, "Mountain", 3);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(!can(&mut t, P0, phoenix, 0));
}
