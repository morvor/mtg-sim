//! CR 119: life.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::Event;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn config(variant: Variant, teams: Option<Vec<u8>>) -> GameConfig {
    GameConfig {
        variant,
        teams,
        ..Default::default()
    }
}

fn life_gained_events(t: &TestGame, p: PlayerId) -> Vec<u32> {
    events_matching(
        t,
        |e| matches!(e, Event::LifeGained { player, .. } if *player == p),
    )
    .into_iter()
    .map(|e| match e {
        Event::LifeGained { amount, .. } => amount,
        _ => 0,
    })
    .collect()
}

fn life_lost_events(t: &TestGame, p: PlayerId) -> Vec<u32> {
    events_matching(
        t,
        |e| matches!(e, Event::LifeLost { player, .. } if *player == p),
    )
    .into_iter()
    .map(|e| match e {
        Event::LifeLost { amount, .. } => amount,
        _ => 0,
    })
    .collect()
}

/// "[cost]: Draw a card."
fn pay_life_drawer(n: i32) -> CardDef {
    CB::new("Life Tap")
        .artifact()
        .ability(act(
            Cost::free().with(CostPart::PayLife(Value::c(n))),
            Body::effect(draw(1)),
        ))
        .build()
}

fn life_cant_change() -> CardDef {
    compile_def(
        "Steady Heart",
        "Enchantment",
        "{2}",
        "Your life total can't change.",
    )
}

#[test]
fn players_start_with_20_life() {
    cr!("119.1");
    let t = TestGame::new(2);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
    let g = Game::new(GameConfig::default(), vec![vec![], vec![], vec![]], vec![]);
    assert!(g.players.iter().all(|p| p.life == 20));
}

#[test]
fn two_headed_giant_teams_start_with_30_life() {
    cr!("119.1a");
    let t = TestGame::with_config(4, config(Variant::TwoHeadedGiant, Some(vec![0, 0, 1, 1])));
    for p in [P0, P1, P2, P3] {
        assert_eq!(t.life(p), 30);
    }
    // The team's life total is shared: damage to one player is applied to it (CR 810.9).
    let mut t = t;
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P2).go();
    t.resolve();
    assert_eq!(t.life(P2), 27);
    assert_eq!(t.life(P3), 27);
    assert_eq!(t.life(P0), 30);
    // Three-Headed Giant: 15 more for the third member (CR 810.11).
    let t3 = TestGame::with_config(
        6,
        config(Variant::TwoHeadedGiant, Some(vec![0, 0, 0, 1, 1, 1])),
    );
    assert_eq!(t3.life(P0), 45);
}

#[test]
fn vanguard_starting_life_is_modified_by_the_life_modifier() {
    cr!("119.1b");
    let mut t = TestGame::with_config(2, config(Variant::Vanguard, None));
    t.command(P0, "Orim");
    t.command(P1, "Titania");
    mtg_engine::life_totals::set_starting_life_totals(&mut t.g);
    assert_eq!(t.life(P0), 32);
    assert_eq!(t.life(P1), 15);
}

#[test]
fn commander_players_start_with_40_life() {
    cr!("119.1c");
    let t = TestGame::with_config(4, config(Variant::Commander, None));
    assert!([P0, P1, P2, P3].iter().all(|p| t.life(*p) == 40));
}

#[test]
fn brawl_players_start_with_25_or_30_life() {
    cr!("119.1d");
    let brawl = |n: usize| {
        TestGame::with_config(
            n,
            GameConfig {
                variant: Variant::Commander,
                brawl: true,
                ..Default::default()
            },
        )
    };
    let two = brawl(2);
    assert_eq!((two.life(P0), two.life(P1)), (25, 25));
    let four = brawl(4);
    assert!([P0, P1, P2, P3].iter().all(|p| four.life(*p) == 30));
}

#[test]
fn the_archenemy_starts_with_40_life() {
    cr!("119.1e");
    let t = TestGame::with_config(4, config(Variant::Archenemy, Some(vec![0, 1, 1, 1])));
    assert_eq!(t.life(P0), 40);
    assert!([P1, P2, P3].iter().all(|p| t.life(*p) == 20));
}

#[test]
fn damage_to_a_player_causes_life_loss() {
    cr!("119.2");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!(life_lost_events(&t, P1), vec![3]);
}

#[test]
fn gaining_and_losing_life_adjusts_the_life_total() {
    cr!("119.3");
    let mut t = TestGame::new(2);
    let spell = CB::new("Drain")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::seq(vec![
            gain(4),
            Effect::LoseLife {
                who: PlayerRef::EachOpponent,
                n: Value::c(3),
            },
        ])))
        .build();
    let s = t.custom(P0, spell, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.life(P1), 17);
    assert_eq!(life_gained_events(&t, P0), vec![4]);
    assert_eq!(life_lost_events(&t, P1), vec![3]);
}

#[test]
fn paying_life_needs_enough_life_and_is_a_loss_of_life() {
    cr!("119.4");
    let mut t = TestGame::new(2);
    let tap = t.custom(P0, pay_life_drawer(3), Zone::Battlefield);
    t.g.players[0].life = 2;
    assert!(t.activate(P0, tap, 0, &[]).is_err());
    t.g.players[0].life = 3;
    t.activate(P0, tap, 0, &[]).unwrap();
    assert_eq!(t.life(P0), 0);
    // The payment is life loss.
    assert_eq!(life_lost_events(&t, P0), vec![3]);
}

#[test]
fn two_headed_giant_life_payments_use_the_team_life_total() {
    cr!("119.4a");
    let mut t = TestGame::with_config(4, config(Variant::TwoHeadedGiant, Some(vec![0, 0, 1, 1])));
    let tap = t.custom(P0, pay_life_drawer(8), Zone::Battlefield);
    // The team's life total is 30: P0 may pay 8, and the team's total is reduced.
    t.activate(P0, tap, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 22);
    // Once the team has 7 life, 8 life can't be paid.
    let lava = CB::new("Singe")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::LoseLife {
            who: PlayerRef::Player(P1),
            n: Value::c(15),
        }))
        .build();
    let s = t.custom(P0, lava, Zone::Hand(P0));
    t.cast(P0, s).try_go().unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 7);
    let tap2 = t.custom(P0, pay_life_drawer(8), Zone::Battlefield);
    assert!(t.activate(P0, tap2, 0, &[]).is_err());
}

#[test]
fn zero_life_can_always_be_paid() {
    cr!("119.4b");
    let mut t = TestGame::new(2);
    t.custom(P0, life_cant_change(), Zone::Battlefield);
    let zero = t.custom(P0, pay_life_drawer(0), Zone::Battlefield);
    let two = t.custom(P0, pay_life_drawer(2), Zone::Battlefield);
    // Even though P0 can't pay life, paying 0 life is possible.
    t.activate(P0, zero, 0, &[]).unwrap();
    t.resolve();
    assert!(t.activate(P0, two, 0, &[]).is_err());
    // Toxic Deluge's additional cost "pay X life" with X = 0 can be paid, X = 1 can't.
    t.lands(P0, "Swamp", 6);
    let deluge = t.hand(P0, "Toxic Deluge");
    t.cast(P0, deluge).x(0).try_go().unwrap();
    t.resolve();
    let deluge2 = t.hand(P0, "Toxic Deluge");
    assert!(t.cast(P0, deluge2).x(1).try_go().is_err());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn setting_a_life_total_gains_or_loses_the_difference() {
    cr!("119.5");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ajani's Pridemate");
    t.lands(P0, "Plains", 9);
    t.g.players[0].life = 5;
    let wind = t.hand(P0, "Blessed Wind");
    t.cast(P0, wind).target(P0).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    // The player gained 15 life, so "whenever you gain life" triggers.
    assert_eq!(life_gained_events(&t, P0), vec![15]);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    // Setting a higher life total lower is a loss of life.
    t.g.players[1].life = 26;
    let wind2 = t.hand(P0, "Blessed Wind");
    t.lands(P0, "Plains", 9);
    t.cast(P0, wind2).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
    assert_eq!(life_lost_events(&t, P1), vec![6]);
}

#[test]
fn a_player_with_0_or_less_life_loses() {
    cr!("119.6");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    t.g.players[1].life = 5;
    let axe = t.hand(P0, "Lava Axe");
    t.cast(P0, axe).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 0);
    assert!(t.has_lost(P1));
}

#[test]
fn a_player_who_cant_gain_life_cant_raise_their_life_total() {
    cr!("119.7");
    let mut t = TestGame::new(2);
    let no_gain = CB::new("Dry Well")
        .enchantment()
        .ability(stat(StaticEffect::Restriction(Restriction::CantGainLife(
            PlayerFilter::You,
        ))))
        .build();
    t.custom(P0, no_gain, Zone::Battlefield);
    // Replacement effects for P0's life gain don't do anything: this one would draw a
    // card instead of gaining life.
    let replace = CB::new("Scholar's Cup")
        .artifact()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::GainLife(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(draw(1))),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P0, replace, Zone::Battlefield);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    let helix = t.hand(P0, "Lightning Helix");
    t.cast(P0, helix).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), 0);
    assert!(life_gained_events(&t, P0).is_empty());
    // An exchange that would raise P0's life total doesn't happen.
    let conduit = t.battlefield(P0, "Soul Conduit");
    t.lands(P0, "Island", 6);
    t.g.players[0].life = 5;
    t.activate(P0, conduit, 0, &[Entity::Player(P0), Entity::Player(P1)])
        .unwrap();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (5, 17));
}

#[test]
fn a_player_who_cant_lose_life_cant_lower_it_or_pay_life() {
    cr!("119.8");
    let mut t = TestGame::new(2);
    t.custom(P1, life_cant_change(), Zone::Battlefield);
    let conduit = t.battlefield(P0, "Soul Conduit");
    t.lands(P0, "Island", 6);
    t.g.players[0].life = 5;
    // P1 (at 20) can't make an exchange that would lower their life total.
    t.activate(P0, conduit, 0, &[Entity::Player(P0), Entity::Player(P1)])
        .unwrap();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (5, 20));
    // P1 can't pay life.
    let tap = t.custom(P1, pay_life_drawer(1), Zone::Battlefield);
    t.set_step(P1, Step::PrecombatMain);
    assert!(t.activate(P1, tap, 0, &[]).is_err());
    // Nor pay 2 life for a Phyrexian mana symbol, while P0 can.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let growth = t.hand(P1, "Mutagenic Growth");
    assert!(t.cast(P1, growth).target(bears).try_go().is_err());
    let growth0 = t.hand(P0, "Mutagenic Growth");
    t.cast(P0, growth0).target(bears).go();
    assert_eq!(t.life(P0), 3);
}

#[test]
fn exchanging_life_totals() {
    cr!("119.5");
    let mut t = TestGame::new(2);
    let conduit = t.battlefield(P0, "Soul Conduit");
    t.lands(P0, "Island", 6);
    t.g.players[0].life = 5;
    t.activate(P0, conduit, 0, &[Entity::Player(P0), Entity::Player(P1)])
        .unwrap();
    t.resolve();
    // Each player gained or lost the life needed to end up with the other's total.
    assert_eq!((t.life(P0), t.life(P1)), (20, 5));
    assert_eq!(life_gained_events(&t, P0), vec![15]);
    assert_eq!(life_lost_events(&t, P1), vec![15]);
}

#[test]
fn gaining_zero_life_is_not_a_life_gain_event_and_each_source_is_separate() {
    cr!("119.9");
    let mut t = TestGame::new(2);
    let mate = t.battlefield(P0, "Ajani's Pridemate");
    let zero = CB::new("Nothing Gained")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(gain(0)))
        .build();
    let s = t.custom(P0, zero, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    // No life gain event: the ability didn't trigger.
    assert!(life_gained_events(&t, P0).is_empty());
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(mate, counters::PLUS1), 0);
    // Two lifelink sources dealing combat damage at once each cause a life gain event.
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    assert_eq!(life_gained_events(&t, P0), vec![2, 2]);
    assert_eq!(t.counters(mate, counters::PLUS1), 2);
}

#[test]
fn life_gain_replacement_effects_dont_apply_to_gaining_zero_life() {
    cr!("119.10");
    let mut t = TestGame::new(2);
    let replace = CB::new("Scholar's Cup")
        .artifact()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::GainLife(PlayerFilter::You),
            action: ReplacementAction::Instead(Box::new(draw(1))),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P0, replace, Zone::Battlefield);
    let zero = CB::new("Nothing Gained")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(gain(0)))
        .build();
    let s = t.custom(P0, zero, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    // Each source's life gain is a separate event the effect replaces.
    let a = t.battlefield(P0, "Vampire Nighthawk");
    let b = t.battlefield(P0, "Vampire Nighthawk");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), 2);
}
