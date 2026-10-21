//! Numbers players choose secretly for the same object, revealed together, and what's
//! said about them (`oracle/patterns/choice_grammar_secret_numbers.rs`): "the highest
//! number", "each player who didn't choose the lowest number", "the player who revealed
//! the fewest items", "if they match".

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

fn number(t: &mut TestGame, p: PlayerId, n: i64) {
    t.answer(p, DecisionKind::Number, Answer::Number(n));
}

#[test]
fn menacing_ogre_each_player_with_the_highest_number_loses_that_much_life() {
    cr!("608.2c");
    ruling!("Menacing Ogre", "You have to choose zero or a positive number");
    compiles("Menacing Ogre");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    number(&mut t, P0, 5);
    number(&mut t, P1, 5);
    number(&mut t, P2, 2);
    let ogre = t.enter(P0, "Menacing Ogre");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (15, 15, 20));
    assert_eq!(t.counters(ogre, "+1/+1"), 2, "you are one of those players");
    let asked = t.asked();
    let min = asked.iter().find_map(|(_, d)| match d {
        mtg_engine::decision::Decision::ChooseNumber { min, .. } => Some(*min),
        _ => None,
    });
    assert_eq!(min, Some(0), "zero or a positive number");
}

#[test]
fn menacing_ogre_no_counters_if_you_didnt_choose_the_highest_number() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    number(&mut t, P0, 1);
    number(&mut t, P1, 3);
    let ogre = t.enter(P0, "Menacing Ogre");
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
    assert_eq!(t.counters(ogre, "+1/+1"), 0);
    assert!(t.g.obj(ogre).choices.secret.is_empty(), "revealed");
}

#[test]
fn wheel_of_misfortune_highest_number_deals_damage_and_others_than_lowest_wheel() {
    cr!("608.2c", "608.2e");
    ruling!(
        "Wheel of Misfortune",
        "if the four players in your game reveal 4, 7, 7, and 7"
    );
    compiles("Wheel of Misfortune");
    let mut t = TestGame::new(4);
    t.lands(P0, "Mountain", 3);
    let wheel = t.hand(P0, "Wheel of Misfortune");
    t.hand(P0, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    t.advance_to(P0, Step::PrecombatMain);
    number(&mut t, P0, 4);
    number(&mut t, P1, 7);
    number(&mut t, P2, 7);
    number(&mut t, P3, 7);
    t.cast(P0, wheel).go();
    t.resolve_all();
    assert_eq!(
        (t.life(P0), t.life(P1), t.life(P2), t.life(P3)),
        (20, 13, 13, 13)
    );
    assert!(t.in_hand(P0, "Grizzly Bears"), "P0 chose the lowest number");
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "P1 discarded their hand");
    assert_eq!(t.hand_size(P1), 7);
    assert_eq!(t.hand_size(P2), 7);
}

#[test]
fn goblin_game_life_loss_by_items_then_the_fewest_loses_half() {
    cr!("107.1a", "608.2c");
    compiles("Goblin Game");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    let game = t.hand(P0, "Goblin Game");
    t.advance_to(P0, Step::PrecombatMain);
    number(&mut t, P0, 3);
    number(&mut t, P1, 1);
    t.cast(P0, game).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // 20 - 1 = 19, then half of 19 rounded up (10).
    assert_eq!(t.life(P1), 9);
}

#[test]
fn goblin_game_each_player_tied_for_fewest_loses_half() {
    cr!("107.1a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 7);
    let game = t.hand(P0, "Goblin Game");
    t.advance_to(P0, Step::PrecombatMain);
    number(&mut t, P0, 2);
    number(&mut t, P1, 2);
    t.cast(P0, game).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (9, 9));
    let min = t.asked().iter().find_map(|(_, d)| match d {
        mtg_engine::decision::Decision::ChooseNumber { min, .. } => Some(*min),
        _ => None,
    });
    assert_eq!(min, Some(1), "at least one item");
}

/// Expert-Level Safe with two cards exiled, activated by P0 against P1 choosing `a` and
/// `b`: (safe still on the battlefield, cards in P0's hand gained, cards exiled).
fn safe(a: i64, b: i64) -> (bool, usize, usize) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    let safe = t.enter(P0, "Expert-Level Safe");
    t.resolve_all();
    t.lands(P0, "Plains", 1);
    let hand = t.hand_size(P0);
    number(&mut t, P0, a);
    number(&mut t, P1, b);
    t.activate(P0, safe, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    let exiled = t
        .g
        .exile
        .iter()
        .filter(|id| t.g.obj(**id).owner == P0)
        .count();
    (t.on_battlefield(safe), t.hand_size(P0) - hand, exiled)
}

#[test]
fn expert_level_safe_opens_if_the_secret_numbers_match() {
    cr!("608.2c", "607.2a");
    compiles("Expert-Level Safe");
    assert_eq!(safe(2, 2), (false, 2, 0), "they match: the safe opens");
    assert_eq!(safe(1, 3), (true, 0, 3), "otherwise: one more card exiled");
}
