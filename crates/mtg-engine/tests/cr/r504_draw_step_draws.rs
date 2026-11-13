//! CR 504.1, 121.2: "except the first one they draw in each of their draw steps" — the
//! card a player draws in their draw step's turn-based action is that draw step's first
//! card; every other card they draw, in that step or at any other time, isn't. Draws are
//! counted one at a time, and only actual draws count (CR 121.1).

use crate::r703_common::supported;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to `p`'s draw step (its turn-based draw done), triggers on the stack.
fn to_draw_step(t: &mut TestGame, p: PlayerId) {
    t.advance_to(p, Step::Draw);
    t.settle();
}

/// The number of +1/+1 counters on `id`, after resolving everything.
fn counters_after_resolving(t: &mut TestGame, id: ObjectId) -> u32 {
    t.resolve_all();
    t.counters(id, "+1/+1")
}

#[test]
fn the_draw_step_draw_is_the_first_one_later_draws_in_that_step_arent() {
    cr!("504.1", "121.2", "603.2");
    supported("Leela, Sevateem Warrior");
    let mut t = TestGame::new(2);
    // "Whenever an opponent draws a card except the first one they draw in each of their
    // draw steps, put a +1/+1 counter on Leela."
    let leela = t.battlefield(P0, "Leela, Sevateem Warrior");
    to_draw_step(&mut t, P1);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(counters_after_resolving(&mut t, leela), 0);
    // Another card in the same draw step: not the first one.
    t.lands(P1, "Island", 1);
    let opt = t.hand(P1, "Opt");
    t.cast(P1, opt).go();
    t.resolve();
    assert_eq!(counters_after_resolving(&mut t, leela), 1);
}

#[test]
fn draws_outside_the_players_own_draw_step_arent_exempt() {
    cr!("504.1", "121.2");
    let mut t = TestGame::new(2);
    let leela = t.battlefield(P1, "Leela, Sevateem Warrior");
    // P0 draws during P1's draw step: it isn't one of P0's draw steps.
    to_draw_step(&mut t, P1);
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve();
    assert_eq!(counters_after_resolving(&mut t, leela), 1);
    // And in P0's main phase, P0's first draw of the turn isn't exempt either.
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let before = t.counters(leela, "+1/+1");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    t.cast(P0, opt).go();
    t.resolve();
    assert_eq!(counters_after_resolving(&mut t, leela), before + 1);
}

#[test]
fn each_card_of_a_multiple_draw_counts_separately() {
    cr!("121.2", "504.1");
    let mut t = TestGame::new(2);
    let leela = t.battlefield(P0, "Leela, Sevateem Warrior");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 3);
    let div = t.hand(P1, "Divination");
    t.cast(P1, div).go();
    t.resolve();
    assert_eq!(counters_after_resolving(&mut t, leela), 2);
}

#[test]
fn putting_a_card_into_a_hand_isnt_drawing_it() {
    cr!("121.1");
    let mut t = TestGame::new(2);
    let leela = t.battlefield(P0, "Leela, Sevateem Warrior");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 2);
    let impulse = t.hand(P1, "Impulse");
    t.cast(P1, impulse).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(counters_after_resolving(&mut t, leela), 0);
}

#[test]
fn a_replacement_effect_excepts_the_draw_step_draw() {
    cr!("504.1", "614.1a", "121.6");
    supported("Notion Thief");
    let mut t = TestGame::new(2);
    // "If an opponent would draw a card except the first one they draw in each of their
    // draw steps, instead that player skips that draw and you draw a card."
    t.battlefield(P0, "Notion Thief");
    to_draw_step(&mut t, P1);
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.hand_size(P0), 0);
    t.lands(P1, "Island", 1);
    let opt = t.hand(P1, "Opt");
    t.cast(P1, opt).go();
    t.resolve();
    // Opt's draw was P1's second card this draw step: P0 drew instead.
    assert_eq!(t.hand_size(P1), 1);
    assert_eq!(t.hand_size(P0), 1);
}
