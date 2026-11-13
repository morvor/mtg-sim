//! Rulings batch S01 — affinity (CR 702.41): "This spell costs {1} less to cast for each
//! [text] you control."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn cost_increases_apply_before_affinity_and_mana_value_is_unchanged() {
    cr!("601.2f", "702.41a", "202.3");
    ruling!(
        "Utrom Monitor",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as affinity for artifacts). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Utrom Monitor");
    supported("Sphere of Resistance");
    let mut t = TestGame::new(2);
    // "Spells cost {1} more to cast." Five artifacts in all.
    t.battlefield(P0, "Sphere of Resistance");
    for _ in 0..4 {
        t.battlefield(P0, "Ornithopter");
    }
    t.lands(P0, "Island", 3);
    // {4}{U} plus {1}, then minus {5}: {U}. (Reducing first would leave {1}{U}.)
    let monitor = t.hand(P0, "Utrom Monitor");
    let spell = t.cast(P0, monitor).go();
    assert_eq!(tapped_lands(&t, P0), 1);
    // Its mana value is still 5.
    assert_eq!(t.g.mana_value_of(spell), 5);
    t.resolve_all();
    let m = t.named_on_battlefield("Utrom Monitor")[0];
    assert_eq!(t.g.mana_value_of(m), 5);
}
