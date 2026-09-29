//! Rulings batch S30 — the monarch (CR 725): its two inherent triggered abilities, a
//! draw that still happens after the monarch changes, and a monarch who loses to combat
//! damage.

use crate::r_s01_common::supported;
use crate::r_s30_common::*;
use mtg_engine::designations::become_monarch;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn being_the_monarch_draws_at_end_step_and_combat_damage_steals_it() {
    cr!("725.2", "725.3");
    ruling!(
        "Skyline Despot",
        "Being the monarch carries two inherent triggered abilities. “At the beginning of the monarch's end step, that player draws a card” and “Whenever a creature deals combat damage to the monarch, its controller becomes the monarch.”"
    );
    supported("Skyline Despot");
    let mut t = TestGame::new(2);
    // "When this creature enters, you become the monarch."
    t.enter(P0, "Skyline Despot");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    // At the beginning of the monarch's end step, that player draws a card.
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Not at the beginning of another player's end step.
    let hand1 = t.hand_size(P1);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand1 + 1, "only P1's draw step draw");
    // Whenever a creature deals combat damage to the monarch, its controller becomes the
    // monarch.
    t.advance_to(P1, Step::BeginningOfCombat);
    t.attack(&[(bear, Entity::Player(P0))], &[]);
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P1));
}

#[test]
fn the_monarch_who_triggered_the_draw_still_draws_after_losing_the_title() {
    cr!("725.2", "603.3");
    ruling!(
        "Throne of the High City",
        "If the triggered ability that causes the monarch to draw a card goes on the stack, and a different player becomes the monarch before that ability resolves, the first player will still draw the card."
    );
    supported("Throne of the High City");
    let mut t = TestGame::new(2);
    // "{4}, {T}, Sacrifice this land: You become the monarch."
    let throne = t.battlefield(P0, "Throne of the High City");
    t.lands(P0, "Wastes", 4);
    t.activate(P0, throne, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // In response, P1 becomes the monarch.
    become_monarch(&mut t.g, P1);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0 + 1, h1));
    assert_eq!(t.g.monarch, Some(P1));
}

#[test]
fn a_monarch_killed_by_combat_damage_doesnt_pass_it_through_the_trigger() {
    cr!("725.2", "725.4", "800.4a");
    ruling!(
        "Court of Grace",
        "If combat damage dealt to the monarch causes that player to lose the game, the triggered ability that causes the controller of the attacking creature to become the monarch doesn't resolve. In most cases, the controller of the attacking creature will still become the monarch as it is likely their turn."
    );
    supported("Court of Grace");
    // Three players: P1 is the monarch (Court of Grace) with 2 life.
    let mut t = TestGame::new(3);
    t.enter(P1, "Court of Grace");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P1));
    set_life(&mut t, P1, 2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    // The combat damage trigger was controlled by P1, who lost the game as state-based
    // actions were checked: it left the stack without resolving.
    assert!(t.has_lost(P1));
    assert!(t.g.stack.is_empty());
    // P0 became the monarch anyway: the monarch left the game during P0's turn.
    assert_eq!(t.g.monarch, Some(P0));
    // During P2's turn, P2 would have become the monarch instead.
    let mut t = TestGame::new(3);
    t.enter(P1, "Court of Grace");
    t.resolve_all();
    set_life(&mut t, P1, 2);
    let bear = t.battlefield(P2, "Grizzly Bears");
    t.advance_to(P2, Step::BeginningOfCombat);
    t.answer(
        P2,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bear, Entity::Player(P1))]),
    );
    t.advance_to(P2, Step::CombatDamage);
    t.settle();
    assert!(t.has_lost(P1));
    assert_eq!(t.g.monarch, Some(P2));
}
