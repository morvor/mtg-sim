//! Rulings batch S12 — parley (an ability word): "Each player reveals the top card of
//! their library. For each nonland card revealed this way, [effect]. Then each player draws
//! a card."

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn each_player_draws_the_card_they_revealed() {
    cr!("701.20a", "701.20b", "121.1");
    ruling!(
        "Selvala's Charge",
        "Except in some very rare cases, the card each player draws will be the card revealed from the top of their library."
    );
    supported("Selvala's Charge");
    supported("Cloakwood Swarmkeeper");
    // Selvala's Charge: "For each nonland card revealed this way, you create a 3/3 green
    // Elephant creature token."
    let mut t = TestGame::new(3);
    let keeper = t.battlefield(P0, "Cloakwood Swarmkeeper");
    let charge = in_hand_with_mana(&mut t, P0, "Selvala's Charge");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let island = t.library_top(P1, "Island");
    let bears = t.library_top(P2, "Grizzly Bears");
    t.cast(P0, charge).go();
    t.resolve_all();
    // Two nonland cards: two Elephants, created at once (Cloakwood Swarmkeeper's "Whenever
    // one or more tokens you control enter" triggers once).
    let elephants = with_subtype(&t, P0, "Elephant");
    assert_eq!(elephants.len(), 2);
    assert_eq!(t.pt(elephants[0]), (3, 3));
    assert_eq!(t.counters(keeper, counters::PLUS1), 1);
    // Each player drew the card they revealed.
    assert_eq!(t.zone(bolt), mtg_engine::object::Zone::Hand(P0));
    assert_eq!(t.zone(island), mtg_engine::object::Zone::Hand(P1));
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Hand(P2));
}

#[test]
fn cutthroat_negotiators_parley_counts_nonland_cards_then_everyone_draws_them() {
    cr!("701.20a", "121.1", "508.1m");
    ruling!(
        "Cutthroat Negotiator",
        "Except in some very unusual cases, the card each player reveals is the one they’ll draw."
    );
    supported("Cutthroat Negotiator");
    // "Whenever this creature attacks, ... For each nonland card revealed this way, you
    // create a tapped Treasure token."
    let mut t = TestGame::new(2);
    let neg = t.battlefield(P0, "Cutthroat Negotiator");
    let giant = t.library_top(P0, "Hill Giant");
    let forest = t.library_top(P1, "Forest");
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    attack_with(&mut t, &[(neg, Entity::Player(P1))]);
    t.resolve_all();
    let treasures = with_subtype(&t, P0, "Treasure");
    assert_eq!(treasures.len(), 1);
    assert!(t.obj(treasures[0]).tapped);
    assert_eq!(t.zone(giant), mtg_engine::object::Zone::Hand(P0));
    assert_eq!(t.zone(forest), mtg_engine::object::Zone::Hand(P1));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0 + 1, h1 + 1));
    // Two lands: no Treasure.
    let mut t = TestGame::new(2);
    let neg = t.battlefield(P0, "Cutthroat Negotiator");
    t.library_top(P0, "Plains");
    t.library_top(P1, "Forest");
    attack_with(&mut t, &[(neg, Entity::Player(P1))]);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Treasure").is_empty());
}

#[test]
fn selvalas_enforcer_gets_a_counter_for_each_nonland_card_revealed() {
    cr!("701.20a", "122.1");
    supported("Selvala's Enforcer");
    // "When this creature enters, ... For each nonland card revealed this way, put a
    // +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    t.library_top(P0, "Lightning Bolt");
    t.library_top(P1, "Grizzly Bears");
    let enforcer = in_hand_with_mana(&mut t, P0, "Selvala's Enforcer");
    t.cast(P0, enforcer).go();
    t.resolve_all();
    assert_eq!(t.counters(enforcer, counters::PLUS1), 2);
    assert_eq!(t.pt(enforcer), (4, 4));
    assert!(t.in_hand(P0, "Lightning Bolt") && t.in_hand(P1, "Grizzly Bears"));
}
