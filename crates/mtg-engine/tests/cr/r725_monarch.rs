//! CR 725: the monarch.

use mtg_engine::designations::become_monarch;
use mtg_engine::monarch_initiative::{is_inherent, MONARCH_DRAW, MONARCH_STEAL};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// The controller and source of the top object of the stack.
fn top(t: &TestGame) -> (PlayerId, ObjectId) {
    let id = *t.g.stack.last().expect("empty stack");
    let o = t.g.obj(id);
    let src = match o.stack.as_deref().map(|s| &s.kind) {
        Some(mtg_engine::object::StackKind::Triggered { source, .. }) => *source,
        _ => id,
    };
    (o.controller, src)
}

#[test]
fn there_is_no_monarch_until_an_effect_makes_a_player_the_monarch() {
    cr!("725.1");
    ruling!(
        "Palace Jailer",
        "The game starts with no monarch. Once an effect makes one player the monarch, the game will have exactly one monarch from that point forward."
    );
    let mut t = TestGame::new(2);
    assert_eq!(t.g.monarch, None);
    // No monarch: nobody draws at the beginning of the end step.
    t.advance_to(P0, Step::End);
    t.settle();
    assert!(t.g.stack.is_empty());
    // "When this creature enters, you become the monarch."
    t.enter(P0, "Palace Jailer");
    // Its second ability has no legal target; only the monarch trigger resolves.
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
}

#[test]
fn the_monarch_draws_a_card_at_the_beginning_of_their_end_step() {
    cr!("725.2");
    ruling!(
        "Palace Jailer",
        "Being the monarch carries two inherent triggered abilities. \"At the beginning of the monarch's end step, that player draws a card\" and \"Whenever a creature deals combat damage to the monarch, its controller becomes the monarch.\""
    );
    let mut t = TestGame::new(2);
    become_monarch(&mut t.g, P0);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.settle();
    // The ability has no source and is controlled by the monarch.
    assert_eq!(t.g.stack.len(), 1);
    let (controller, src) = top(&t);
    assert_eq!(controller, P0);
    assert!(is_inherent(&t.g, src, MONARCH_DRAW));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Not at the beginning of another player's end step.
    t.advance_to(P1, Step::End);
    t.settle();
    assert!(t.g.stack.is_empty());
}

#[test]
fn the_monarch_who_drew_is_the_player_who_was_the_monarch_when_it_triggered() {
    cr!("725.2");
    ruling!(
        "Fealty to the Realm",
        "If the triggered ability that causes the monarch to draw a card goes on the stack and a different player becomes the monarch before that ability resolves, the first player will still draw the card."
    );
    let mut t = TestGame::new(2);
    become_monarch(&mut t.g, P0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.g.stack.len(), 1);
    become_monarch(&mut t.g, P1);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0 + 1, h1));
}

#[test]
fn combat_damage_to_the_monarch_makes_the_creatures_controller_the_monarch() {
    cr!("725.2", "725.3");
    ruling!(
        "Fealty to the Realm",
        "There are two inherent triggered abilities associated with being the monarch. These triggered abilities have no source and are controlled by the player who was the monarch at the time the abilities triggered."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bear = t.battlefield(P1, "Grizzly Bears");
    become_monarch(&mut t.g, P0);
    // Noncombat damage doesn't make anyone the monarch.
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    t.answer(
        P1,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P0))]),
    );
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    // The ability is controlled by the player who was the monarch as it triggered.
    assert_eq!(t.g.stack.len(), 1);
    let (controller, src) = top(&t);
    assert_eq!(controller, P0);
    assert!(is_inherent(&t.g, src, MONARCH_STEAL));
    t.resolve_all();
    // Only one player is the monarch: P0 ceased to be the monarch.
    assert_eq!(t.g.monarch, Some(P1));
    // Now P1 draws at the beginning of their end step, and P0 no longer does.
    t.advance_to(P1, Step::End);
    t.settle();
    let (_, src) = top(&t);
    assert!(is_inherent(&t.g, src, MONARCH_DRAW));
    assert_eq!(top(&t).0, P1);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    assert!(t.g.stack.is_empty());
}

#[test]
fn becoming_the_monarch_ends_the_current_monarchs_reign() {
    cr!("725.3");
    ruling!(
        "Fealty to the Realm",
        "As a player becomes the monarch, the current monarch (if any) ceases being the monarch."
    );
    let mut t = TestGame::new(3);
    become_monarch(&mut t.g, P1);
    t.enter(P2, "Palace Jailer");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P2));
}

#[test]
fn if_the_monarch_leaves_the_game_the_active_player_becomes_the_monarch() {
    cr!("725.4");
    ruling!(
        "Palace Jailer",
        "In a multiplayer game, if the monarch leaves the game, the player whose turn it is immediately becomes the monarch."
    );
    ruling!(
        "Fealty to the Realm",
        "If the monarch leaves the game during another player's turn, that player becomes the monarch."
    );
    let mut t = TestGame::new(3);
    become_monarch(&mut t.g, P1);
    t.take_action(P1, Action::Concede);
    assert_eq!(t.g.monarch, Some(P0));
}

#[test]
fn if_the_monarch_leaves_during_their_turn_the_next_player_becomes_the_monarch() {
    cr!("725.4");
    ruling!(
        "Fealty to the Realm",
        "If the monarch leaves the game during their turn, the next player in turn order becomes the monarch."
    );
    let mut t = TestGame::new(4);
    t.set_step(P1, Step::PrecombatMain);
    become_monarch(&mut t.g, P1);
    // P2 already left the game: the next player in turn order who can become the monarch
    // is P3.
    t.take_action(P2, Action::Concede);
    t.take_action(P1, Action::Concede);
    assert_eq!(t.g.monarch, Some(P3));
}

#[test]
fn an_effect_determined_by_the_monarch_does_nothing_while_there_is_no_monarch() {
    cr!("725.5");
    ruling!(
        "Fealty to the Realm",
        "In the case where Fealty to the Realm is enchanting a creature but there is no monarch, the second ability will create a control-changing effect with the timestamp mentioned above, but that effect won't do anything until a player becomes the monarch."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    // Put onto the battlefield without its enters ability: there's no monarch.
    let fealty = t.battlefield(P0, "Fealty to the Realm");
    t.g.attach(fealty, Entity::Object(bear));
    t.g.recompute();
    assert_eq!(t.g.monarch, None);
    assert_eq!(t.g.obj(bear).controller, P1);
    // "The monarch controls enchanted creature."
    become_monarch(&mut t.g, P0);
    t.g.recompute();
    assert_eq!(t.g.obj(bear).controller, P0);
    become_monarch(&mut t.g, P1);
    t.g.recompute();
    assert_eq!(t.g.obj(bear).controller, P1);
}
