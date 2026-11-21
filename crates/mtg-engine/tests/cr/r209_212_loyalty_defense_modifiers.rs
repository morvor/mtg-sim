//! CR 209–212: loyalty, loyalty abilities, defense, and the hand and life modifiers of
//! vanguard cards.

use crate::r100_common::{fillers, pregame};
use crate::r105_util::matches;
use crate::r300_common::hold_stack;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn printed_loyalty_is_the_loyalty_off_the_battlefield_and_the_counters_it_enters_with() {
    cr!("209.1");
    let mut t = TestGame::new(2);
    // Jace Beleren has a printed loyalty of 3.
    let jace = t.hand(P0, "Jace Beleren");
    assert_eq!(t.obj_now(jace).chars.loyalty, Some(3));
    assert_eq!(t.obj_now(jace).loyalty(), 3);
    let f = Filter::Loyalty(Cmp::Eq, Box::new(Value::c(3)));
    assert!(matches(&t, jace, &f, P0));
    let gy = t.graveyard(P0, "Jace Beleren");
    assert_eq!(t.obj_now(gy).loyalty(), 3);
    // It enters with three loyalty counters.
    let jace = t.enter(P0, "Jace Beleren");
    assert_eq!(t.counters(jace, "loyalty"), 3);
    assert_eq!(t.obj_now(jace).loyalty(), 3);
    // On the battlefield, its loyalty is the number of loyalty counters on it.
    t.g.add_counters(Entity::Object(jace), "loyalty", 2, None);
    assert_eq!(t.obj_now(jace).loyalty(), 5);
}

#[test]
fn loyalty_abilities_are_activated_at_sorcery_speed_once_each_turn() {
    cr!("209.2");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P0, "Jace Beleren");
    t.g.objects[jace.0 as usize]
        .counters
        .insert("loyalty".into(), 3);
    t.g.recompute();
    // Not while the stack isn't empty.
    hold_stack(&mut t, P0);
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    t.resolve_all();
    // Not during an opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    t.g.turn.priority = Some(P0);
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    // Not outside a main phase.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    // In a main phase of its controller's turn with an empty stack: yes, but only one
    // loyalty ability of that permanent each turn.
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.activate(P0, jace, 0, &[]).unwrap(); // +2: Each player draws a card.
    assert_eq!(t.counters(jace, "loyalty"), 5);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    assert!(t.activate(P0, jace, 1, &[Entity::Player(P0)]).is_err());
    assert_eq!(t.counters(jace, "loyalty"), 5);
}

#[test]
fn printed_defense_is_the_defense_off_the_battlefield_and_the_counters_it_enters_with() {
    cr!("210.1");
    let mut t = TestGame::new(2);
    // Invasion of Segovia has a printed defense of 4.
    let inv = t.hand(P0, "Invasion of Segovia");
    assert_eq!(t.obj_now(inv).chars.defense, Some(4));
    assert_eq!(t.obj_now(inv).defense(), 4);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let inv = t.enter(P0, "Invasion of Segovia");
    assert_eq!(t.counters(inv, "defense"), 4);
    assert_eq!(t.obj_now(inv).defense(), 4);
}

/// A Vanguard game (not yet started) where P0's deck includes the vanguard card.
fn vanguard_pregame(vanguard: &str) -> TestGame {
    let mut deck = fillers(40);
    deck.push(card(vanguard));
    pregame(
        GameConfig {
            variant: Variant::Vanguard,
            skip_mulligans: true,
            ..Default::default()
        },
        vec![deck, fillers(40)],
    )
}

#[test]
fn the_hand_modifier_changes_starting_and_maximum_hand_size() {
    cr!("211.1");
    // A number preceded by a minus sign: Orcish Squatters Avatar (-1).
    let squatters = card("Orcish Squatters Avatar");
    assert_eq!(squatters.front().chars.hand_modifier, Some(-1));
    let mut t = vanguard_pregame("Orcish Squatters Avatar");
    assert_eq!(t.g.starting_hand_size(P0), 6);
    t.g.start();
    assert_eq!(t.hand_size(P0), 6);
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.player(P0).max_hand_size, Some(6));
    // A plus sign: Dakkon Blackblade Avatar (+1).
    let mut t = vanguard_pregame("Dakkon Blackblade Avatar");
    t.g.start();
    assert_eq!(t.hand_size(P0), 8);
    assert_eq!(t.player(P0).max_hand_size, Some(8));
    // Zero: Stuffy Doll Avatar (+0).
    assert_eq!(
        card("Stuffy Doll Avatar").front().chars.hand_modifier,
        Some(0)
    );
    let mut t = vanguard_pregame("Stuffy Doll Avatar");
    t.g.start();
    assert_eq!(t.hand_size(P0), 7);
}

#[test]
fn the_life_modifier_changes_the_starting_life_total() {
    cr!("212.1");
    // Minus: Orcish Squatters Avatar (-1).
    let mut t = vanguard_pregame("Orcish Squatters Avatar");
    assert_eq!(t.g.starting_life(P0), 19);
    t.g.start();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 20);
    // Plus: Prodigal Sorcerer Avatar (+5).
    let mut t = vanguard_pregame("Prodigal Sorcerer Avatar");
    t.g.start();
    assert_eq!(t.life(P0), 25);
    // Zero: Dakkon Blackblade Avatar (+0).
    assert_eq!(
        card("Dakkon Blackblade Avatar").front().chars.life_modifier,
        Some(0)
    );
    let mut t = vanguard_pregame("Dakkon Blackblade Avatar");
    t.g.start();
    assert_eq!(t.life(P0), 20);
}
