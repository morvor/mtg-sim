//! Rulings batch P217 — kicker (CR 702.33): an optional additional cost, paid at most once
//! per kicker cost; "kicked" is checked as the spell's (or permanent's) abilities need it.

use crate::r_s01_common::{give_mana_for, supported, triggers_on_stack};
use crate::r_s04_common::{add_mana, asked_of_since};
use crate::r_s08_common::mana_value;
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Thornscape Battlemage ({2}{G}; "Kicker {R} and/or {W}") with the given
/// kicker choices; returns the spell, and how many kicker choices P0 was offered.
fn battlemage(t: &mut TestGame, red: bool, white: bool) -> (ObjectId, usize) {
    supported("Thornscape Battlemage");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    add_mana(t, P0, ManaType::R, 2);
    add_mana(t, P0, ManaType::W, 2);
    let card = t.hand(P0, "Thornscape Battlemage");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::OptionalCost, mtg_engine::decision::Answer::Bool(red));
    t.answer(P0, DecisionKind::OptionalCost, mtg_engine::decision::Answer::Bool(white));
    let spell = t.cast(P0, card).go();
    let offered = asked_of_since(t, P0, from, |d| matches!(d, Decision::OptionalCost { .. }));
    (spell, offered)
}

#[test]
fn kicker_costs_dont_change_mana_cost_or_mana_value() {
    cr!("702.33a", "118.8d", "202.3");
    ruling!(
        "Thornscape Battlemage",
        "Kicker costs don't change a spell's mana cost or mana value."
    );
    let mut t = TestGame::new(2);
    let (spell, _) = battlemage(&mut t, true, true);
    assert_eq!(mana_value(&t, spell), 3);
    assert_eq!(
        t.obj(spell).chars.mana_cost,
        mtg_engine::card::card("Thornscape Battlemage")
            .front()
            .chars
            .mana_cost
            .clone()
    );
}

#[test]
fn each_kicker_cost_can_be_paid_only_once() {
    cr!("702.33a", "702.33c", "702.33e");
    ruling!(
        "Thornscape Battlemage",
        "You can pay any particular kicker cost only once. You can't pay it multiple times to get multiples of either triggered ability."
    );
    let mut t = TestGame::new(2);
    let artifact = t.battlefield(P1, "Ornithopter");
    // One choice per kicker cost; with both paid, each ability triggers once.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Object(artifact)]);
    let (_, offered) = battlemage(&mut t, true, true);
    assert_eq!(offered, 2);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "kicked with its"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    // Only {R}: just the damage trigger, once (mana left over can't pay it again).
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let (_, offered) = battlemage(&mut t, true, false);
    assert_eq!(offered, 2);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "kicked with its"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn skizzik_checks_each_end_step_whether_it_was_kicked() {
    cr!("702.33d", "603.4", "607.2i");
    ruling!(
        "Skizzik",
        "Skizzik's ability checks at each end step whether it was kicked while it was being cast. You don't have to pay its kicker cost each turn (and can't do so, even if you really want to kick it again)."
    );
    supported("Skizzik");
    // Kicked: it survives this end step and later ones.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Skizzik");
    add_mana(&mut t, P0, ManaType::R, 1);
    let card = t.hand(P0, "Skizzik");
    t.cast(P0, card).kicked(true).go();
    t.resolve_all();
    let skizzik = t.named_on_battlefield("Skizzik")[0];
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "wasn't kicked"), 0);
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "wasn't kicked"), 0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(skizzik));
    // Not kicked: sacrificed at the first end step.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Skizzik");
    let card = t.hand(P0, "Skizzik");
    t.cast(P0, card).kicked(false).go();
    t.resolve_all();
    let skizzik = t.named_on_battlefield("Skizzik")[0];
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(skizzik));
    assert!(t.in_graveyard(P0, "Skizzik"));
}

#[test]
fn sphinx_of_lost_truths_always_draws_and_discards_only_if_unkicked() {
    cr!("702.33d", "603.6a");
    ruling!(
        "Sphinx of Lost Truths",
        "Sphinx of Lost Truths behaves differently than other cards with kicker. Its \"enters\" ability always triggers, and it checks whether it *wasn't* kicked."
    );
    supported("Sphinx of Lost Truths");
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        give_mana_for(&mut t, P0, "Sphinx of Lost Truths");
        add_mana(&mut t, P0, ManaType::U, 2);
        let card = t.hand(P0, "Sphinx of Lost Truths");
        t.cast(P0, card).kicked(kicked).go();
        t.resolve();
        assert_eq!(triggers_on_stack(&t, "draw three cards"), 1);
        let hand = t.hand_size(P0);
        let gy = t.graveyard_size(P0);
        t.resolve_all();
        if kicked {
            assert_eq!(t.hand_size(P0), hand + 3);
            assert_eq!(t.graveyard_size(P0), gy);
        } else {
            assert_eq!(t.hand_size(P0), hand);
            assert_eq!(t.graveyard_size(P0), gy + 3);
        }
    }
}

#[test]
fn a_summoning_sick_vampire_can_be_tapped_for_blood_tributes_kicker() {
    cr!("302.6", "702.33a", "118.8");
    ruling!(
        "Blood Tribute",
        "You can tap a creature that hasn't been under your control since your most recent turn began to pay the kicker cost."
    );
    supported("Blood Tribute");
    let mut t = TestGame::new(2);
    // Vampire Interloper just came under P0's control.
    let vamp = t.battlefield_sick(P0, "Vampire Interloper");
    give_mana_for(&mut t, P0, "Blood Tribute");
    let card = t.hand(P0, "Blood Tribute");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(vamp)]);
    t.cast(P0, card).kicked(true).go();
    assert!(t.obj_now(vamp).tapped);
    t.resolve_all();
    // "Target opponent loses half their life, rounded up. If this spell was kicked, you
    // gain life equal to the life lost this way."
    assert_eq!(t.life(P1), 10);
    assert_eq!(t.life(P0), 30);
}

#[test]
fn goblin_barrage_needs_a_creature_target_which_can_be_sacrificed_for_its_kicker() {
    cr!("601.2c", "601.2h", "608.2b", "702.33a");
    ruling!(
        "Goblin Barrage",
        "You can't cast Goblin Barrage unless you choose a creature as a target, even if it's kicked. However, you can target a Goblin or artifact creature you control and then sacrifice it to pay the kicker cost. The target player or planeswalker will be dealt 4 damage."
    );
    supported("Goblin Barrage");
    // No creature to target: it can't be cast, kicked or not.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Goblin Barrage");
    let card = t.hand(P0, "Goblin Barrage");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    assert!(t.cast(P0, card).kicked(true).try_go().is_err());
    assert_eq!(t.zone(card), Zone::Hand(P0));
    // P0's Raging Goblin is both the target and the sacrifice for the kicker cost: the
    // spell still deals 4 damage to the target player.
    let mut t = TestGame::new(2);
    let goblin = t.battlefield(P0, "Raging Goblin");
    give_mana_for(&mut t, P0, "Goblin Barrage");
    let card = t.hand(P0, "Goblin Barrage");
    t.answer_targets(P0, &[Entity::Object(goblin)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[Entity::Object(goblin)]);
    t.cast(P0, card).kicked(true).go();
    assert!(t.in_graveyard(P0, "Raging Goblin"));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}
