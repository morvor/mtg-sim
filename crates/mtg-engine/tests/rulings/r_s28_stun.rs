//! Rulings batch S28 — stun counters (CR 122.1d): one or more stun counters create a
//! single replacement effect, "If a permanent with a stun counter on it would become
//! untapped, instead remove a stun counter from it" — for any untap, in the untap step or
//! not, one counter at a time. It doesn't untap (so "becomes untapped" doesn't trigger),
//! and an untap cost is still paid.

use crate::r_s01_common::{supported, tokens};
use crate::r_s04_common::next_upkeep;
use crate::r_s28_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

const STUN: &str = "stun";

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

#[test]
fn stun_counters_replace_untapping_by_an_effect_and_in_the_untap_step() {
    cr!("122.1d", "502.3");
    ruling!(
        "Impede Momentum",
        "Stun counters replace untapping for any reason, including players untapping tapped permanents during their untap steps."
    );
    supported("Impede Momentum");
    supported("Kiora's Follower");
    // "Tap target creature and put three stun counters on it. Scry 1."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_card(&mut t, P0, "Impede Momentum");
    t.resolve_all();
    assert!(tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 3);
    // Kiora's Follower: "{T}: Untap another target permanent."
    let follower = t.battlefield(P1, "Kiora's Follower");
    t.activate(P1, follower, 0, &[Entity::Object(bears)])
        .expect("untap the Bears");
    t.resolve_all();
    assert!(tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 2);
    // P1's untap step.
    next_upkeep(&mut t, P1);
    assert!(tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
    assert!(!tapped(&t, follower));
}

#[test]
fn only_one_stun_counter_is_removed_each_time() {
    cr!("122.1d", "502.3");
    ruling!(
        "Involuntary Cooldown",
        "If a permanent has more than one stun counter on it, only one will be removed each time it would become untapped."
    );
    supported("Involuntary Cooldown");
    // "Tap up to two target artifacts and/or creatures. Put two stun counters on each of
    // them."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(giant)]);
    cast_card(&mut t, P0, "Involuntary Cooldown");
    t.resolve_all();
    for id in [bears, giant] {
        assert!(tapped(&t, id));
        assert_eq!(t.counters(id, STUN), 2);
    }
    next_upkeep(&mut t, P1);
    for id in [bears, giant] {
        assert!(tapped(&t, id));
        assert_eq!(t.counters(id, STUN), 1);
    }
    next_upkeep(&mut t, P1);
    for id in [bears, giant] {
        assert!(tapped(&t, id));
        assert_eq!(t.counters(id, STUN), 0);
    }
    next_upkeep(&mut t, P1);
    assert!(!tapped(&t, bears) && !tapped(&t, giant));
}

#[test]
fn a_stun_counter_is_removed_instead_of_untapping() {
    cr!("122.1d", "614.1a");
    ruling!(
        "Frostfist Strider",
        "If a tapped permanent with a stun counter on it would become untapped, a stun counter will be removed from it instead. This is a replacement effect."
    );
    supported("Frostfist Strider");
    supported("Vitalize");
    // "When this creature enters, tap target creature an opponent controls and put a stun
    // counter on it."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Frostfist Strider");
    t.resolve_all();
    assert!(tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
    // Vitalize: "Untap all creatures you control."
    cast_card(&mut t, P1, "Vitalize");
    t.resolve_all();
    assert!(tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 0);
    cast_card(&mut t, P1, "Vitalize");
    t.resolve_all();
    assert!(!tapped(&t, bears));
    // An untapped permanent with a stun counter doesn't become untapped: the counter
    // stays.
    t.g.add_counters(Entity::Object(bears), STUN, 1, None);
    cast_card(&mut t, P1, "Vitalize");
    t.resolve_all();
    assert!(!tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
}

#[test]
fn several_stun_counters_make_a_single_replacement_effect() {
    cr!("122.1d", "700.2");
    ruling!(
        "Sygg's Command",
        "One or more stun counters on a permanent create a single replacement effect that stops the permanent from untapping. That effect is \"If a permanent with a stun counter on it would become untapped, instead remove a stun counter from it.\""
    );
    supported("Sygg's Command");
    // "• Target player draws a card. • Tap target creature. Put a stun counter on it."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    for _ in 0..2 {
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2, 3]));
        t.answer_targets(P0, &[Entity::Player(P0)]);
        t.answer_targets(P0, &[Entity::Object(bears)]);
        cast_card(&mut t, P0, "Sygg's Command");
        t.resolve_all();
    }
    assert!(tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 2);
    // One untap event: the single effect removes one counter.
    next_upkeep(&mut t, P1);
    assert!(tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
}

#[test]
fn becomes_untapped_triggers_dont_trigger_when_a_stun_counter_is_removed() {
    cr!("122.1d", "603.2");
    ruling!(
        "Stall for Time",
        "Abilities that trigger when a permanent \"becomes untapped\" won't trigger if a stun counter is removed instead."
    );
    supported("Stall for Time");
    supported("Pheres-Band Tromper");
    // Stall for Time, kicked: "Tap up to two target creatures. If this spell was kicked,
    // put a stun counter on each of those creatures. Draw a card." Pheres-Band Tromper:
    // "Whenever this creature becomes untapped, put a +1/+1 counter on it."
    let mut t = TestGame::new(2);
    let tromper = t.battlefield(P1, "Pheres-Band Tromper");
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    t.answer_targets(P0, &[Entity::Object(tromper)]);
    crate::r_s25_common::lands_for_cost(&mut t, P0, "Stall for Time");
    let stall = t.hand(P0, "Stall for Time");
    t.cast(P0, stall).kicked(true).go();
    t.resolve_all();
    assert!(tapped(&t, tromper));
    assert_eq!(t.counters(tromper, STUN), 1);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(tapped(&t, tromper));
    assert_eq!(t.counters(tromper, "+1/+1"), 0);
    // Next time it really untaps, and the ability triggers.
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(!tapped(&t, tromper));
    assert_eq!(t.counters(tromper, "+1/+1"), 1);
}

#[test]
fn an_untap_cost_can_be_paid_by_removing_a_stun_counter() {
    cr!("122.1d", "118.1");
    ruling!(
        "Involuntary Cooldown",
        "If untapping a permanent is part of a cost (such as that of Halo Fountain's first ability), you may pay that cost by \"untapping\" a tapped permanent with a stun counter on it. The stun counter will be removed and the creature will remain tapped. However, the cost will still be paid."
    );
    supported("Halo Fountain");
    // Halo Fountain: "{W}, {T}, Untap a tapped creature you control: Create a 1/1 green and
    // white Citizen creature token."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_card(&mut t, P0, "Involuntary Cooldown");
    t.resolve_all();
    assert_eq!(t.counters(bears, STUN), 2);
    let fountain = t.battlefield(P0, "Halo Fountain");
    t.lands(P0, "Plains", 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, fountain, 0, &[])
        .expect("pay the untap cost");
    assert!(tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    // Without a tapped creature, the cost can't be paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let fountain = t.battlefield(P0, "Halo Fountain");
    t.lands(P0, "Plains", 1);
    assert!(t.activate(P0, fountain, 0, &[]).is_err());
}

#[test]
fn an_untap_symbol_cost_can_be_paid_by_removing_a_stun_counter() {
    cr!("122.1d", "107.6");
    ruling!(
        "Involuntary Cooldown",
        "If untapping a permanent is part of a cost (such as that of Halo Fountain's first ability), you may pay that cost by \"untapping\" a tapped permanent with a stun counter on it. The stun counter will be removed and the creature will remain tapped. However, the cost will still be paid."
    );
    supported("Safehold Sentry");
    // Safehold Sentry: "{2}{W}, {Q}: This creature gets +0/+2 until end of turn."
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Safehold Sentry");
    t.g.tap(sentry);
    t.g.add_counters(Entity::Object(sentry), STUN, 1, None);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    t.activate(P0, sentry, 0, &[])
        .expect("{Q} is paid by removing the stun counter");
    assert!(tapped(&t, sentry));
    assert_eq!(t.counters(sentry, STUN), 0);
    t.resolve_all();
    assert_eq!(t.pt(sentry), (2, 4));
    // An untapped permanent can't pay {Q}.
    let mut t = TestGame::new(2);
    let sentry = t.battlefield(P0, "Safehold Sentry");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 2);
    assert!(t.activate(P0, sentry, 0, &[]).is_err());
}

#[test]
fn crackleburrs_untap_cost_needs_two_other_tapped_creatures() {
    cr!("107.6", "118.3");
    ruling!(
        "Crackleburr",
        "To activate either ability, you'll need Crackleburr plus two other creatures. Crackleburr must have been under your control since your most recent turn began (or have haste), but the other two creatures don't."
    );
    supported("Crackleburr");
    // "{U/R}{U/R}, {Q}, Untap two tapped blue creatures you control: Return target creature
    // to its owner's hand." Crackleburr is blue and red, but it can't be one of the two.
    let mut t = TestGame::new(2);
    let burr = t.battlefield(P0, "Crackleburr");
    let merfolk = t.battlefield(P0, "Coral Merfolk");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(burr);
    t.g.tap(merfolk);
    t.lands(P0, "Island", 2);
    assert!(t.activate(P0, burr, 1, &[]).is_err());
    assert!(tapped(&t, burr) && tapped(&t, merfolk));
    // A second tapped blue creature, which just entered, is enough.
    let drake = t.enter(P0, "Wind Drake");
    t.g.tap(drake);
    t.activate(P0, burr, 1, &[Entity::Object(bears)])
        .expect("untap Crackleburr and two other blue creatures");
    assert!(!tapped(&t, burr) && !tapped(&t, merfolk) && !tapped(&t, drake));
    t.resolve_all();
    assert!(t.in_hand(P1, "Grizzly Bears"));
}
