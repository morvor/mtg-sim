//! Shared helpers for the tests of rulings batch S13 (`r_s13_*.rs`): partner with,
//! persist, plot, populate, prepared, proliferate, protection. (The helpers of batches
//! S01–S11 are used too.)

#![allow(dead_code)]

use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts `n` counters of `kind` on a permanent or player (as an effect would).
pub fn add(t: &mut TestGame, e: impl Into<Entity>, kind: &str, n: u32) {
    t.g.add_counters(e.into(), kind, n, None);
    t.g.flush_events();
}
