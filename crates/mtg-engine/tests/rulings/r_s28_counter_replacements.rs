//! Rulings batch S28 — replacement effects that modify how many counters are put on a
//! permanent (CR 614.1, 616.1): when several apply, the affected permanent's controller
//! chooses the order ("twice that many" then "plus one", or the reverse), and "enters with
//! N counters" is modified too (CR 122.6).

use crate::r_s01_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts one +1/+1 counter on P0's Grizzly Bears with the two counter-modifying permanents
/// `a` and `b` on the battlefield, P0 choosing the `pick`th replacement effect offered
/// first. Returns (counters put, whether the doubling effect was applied first).
fn one_counter(a: &str, b: &str, pick: usize) -> (u32, bool) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, a);
    t.battlefield(P0, b);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    let options: Vec<String> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseReplacement { options } if *p == P0 => Some(options.clone()),
            _ => None,
        })
        .next()
        .expect("P0 chooses the order");
    assert_eq!(options.len(), 2);
    let doubled_first = options[pick].contains("twice");
    (t.counters(bears, "+1/+1"), doubled_first)
}

/// Both orders are possible: doubling first gives 1×2+1 = 3, adding one first (1+1)×2 = 4.
fn check_both_orders(a: &str, b: &str) {
    let mut results = vec![one_counter(a, b, 0), one_counter(a, b, 1)];
    results.sort();
    assert_eq!(results, vec![(3, true), (4, false)]);
}

#[test]
fn ozolith_and_a_doubler_apply_in_the_order_you_choose() {
    cr!("616.1", "616.1e", "614.1a");
    ruling!(
        "Ozolith, the Shattered Spire",
        "If two or more effects attempt to modify how many counters would be put onto a permanent you control, you choose the order to apply those effects, no matter who controls the sources of those effects."
    );
    supported("Ozolith, the Shattered Spire");
    supported("Corpsejack Menace");
    // Ozolith: "that many plus one"; Corpsejack Menace: "twice that many".
    check_both_orders("Ozolith, the Shattered Spire", "Corpsejack Menace");
}

#[test]
fn hardened_scales_and_a_doubler_apply_in_the_order_you_choose() {
    cr!("616.1", "616.1e");
    ruling!(
        "Hardened Scales",
        "If two or more effects attempt to modify how many counters would be put on a creature you control, you choose the order to apply those effects, no matter who controls the sources of those effects."
    );
    supported("Hardened Scales");
    check_both_orders("Hardened Scales", "Corpsejack Menace");
}

#[test]
fn branching_evolution_and_conclave_mentor_apply_in_the_order_you_choose() {
    cr!("616.1", "616.1e");
    ruling!(
        "Branching Evolution",
        "If two or more effects attempt to modify how many counters would be put onto a creature you control, you choose the order to apply those effects, no matter who controls the sources of those effects."
    );
    supported("Branching Evolution");
    supported("Conclave Mentor");
    check_both_orders("Branching Evolution", "Conclave Mentor");
}

#[test]
fn a_creature_entering_with_counters_gets_twice_as_many() {
    cr!("122.6", "614.1c", "614.12");
    ruling!(
        "Corpsejack Menace",
        "If a creature you control would enter the battlefield with a number of +1/+1 counters on it, it enters with twice that many instead."
    );
    // Vebulid: "This creature enters with a +1/+1 counter on it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Corpsejack Menace");
    let vebulid = t.enter(P0, "Vebulid");
    assert_eq!(t.counters(vebulid, "+1/+1"), 2);
    assert_eq!(t.pt(vebulid), (2, 2));
    // Not for an opponent's creature.
    let theirs = t.enter(P1, "Vebulid");
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
}
