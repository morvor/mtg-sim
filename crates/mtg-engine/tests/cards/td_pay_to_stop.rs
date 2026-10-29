//! Pay-to-stop delayed triggered abilities (CR 116.2c, 603.7): Glass Asp, Nafs Asp,
//! Sabertooth Cobra, Quenchable Fire.

use crate::combat2_common::*;
use crate::td_next_turn_durations::mana;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The special actions offered by effects that `p` could take now.
fn offers(t: &mut TestGame, p: PlayerId) -> Vec<Action> {
    t.g.recompute();
    mtg_engine::special_actions::available(&t.g, p)
        .into_iter()
        .filter(|a| matches!(a, Action::Special(SpecialAction::Offer { .. })))
        .collect()
}

/// `p`, holding priority, takes the first special action offered to them (with mana to pay
/// for it).
fn pay_offer(t: &mut TestGame, p: PlayerId) {
    mana(t, p);
    t.g.turn.priority = Some(p);
    let a = offers(t, p).into_iter().next().expect("an offer");
    t.g.take_action(p, a);
}

/// P0's Glass Asp deals combat damage to P1.
fn asp_hits(t: &mut TestGame, name: &str) {
    let asp = t.battlefield(P0, name);
    declare(t, P0, &[(asp, Entity::Player(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
}

#[test]
fn glass_asp_life_loss_waits_for_that_players_next_draw_step_and_happens_once() {
    cr!("603.7a", "603.7b", "116.2c");
    ruling!(
        "Glass Asp",
        "Once that player’s next draw step begins, the player no longer has the ability to perform this action"
    );
    supported("Glass Asp");
    let mut t = TestGame::new(2);
    asp_hits(&mut t, "Glass Asp");
    assert_eq!(t.life(P1), 18);
    // Only P1 may pay; P0 can't.
    assert!(offers(&mut t, P0).is_empty());
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.life(P1), 18, "not before the draw step");
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Too late to pay now.
    mana(&mut t, P1);
    assert!(offers(&mut t, P1).is_empty());
    // Once only.
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn glass_asp_paying_two_stops_the_life_loss() {
    cr!("116.2c");
    ruling!(
        "Glass Asp",
        "If the player performed the action, the delayed “lose 2 life” triggered ability won’t trigger"
    );
    let mut t = TestGame::new(2);
    asp_hits(&mut t, "Glass Asp");
    t.advance_to(P1, Step::Upkeep);
    mana(&mut t, P1);
    assert_eq!(offers(&mut t, P1).len(), 1);
    pay_offer(&mut t, P1);
    // Paid: {2} spent, and the action can't be taken again.
    assert!(offers(&mut t, P1).is_empty());
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn glass_asp_unpayable_means_the_life_is_lost() {
    cr!("116.2c");
    let mut t = TestGame::new(2);
    asp_hits(&mut t, "Glass Asp");
    t.advance_to(P1, Step::Upkeep);
    // No mana: nothing offered.
    assert!(offers(&mut t, P1).is_empty());
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn nafs_asp_two_hits_are_paid_for_separately() {
    cr!("603.7a", "116.2c");
    ruling!(
        "Nafs Asp",
        "If it damages a player twice, they get to pay or take damage twice"
    );
    supported("Nafs Asp");
    let mut t = TestGame::new(2);
    let asp = t.battlefield(P0, "Nafs Asp");
    // Two damage events (its damage dealt by an effect twice).
    for _ in 0..2 {
        t.g.deal_damage(asp, Entity::Player(P1), 1, false);
        t.resolve_all();
    }
    assert_eq!(t.life(P1), 18);
    t.advance_to(P1, Step::Upkeep);
    mana(&mut t, P1);
    assert_eq!(offers(&mut t, P1).len(), 2);
    pay_offer(&mut t, P1);
    assert_eq!(offers(&mut t, P1).len(), 1);
    t.advance_to(P1, Step::Draw);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn sabertooth_cobra_gives_another_poison_counter_at_their_next_upkeep() {
    cr!("603.7a");
    supported("Sabertooth Cobra");
    let mut t = TestGame::new(2);
    asp_hits(&mut t, "Sabertooth Cobra");
    let poison = |t: &TestGame| t.g.players[1].counters.get("poison").copied().unwrap_or(0);
    assert_eq!(poison(&t), 1);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(poison(&t), 2);
    // With the payment, no second counter.
    let mut t = TestGame::new(2);
    asp_hits(&mut t, "Sabertooth Cobra");
    // P1 pays during P0's end step.
    t.advance_to(P0, Step::End);
    pay_offer(&mut t, P1);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(poison(&t), 1);
}

#[test]
fn quenchable_fire_deals_three_more_at_your_next_upkeep_unless_quenched() {
    cr!("603.7a", "116.2c");
    ruling!(
        "Quenchable Fire",
        "If it triggers, the player can’t stop it by paying {U}"
    );
    supported("Quenchable Fire");
    let mut t = TestGame::new(2);
    let fire = t.hand(P0, "Quenchable Fire");
    mana(&mut t, P0);
    t.cast_with(P0, fire, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.advance_to(P0, Step::Upkeep);
    // It has triggered: P1 can't pay any more.
    mana(&mut t, P1);
    assert!(offers(&mut t, P1).is_empty());
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn quenchable_fire_on_a_planeswalker_its_controller_pays() {
    cr!("116.2c");
    let mut t = TestGame::new(2);
    let pw = t.battlefield(P1, "Gideon Jura");
    let fire = t.hand(P0, "Quenchable Fire");
    mana(&mut t, P0);
    t.cast_with(P0, fire, &[Entity::Object(pw)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(pw, "loyalty"), 3);
    mana(&mut t, P0);
    assert!(offers(&mut t, P0).is_empty(), "the caster can't pay");
    t.advance_to(P1, Step::Upkeep);
    pay_offer(&mut t, P1);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(pw, "loyalty"), 3);
}
