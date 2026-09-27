//! Rulings batch S04 — cumulative upkeep (CR 702.24): "At the beginning of your upkeep, if
//! this permanent is on the battlefield, put an age counter on this permanent. Then you
//! may pay [cost] for each age counter on it. If you don't, sacrifice it."

use crate::r_s01_common::*;
use crate::r_s04_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Number of "Pay ...?" questions asked of `p` since decision `from`.
fn pay_questions_since(t: &TestGame, p: PlayerId, from: usize) -> usize {
    asked_of_since(
        t,
        p,
        from,
        |d| matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay")),
    )
}

#[test]
fn the_whole_cumulative_upkeep_is_paid_or_the_permanent_is_sacrificed() {
    cr!("702.24a", "118.3", "118.12");
    ruling!(
        "Soldevi Simulacrum",
        "Paying cumulative upkeep is always optional. If it's not paid, the permanent with cumulative upkeep is sacrificed. Partial payments of the total cumulative upkeep cost can't be made. For example, if a permanent with \"cumulative upkeep {1}\" has three age counters on it when its cumulative upkeep ability triggers, it gets another age counter and then its controller chooses to either pay {4} or sacrifice the permanent."
    );
    supported("Soldevi Simulacrum");
    // Soldevi Simulacrum: "Cumulative upkeep {1}".
    // Three age counters: it gets a fourth, and its controller pays {4}.
    let mut t = TestGame::new(2);
    let sim = t.battlefield(P0, "Soldevi Simulacrum");
    t.g.add_counters(Entity::Object(sim), "age", 3, None);
    t.lands(P0, "Wastes", 5);
    next_upkeep(&mut t, P0);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(pay_questions_since(&t, P0, from), 1);
    assert!(t.on_battlefield(sim));
    assert_eq!(t.counters(sim, "age"), 4);
    assert_eq!(untapped_lands(&t, P0), 1);

    // Paying is optional: declining sacrifices it, though the cost could be paid.
    let mut t = TestGame::new(2);
    let sim = t.battlefield(P0, "Soldevi Simulacrum");
    t.g.add_counters(Entity::Object(sim), "age", 3, None);
    t.lands(P0, "Wastes", 5);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(!t.on_battlefield(sim));
    assert!(t.in_graveyard(P0, "Soldevi Simulacrum"));
    assert_eq!(untapped_lands(&t, P0), 5);

    // No partial payment: with only three lands for {4}, nothing is paid and it's
    // sacrificed.
    let mut t = TestGame::new(2);
    let sim = t.battlefield(P0, "Soldevi Simulacrum");
    t.g.add_counters(Entity::Object(sim), "age", 3, None);
    t.lands(P0, "Wastes", 3);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.on_battlefield(sim));
    assert!(t.in_graveyard(P0, "Soldevi Simulacrum"));
    assert_eq!(untapped_lands(&t, P0), 3);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn a_non_mana_cumulative_upkeep_is_paid_once_for_each_age_counter() {
    cr!("702.24a");
    ruling!(
        "Aboroth",
        "Paying cumulative upkeep is always optional. If it's not paid, the permanent with cumulative upkeep is sacrificed."
    );
    supported("Aboroth");
    // Aboroth: a 9/9 with "Cumulative upkeep—Put a -1/-1 counter on this creature."
    let mut t = TestGame::new(2);
    let aboroth = t.battlefield(P0, "Aboroth");
    t.g.add_counters(Entity::Object(aboroth), "age", 2, None);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve();
    // Three age counters: three -1/-1 counters.
    assert!(t.on_battlefield(aboroth));
    assert_eq!(t.counters(aboroth, "age"), 3);
    assert_eq!(t.counters(aboroth, "-1/-1"), 3);
    assert_eq!(t.pt(aboroth), (6, 6));
    // Next upkeep it isn't paid: it's sacrificed, with no more -1/-1 counters.
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert!(t.in_graveyard(P0, "Aboroth"));
    assert_eq!(t.g.obj(aboroth).counter("-1/-1"), 3);
}
