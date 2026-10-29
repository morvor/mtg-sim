//! Rulings batch S22 — exiling a card, copying it and casting the copy without paying its
//! mana cost (CR 707.12, 118.9): Roving Actuator ("exile up to one target instant or
//! sorcery card with mana value 2 or less from your graveyard. Copy it. You may cast the
//! copy without paying its mana cost.") and Narset, Enlightened Exile ("exile target
//! noncreature, nonland card with mana value less than Narset's power from a graveyard and
//! copy it. You may cast the copy without paying its mana cost.").

use crate::r_s01_common::*;
use crate::r_s22_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn in_graveyard(t: &mut TestGame, name: &str) -> ObjectId {
    t.graveyard(P0, name)
}

/// A nonland permanent left the battlefield this turn (Roving Actuator's void), then
/// Roving Actuator enters targeting `card`.
fn run_actuator(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    t.answer_targets(P0, &[Entity::Object(card)]);
    answers(t);
    t.enter(P0, "Roving Actuator");
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn roving_actuator_casts_the_copy_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "707.12", "608.2g");
    ruling!(
        "Roving Actuator",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, such as that of Embrace Oblivion, those must be paid to cast the spell."
    );
    ruling!(
        "Roving Actuator",
        "You cast the copy while Roving Actuator’s ability is resolving and still on the stack. You can’t wait to cast it later in the turn."
    );
    supported("Roving Actuator");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: in_graveyard,
        run: run_actuator,
    });
}

#[test]
fn roving_actuator_a_copy_not_cast_ceases_to_exist() {
    cr!("707.12", "704.5e");
    ruling!(
        "Roving Actuator",
        "If you don’t want to cast the copy, you can choose not to; the copy ceases to exist the next time state-based actions are checked."
    );
    ruling!(
        "Roving Actuator",
        "If an instant or sorcery card in your graveyard has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    supported("Roving Actuator");
    // Blaze ({X}{R}) has mana value 1 in the graveyard: it can be targeted.
    let mut t = TestGame::new(2);
    let blaze = t.graveyard(P0, "Blaze");
    let exiled_before = t.g.exile.len();
    run_actuator(&mut t, blaze, &|t| {
        t.answer_yes(P0, false);
    });
    // The card is exiled; the copy that wasn't cast is gone.
    assert_eq!(t.zone(blaze), Zone::Exile);
    assert_eq!(t.g.exile.len(), exiled_before + 1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.stack_len(), 0);
}

/// Narset, Enlightened Exile (power 3) attacks P1 unblocked; its trigger targets `card`.
fn run_narset(t: &mut TestGame, card: ObjectId, answers: &dyn Fn(&mut TestGame)) {
    let narset = t.battlefield(P0, "Narset, Enlightened Exile");
    t.answer_targets(P0, &[Entity::Object(card)]);
    answers(t);
    attack_p1_unblocked(t, narset);
    t.clear_answers();
}

#[test]
fn narset_enlightened_exile_casts_the_copy_without_paying_but_additional_costs_are_paid() {
    cr!("118.9a", "118.9b", "118.8a", "601.2b", "707.12", "608.2g");
    ruling!(
        "Narset, Enlightened Exile",
        "If you cast a spell \"without paying its mana cost,\" you can't pay any alternative costs. You can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, you must pay those."
    );
    ruling!(
        "Narset, Enlightened Exile",
        "You cast the copy while the ability is resolving and still on the stack. You can't wait to cast it later in the turn."
    );
    supported("Narset, Enlightened Exile");
    check_free_cast_costs(&FreeCaster {
        instants_only: false,
        place: in_graveyard,
        run: run_narset,
    });
}

#[test]
fn narset_enlightened_exile_a_card_with_mana_value_not_less_than_her_power_isnt_a_target() {
    cr!("115.1", "202.3");
    supported("Narset, Enlightened Exile");
    // Divination (mana value 3) isn't less than Narset's power 3.
    let mut t = TestGame::new(2);
    let div = t.graveyard(P1, "Divination");
    let hand = t.hand_size(P0);
    run_narset(&mut t, div, &|_| {});
    assert_eq!(t.zone(div), Zone::Graveyard(P1));
    assert_eq!(t.hand_size(P0), hand);
    // Opt (mana value 1) in P1's graveyard is.
    let mut t = TestGame::new(2);
    let opt = t.graveyard(P1, "Opt");
    let hand = t.hand_size(P0);
    run_narset(&mut t, opt, &|_| {});
    assert_eq!(t.zone(opt), Zone::Exile);
    assert_eq!(t.hand_size(P0), hand + 1);
}
