//! CR 614.10 audit: skip effects. A "skip your next [step]" effect waits for the first
//! occurrence that isn't skipped (614.10a), so with a static skip effect also skipping that
//! step, it is still waiting when the static effect ends.

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to P0's next precombat main phase (through P1's turn).
fn next_p0_main(t: &mut TestGame) {
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
}

#[test]
fn fatigue_waits_for_the_first_draw_step_necropotence_doesnt_skip() {
    cr!("614.10", "614.10a", "614.1b");
    supported("Necropotence");
    supported("Fatigue");
    let mut t = TestGame::new(2);
    let necro = t.battlefield(P0, "Necropotence");
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Fatigue", &[Entity::Player(P0)]);
    let h = t.hand_size(P0);
    // Necropotence skips P0's draw step; Fatigue's skip waits for one that isn't skipped.
    next_p0_main(&mut t);
    assert_eq!(t.hand_size(P0), h);
    next_p0_main(&mut t);
    assert_eq!(t.hand_size(P0), h);
    // Without Necropotence, the next draw step is the one Fatigue skips.
    cast_resolve(&mut t, P1, "Disenchant", &[obj(necro)]);
    next_p0_main(&mut t);
    assert_eq!(t.hand_size(P0), h);
    // Then P0 draws again.
    next_p0_main(&mut t);
    assert_eq!(t.hand_size(P0), h + 1);
}
