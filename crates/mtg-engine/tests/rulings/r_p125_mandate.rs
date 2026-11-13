//! Rulings batch P125 — Mandate of Peace ("Cast this spell only during combat. Your
//! opponents can't cast spells this turn. End the combat phase."): a casting restriction
//! (CR 601.3), a prohibition that starts as it resolves (CR 101.2), and what it leaves
//! alone.

use crate::r_p125_common::*;
use crate::r_s02_common::can_activate;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's Mandate of Peace in hand with the mana for it.
fn mandate(t: &mut TestGame) -> ObjectId {
    supported("Mandate of Peace");
    lands_for_cost(t, P0, "Mandate of Peace");
    t.hand(P0, "Mandate of Peace")
}

#[test]
fn mandate_of_peace_only_in_combat_and_then_opponents_cant_cast_spells() {
    cr!("601.3", "101.2");
    ruling!(
        "Mandate of Peace",
        "If an effect allows or instructs you to cast Mandate of Peace outside of a combat phase, you can’t do so."
    );
    let mut t = TestGame::new(2);
    let card = mandate(&mut t);
    assert!(!castable(&mut t, P0, card), "not in a main phase");
    let giant = t.battlefield(P0, "Hill Giant");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[giant]));
    assert!(castable(&mut t, P0, card));
    t.cast(P0, card).go();
    t.resolve_all();
    // Combat ended; P1 can't cast spells this turn.
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    assert!(!castable(&mut t, P1, shock));
}

#[test]
fn mandate_of_peace_can_be_responded_to() {
    cr!("101.2", "117.7");
    ruling!(
        "Mandate of Peace",
        "Mandate of Peace doesn’t stop any player from casting spells in response to Mandate of Peace before it resolves."
    );
    let mut t = TestGame::new(2);
    let card = mandate(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[giant]));
    t.cast(P0, card).go();
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    assert!(castable(&mut t, P1, shock));
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn mandate_of_peace_doesnt_stop_abilities() {
    cr!("101.2", "602.1");
    ruling!(
        "Mandate of Peace",
        "Your opponents can still activate abilities, including abilities of cards in their hand (such as cycling abilities), and can still play lands."
    );
    let mut t = TestGame::new(2);
    let card = mandate(&mut t);
    let giant = t.battlefield(P0, "Hill Giant");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[giant]));
    t.cast(P0, card).go();
    t.resolve_all();
    assert!(can_activate(&mut t, P1, sorcerer));
    // A cycling card in P1's hand.
    let cycler = t.hand(P1, "Drifting Meadow");
    t.lands(P1, "Plains", 2);
    assert!(can_activate(&mut t, P1, cycler));
}
