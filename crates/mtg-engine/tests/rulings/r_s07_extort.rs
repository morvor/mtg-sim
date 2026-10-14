//! Rulings batch S07 — extort (CR 702.101): "Whenever you cast a spell, you may pay
//! {W/B}. If you do, each opponent loses 1 life and you gain that much life."

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s07_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

fn pay_questions(t: &TestGame, from: usize) -> usize {
    count_asked(t, from, |d| {
        matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay"))
    })
}

#[test]
fn extort_doesnt_target() {
    cr!("702.101a", "115.1");
    ruling!(
        "Basilica Screecher",
        "The extort ability doesn't target any player."
    );
    supported("Basilica Screecher");
    supported("Leyline of Sanctity");
    // P1 has hexproof (Leyline of Sanctity): extort still makes them lose life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Basilica Screecher");
    t.battlefield(P1, "Leyline of Sanctity");
    t.lands(P0, "Swamp", 1);
    let from = t.asked().len();
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Extort"), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(
        count_asked(&t, from, |d| matches!(d, Decision::ChooseTargets { .. })),
        0
    );
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn extort_is_paid_at_most_once_per_trigger_as_it_resolves() {
    cr!("702.101a", "702.101b");
    ruling!(
        "Syndicate Heavy",
        "You may pay {W}{B} a maximum of one time for each extort triggered ability. You decide whether to pay when the ability resolves."
    );
    supported("Syndicate Heavy");
    // Syndicate Heavy: extort. One spell, one trigger: with plenty of mana, it's paid once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syndicate Heavy");
    t.lands(P0, "Plains", 4);
    let from = t.asked().len();
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Extort"), 1);
    // Nothing is decided until it resolves.
    assert_eq!(pay_questions(&t, from), 0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(pay_questions(&t, from), 1);
    assert_eq!(untapped_lands(&t, P0), 3);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
    // Two permanents with extort: each triggers separately, and each is paid once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syndicate Heavy");
    t.battlefield(P0, "Basilica Screecher");
    t.lands(P0, "Plains", 4);
    let from = t.asked().len();
    let memnite = t.hand(P0, "Memnite");
    t.cast(P0, memnite).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Extort"), 2);
    for _ in 0..4 {
        t.answer_yes(P0, true);
    }
    t.resolve_all();
    assert_eq!(pay_questions(&t, from), 2);
    assert_eq!(untapped_lands(&t, P0), 2);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}
