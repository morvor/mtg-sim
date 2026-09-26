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
