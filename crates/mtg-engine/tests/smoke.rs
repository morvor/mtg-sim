//! End-to-end smoke tests for the core engine.

use mtg_engine::agents::RandomAgent;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn lightning_bolt_kills_grizzly_bears() {
    cr!("601.2", "608.2b", "704.5g");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bears).go();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn lightning_bolt_to_face() {
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn cant_cast_without_mana() {
    cr!("601.2h");
    let mut t = TestGame::new(2);
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    // The card is still in hand (the casting was reversed, CR 733).
    assert!(t.in_hand(P0, "Lightning Bolt"));
}

#[test]
fn unblocked_attacker_deals_damage() {
    cr!("510.1b", "506.2");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn blocked_creatures_trade() {
    cr!("510.1c", "510.1d", "704.5g");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1))], &[(b, a)]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn creature_spell_resolves_onto_battlefield() {
    cr!("608.3a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

fn simple_deck() -> Vec<std::sync::Arc<CardDef>> {
    let mut d = Vec::new();
    for _ in 0..24 {
        d.push(card("Mountain"));
    }
    for _ in 0..4 {
        for n in [
            "Grizzly Bears",
            "Lightning Bolt",
            "Hill Giant",
            "Shock",
            "Raging Goblin",
            "Gray Ogre",
            "Giant Growth",
            "Llanowar Elves",
            "Forest",
        ] {
            d.push(card(n));
        }
    }
    d
}

#[test]
fn random_games_complete() {
    for seed in 0..20u64 {
        let config = GameConfig {
            seed,
            max_turns: 60,
            ..Default::default()
        };
        let agents: Vec<Box<dyn Agent>> = vec![
            Box::new(RandomAgent::new(seed * 2 + 1)),
            Box::new(RandomAgent::new(seed * 2 + 2)),
        ];
        let mut g = Game::new(config, vec![simple_deck(), simple_deck()], agents);
        let r = g.run();
        let _ = r;
        assert!(g.result.is_some());
    }
}

/// A free repeatable ability ("{0}: Viscid Lemures gets -1/-0 and gains swampwalk until
/// end of turn") used to keep random agents activating it hundreds of times a step (each
/// time they got priority they acted with probability 0.8), each activation adding an
/// effect that slowed every later one: games took minutes. Now each action in a step
/// makes passing more likely.
#[test]
fn random_agents_pass_after_acting_repeatedly_in_a_step() {
    let mut t = TestGame::new(2);
    let lemures = t.battlefield(PlayerId(0), "Viscid Lemures");
    let activate = Action::Activate {
        source: lemures,
        ability: t.obj(lemures).chars.abilities[0].uid,
    };
    let mut agent = RandomAgent::new(7);
    let mut acted = 0;
    for _ in 0..200 {
        let d = Decision::Priority {
            actions: vec![Action::Pass, activate.clone()],
        };
        if agent.decide(&t.g, PlayerId(0), &d) != Answer::Action(Action::Pass) {
            acted += 1;
        }
    }
    // With a constant 0.8 it would act about 160 times.
    assert!(acted < 40, "acted {acted} times in one step");
}

#[test]
fn an_event_observer_sees_every_event_without_changing_the_game() {
    use mtg_engine::events::Event;
    use mtg_engine::game::EventObserver;
    use std::sync::{Arc, Mutex};
    let mut t = TestGame::new(2);
    let seen: Arc<Mutex<Vec<String>>> = Arc::default();
    let seen2 = seen.clone();
    t.g.observer = Some(EventObserver::new(move |g, ev| {
        let what = match ev {
            Event::SpellCast { spell, .. } => format!("cast {}", g.obj(*spell).chars.name),
            Event::Damage { amount, .. } => format!("damage {amount}"),
            Event::SpellResolved { .. } => "resolved".to_string(),
            _ => return,
        };
        seen2.lock().unwrap().push(what);
    }));
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(
        *seen.lock().unwrap(),
        ["cast Lightning Bolt", "damage 3", "resolved"]
    );
}

#[test]
fn an_event_observer_is_told_when_an_illegal_action_is_reversed() {
    cr!("733.1");
    use mtg_engine::game::EventObserver;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    let mut t = TestGame::new(2);
    let rollbacks = Arc::new(AtomicU32::new(0));
    let r2 = rollbacks.clone();
    t.g.observer = Some(EventObserver::new(|_, _| {}).with_rollback(move |g| {
        // The game is back as it was: the spell is in its owner's hand.
        assert!(g.stack.is_empty());
        r2.fetch_add(1, Ordering::Relaxed);
    }));
    let bolt = t.hand(P0, "Lightning Bolt");
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    assert_eq!(rollbacks.load(Ordering::Relaxed), 1);
    assert!(t.in_hand(P0, "Lightning Bolt"));
}
