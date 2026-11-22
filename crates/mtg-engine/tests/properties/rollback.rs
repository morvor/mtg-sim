//! Rollback identity: an action the engine reversed leaves the game as it was (see
//! `mtg_engine::properties::rollback_identity`).

use super::common::*;
use mtg_engine::properties::*;
use mtg_engine::*;

#[test]
fn reversed_actions_leave_the_game_as_it_was() {
    cr!("733.1", "733.2");
    let mut checked = 0;
    for s in fixed_set() {
        let r = rollback_identity(&s).unwrap_or_else(|e| panic!("seed {}: {e}", s.seed));
        checked += r.rollbacks;
    }
    // The agents attempt actions that can't be completed often enough to test this.
    assert!(checked > 50, "only {checked} reversed actions");
}
