//! Rulings batch S10 — inspired: "Inspired — Whenever this creature becomes untapped,
//! ..." (an ability word, CR 207.2c).

use crate::r_s01_common::{supported, triggers_on_stack, watch};
use crate::r_s02_common::{can_attack, destroy};
use crate::r_s04_common::next_upkeep;
use crate::r_s05_common::{enter, run_from};
use crate::r_s07_common::is_priority;
use mtg_engine::ability::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const INSPIRED: &str = "becomes untapped";

/// Untaps the permanent as a resolving effect of `p`'s would, and settles.
fn untap(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    run_from(
        t,
        p,
        None,
        Effect::Untap {
            what: Sel::Target(0),
        },
        &[Entity::Object(id)],
    );
}

/// Taps the permanent and settles.
fn tap(t: &mut TestGame, id: ObjectId) {
    t.g.tap(id);
    t.g.flush_events();
    t.settle();
}

#[test]
fn inspired_triggers_however_the_creature_becomes_untapped() {
    cr!("207.2c", "502.3", "603.2");
    ruling!(
        "Kragma Butcher",
        "Inspired abilities trigger no matter how the creature becomes untapped: by the turn-based action at the beginning of the untap step or by a spell or ability."
    );
    supported("Kragma Butcher");
    // Kragma Butcher (2/3): "it gets +2/+0 until end of turn".
    let mut t = TestGame::new(2);
    let butcher = t.battlefield(P0, "Kragma Butcher");
    tap(&mut t, butcher);
    // By an effect.
    untap(&mut t, P0, butcher);
    assert_eq!(triggers_on_stack(&t, INSPIRED), 1);
    t.resolve_all();
    assert_eq!(t.pt(butcher), (4, 3));
    // By the untap step's turn-based action.
    tap(&mut t, butcher);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, INSPIRED), 1);
    t.resolve_all();
    assert_eq!(t.pt(butcher), (4, 3));
}

#[test]
fn an_inspired_optional_cost_is_paid_on_resolution_even_if_the_creature_left() {
    cr!("603.5", "118.12", "608.2b");
    ruling!(
        "Pheres-Band Raiders",
        "If the inspired ability includes an optional cost, you decide whether to pay that cost as the ability resolves. You can do this even if the creature leaves the battlefield in response to the ability."
    );
    supported("Pheres-Band Raiders");
    // "you may pay {2}{G}. If you do, create a 3/3 green Centaur enchantment creature
    // token."
    let mut t = TestGame::new(2);
    let raiders = t.battlefield(P0, "Pheres-Band Raiders");
    t.lands(P0, "Forest", 3);
    tap(&mut t, raiders);
    untap(&mut t, P0, raiders);
    assert_eq!(triggers_on_stack(&t, INSPIRED), 1);
    // Nothing was paid as it triggered.
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 3);
    // In response, the Raiders are destroyed; the ability still resolves, and P0 pays.
    destroy(&mut t, raiders);
    assert!(t.in_graveyard(P0, "Pheres-Band Raiders"));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Centaur").len(),
        1
    );
}

#[test]
fn inspired_doesnt_trigger_on_entering_the_battlefield() {
    cr!("207.2c", "603.2");
    ruling!(
        "Kragma Butcher",
        "Inspired abilities don’t trigger when the creature enters the battlefield."
    );
    let mut t = TestGame::new(2);
    let butcher = enter(&mut t, P0, "Kragma Butcher");
    assert!(!t.obj_now(butcher).tapped);
    assert_eq!(triggers_on_stack(&t, INSPIRED), 0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn inspired_doesnt_trigger_on_entering_the_battlefield_straight() {
    cr!("207.2c", "603.2");
    ruling!(
        "Pain Seer",
        "Inspired abilities don't trigger when the creature enters the battlefield."
    );
    supported("Pain Seer");
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Pain Seer");
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn inspired_triggering_in_the_untap_step_goes_on_the_stack_in_upkeep() {
    cr!("502.4", "503.1a", "302.6");
    ruling!(
        "God-Favored General",
        "If an inspired ability triggers during your untap step, the ability will be put on the stack at the beginning of your upkeep. If the ability creates one or more token creatures, those creatures won’t be able to attack that turn (unless they gain haste)."
    );
    supported("God-Favored General");
    // "you may pay {2}{W}. If you do, create two 1/1 white Soldier enchantment creature
    // tokens."
    let mut t = TestGame::new(2);
    let general = t.battlefield(P0, "God-Favored General");
    t.lands(P0, "Plains", 3);
    tap(&mut t, general);
    let steps = watch(&mut t, P0, is_priority, |g| g.turn.step);
    next_upkeep(&mut t, P0);
    assert_eq!(t.g.turn.step, Step::Upkeep);
    assert_eq!(triggers_on_stack(&t, INSPIRED), 1);
    assert!(!steps.lock().unwrap().contains(&Step::Untap));
    t.answer_yes(P0, true);
    t.resolve_all();
    let soldiers = crate::r_s05_common::tokens_with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 2);
    // The tokens came under P0's control this turn: they can't attack.
    t.advance_to(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, general));
    for s in soldiers {
        assert!(!can_attack(&mut t, s));
    }
}

#[test]
fn inspired_triggering_in_the_untap_step_goes_on_the_stack_in_upkeep_straight() {
    cr!("502.4", "503.1a");
    ruling!(
        "Pain Seer",
        "If an inspired ability triggers during your untap step, the ability will be put on the stack at the beginning of your upkeep."
    );
    // Pain Seer: "reveal the top card of your library and put that card into your hand.
    // You lose life equal to that card's mana value."
    let mut t = TestGame::new(2);
    let seer = t.battlefield(P0, "Pain Seer");
    tap(&mut t, seer);
    let steps = watch(&mut t, P0, is_priority, |g| g.turn.step);
    next_upkeep(&mut t, P0);
    assert!(!steps.lock().unwrap().contains(&Step::Untap));
    assert_eq!(triggers_on_stack(&t, INSPIRED), 1);
    t.library_top(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.in_hand(P0, "Hill Giant"));
    assert_eq!(t.life(P0), 16);
}
