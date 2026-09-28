//! Rulings batch S23 — "When a spell or ability an opponent controls causes you to discard
//! this card, ..." (Guerrilla Tactics, Psychic Purge): only a discard an opponent's spell
//! or ability's effect causes triggers it (CR 701.9a, 113.6k), not discarding the card to
//! pay a cost (CR 601.2h) or because of your own effect.

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// In P1's main phase, P1 casts Mind Rot targeting P0 (who discards two cards) and it
/// resolves; the triggered abilities it causes go on the stack.
fn mind_rot_p0(t: &mut TestGame) {
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let rot = t.hand(P1, "Mind Rot");
    t.cast(P1, rot).target(Entity::Player(P0)).go();
    t.resolve();
}

#[test]
fn guerrilla_tactics_triggers_on_an_opponents_discard_effect_not_on_a_cost() {
    cr!("701.9a", "113.6k", "601.2h", "603.2");
    ruling!(
        "Guerrilla Tactics",
        "Discarding as a cost to cast a spell will not trigger the ability. Only discarding as an effect will trigger the ability."
    );
    supported("Guerrilla Tactics");
    supported("Psychic Purge");
    supported("Mind Rot");
    supported("Tormenting Voice");
    // Guerrilla Tactics: "When a spell or ability an opponent controls causes you to
    // discard this card, it deals 4 damage to any target." Psychic Purge: "... that
    // player loses 5 life."
    // An opponent's Mind Rot: both trigger from the graveyard.
    let mut t = TestGame::new(2);
    t.hand(P0, "Guerrilla Tactics");
    t.hand(P0, "Psychic Purge");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    mind_rot_p0(&mut t);
    assert!(t.in_graveyard(P0, "Guerrilla Tactics"));
    assert!(t.in_graveyard(P0, "Psychic Purge"));
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 4 - 5);
    assert_eq!(t.life(P0), 20);
    // Discarded to pay Tormenting Voice's additional cost: no trigger.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let tactics = t.hand(P0, "Guerrilla Tactics");
    let voice = t.hand(P0, "Tormenting Voice");
    t.answer_choose(P0, &[Entity::Object(tactics)]);
    t.cast(P0, voice).go();
    assert!(t.in_graveyard(P0, "Guerrilla Tactics"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Discarded because of its owner's own spell (P0's Mind Rot targeting P0): no
    // trigger either.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    t.hand(P0, "Guerrilla Tactics");
    t.hand(P0, "Psychic Purge");
    let rot = t.hand(P0, "Mind Rot");
    t.cast(P0, rot).target(Entity::Player(P0)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Guerrilla Tactics"));
    assert!(t.in_graveyard(P0, "Psychic Purge"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn guerrilla_tactics_triggers_on_an_optional_discard_an_opponent_causes() {
    cr!("701.9a", "702.21a", "118.12");
    ruling!(
        "Guerrilla Tactics",
        "The second ability will trigger even on an optional discard caused by an opponent."
    );
    supported("Guerrilla Tactics");
    supported("Westgate Regent");
    // P1's Westgate Regent has "Ward—Discard a card." P0 targets it with Lightning Bolt
    // and chooses to discard Guerrilla Tactics to pay the ward cost.
    let mut t = TestGame::new(2);
    let regent = t.battlefield(P1, "Westgate Regent");
    t.lands(P0, "Mountain", 1);
    let tactics = t.hand(P0, "Guerrilla Tactics");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(regent).go();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(tactics)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    // The ward trigger resolves; Guerrilla Tactics's trigger goes on the stack above the
    // bolt, which isn't countered (the ward cost was paid).
    t.resolve();
    assert!(t.in_graveyard(P0, "Guerrilla Tactics"));
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // The bolt resolved: the 4/4 Regent has 3 damage marked on it.
    assert_eq!(t.obj_now(regent).damage, 3);
}
