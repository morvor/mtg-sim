//! Helpers for the `basic_effects_*` card tests.

use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Panics unless every ability of the card compiled.
pub fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

/// The candidates offered by the last target choice asked of `p`.
pub fn last_target_candidates(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .expect("no target choice was asked")
}
