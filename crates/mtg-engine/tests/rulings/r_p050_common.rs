//! Shared helpers for the tests of rulings batch P050 (`r_p050_*.rs`): "drain life"
//! effects — a player loses life (or is dealt damage) and you gain life. (The helpers of
//! batches S01–S30 are used too.)

#![allow(dead_code)]

use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::*;

/// A Two-Headed Giant game: P0 and P1 are a team (30 life), P2 and P3 the other.
pub fn two_headed_giant() -> TestGame {
    TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::TwoHeadedGiant,
            teams: Some(vec![0, 0, 1, 1]),
            ..Default::default()
        },
    )
}

/// The life totals of P0's team and of the opposing team (2HG).
pub fn team_lives(t: &TestGame) -> (i32, i32) {
    (t.life(P0), t.life(P2))
}

/// Asserts that, since `before` (from [`team_lives`]), P0's team gained `gain` and the
/// opposing team lost `loss`.
pub fn assert_team_change(t: &TestGame, before: (i32, i32), gain: i32, loss: i32, what: &str) {
    let now = team_lives(t);
    assert_eq!(now.0 - before.0, gain, "{what}: your team's life change");
    assert_eq!(before.1 - now.1, loss, "{what}: the opposing team's life loss");
    // The teammates share the life total.
    assert_eq!(t.life(P1), now.0, "{what}: shared total");
    assert_eq!(t.life(P3), now.1, "{what}: shared total");
}
