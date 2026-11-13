//! CR 603.4 / 603.5 / 603.8 audit: intervening "if" clauses (checked as the event occurs
//! and again on resolution, where a false condition removes the ability — its targets
//! still being legal and its "may" not being asked), optional and "unless" abilities
//! (on the stack whatever their controller intends; the choice is made on resolution,
//! even after the source has left), and state triggers (once until the ability leaves the
//! stack; no recheck of the state on resolution).

use crate::r603_audit_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 controls Revel in Riches and `n` Treasures (from P1's creatures dying), and the
/// game is at the start of P0's next upkeep (triggers not yet put on the stack).
fn revel_with_treasures(n: usize) -> TestGame {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Revel in Riches");
    for _ in 0..n {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.lands(P0, "Plains", 4);
    let w = t.hand(P0, "Wrath of God");
    t.cast(P0, w).go();
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Treasure"), n);
    t.advance_to(P0, Step::Upkeep);
    t
}

#[test]
fn an_intervening_if_false_on_resolution_removes_the_ability() {
    cr!("603.4");
    ruling!(
        "Revel in Riches",
        "If you don't control ten Treasures as the second ability of Revel in Riches resolves, you won't win the game."
    );
    let mut t = revel_with_treasures(10);
    t.settle();
    assert_eq!(triggers_of(&t, "Revel in Riches"), 1);
    // In response, P0 sacrifices a Treasure for mana.
    let treasure = t.named_on_battlefield("Treasure Token")[0];
    t.activate(P0, treasure, 0, &[]).expect("Treasure's mana ability");
    assert_eq!(tokens_named(&t, P0, "Treasure"), 9);
    t.resolve();
    assert!(!t.has_lost(P1));
    assert!(t.g.result.is_none());
    // Without the response, P0 wins.
    let mut t = revel_with_treasures(10);
    t.resolve_all();
    assert!(t.has_lost(P1));
}

#[test]
fn an_intervening_if_false_as_the_event_occurs_doesnt_trigger() {
    cr!("603.4");
    ruling!(
        "Revel in Riches",
        "If you don't control ten Treasures as your upkeep begins, the second ability of Revel in Riches won't trigger."
    );
    let mut t = revel_with_treasures(9);
    t.settle();
    assert_eq!(triggers_of(&t, "Revel in Riches"), 0);
}

#[test]
fn an_intervening_if_false_on_resolution_doesnt_ask_the_may() {
    cr!("603.4", "603.5");
    supported("Pyre Zombie");
    let mut t = TestGame::new(2);
    let zombie = t.graveyard(P0, "Pyre Zombie");
    t.lands(P0, "Swamp", 3);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_of(&t, "Pyre Zombie"), 1);
    // In response the card leaves the graveyard.
    t.g.exile_object(t.g.current(zombie), None);
    let asked_before = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    let yes_no = t.asked()[asked_before..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(yes_no, 0, "the ability was removed before its \"may\"");
    assert!(t.in_exile("Pyre Zombie"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn an_intervening_if_false_on_resolution_removes_it_though_its_target_is_still_legal() {
    cr!("603.4", "608.2b");
    supported("Oversold Cemetery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oversold Cemetery");
    let first = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Craw Wurm");
    let last = t.graveyard(P0, "Soul Warden");
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::Upkeep);
    t.answer_targets(P0, &[Entity::Object(t.g.current(first))]);
    t.settle();
    assert_eq!(triggers_of(&t, "Oversold Cemetery"), 1);
    // Only three creature cards are left when it resolves; its target still is one.
    t.g.exile_object(t.g.current(last), None);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn an_optional_cost_is_chosen_on_resolution_even_after_the_source_left() {
    cr!("603.5", "113.7a");
    ruling!(
        "Pheres-Band Raiders",
        "If the inspired ability includes an optional cost, you decide whether to pay that cost as the ability resolves. You can do this even if the creature leaves the battlefield in response to the ability."
    );
    let mut t = TestGame::new(2);
    let raiders = t.battlefield(P0, "Pheres-Band Raiders");
    t.g.tap(raiders);
    t.lands(P0, "Forest", 3);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::Upkeep);
    let asked_before = t.asked().len();
    t.settle();
    assert_eq!(triggers_of(&t, "Pheres-Band Raiders"), 1);
    assert!(
        !t.asked()[asked_before..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::YesNo { .. })),
        "nothing about paying is asked as it goes on the stack"
    );
    t.g.destroy(raiders, None);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.in_graveyard(P0, "Pheres-Band Raiders"));
    assert_eq!(tokens_named(&t, P0, "Centaur"), 1);
}

#[test]
fn an_unless_choice_is_made_on_resolution_even_if_the_spell_was_countered() {
    cr!("603.5");
    ruling!(
        "Rhystic Study",
        "It resolves even if that spell is countered. The player gets the option to pay when this triggered ability resolves."
    );
    ruling!(
        "Rhystic Study",
        "You don't have to decide whether or not to draw a card until after the player decides whether or not to pay."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhystic Study");
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.settle();
    assert_eq!(triggers_of(&t, "Rhystic Study"), 1);
    // The Bolt is countered before the Study's ability resolves (it's below it).
    t.g.counter(spell, None);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    let asked_before = t.asked().len();
    let hand = t.hand_size(P0);
    t.answer_yes(P1, false);
    t.answer_yes(P0, true);
    t.resolve();
    let asked: Vec<PlayerId> = t.asked()[asked_before..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(asked, vec![P1, P0], "P1 decides about paying, then P0 about drawing");
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_state_trigger_triggers_once_until_it_leaves_the_stack_and_isnt_rechecked() {
    cr!("603.8");
    supported("Dandân");
    let mut t = TestGame::new(2);
    let dandan = t.battlefield(P0, "Dandân");
    t.settle();
    t.settle();
    assert_eq!(triggers_of(&t, "Dandân"), 1, "once, while it's on the stack");
    // Countered: the state still holds, so it triggers again.
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    let ability = *t.stack.last().unwrap();
    t.cast(P1, stifle).target(ability).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Dandân"), 1, "triggered again");
    assert!(t.on_battlefield(dandan));
    // An Island arrives before it resolves: a state trigger has no intervening "if", so
    // the creature is still sacrificed.
    t.battlefield(P0, "Island");
    t.resolve();
    assert!(t.in_graveyard(P0, "Dandân"));
    assert_eq!(t.stack_len(), 0);
}
