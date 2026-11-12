//! CR 614.10 audit: skip effects. A "skip your next [step]" effect waits for the first
//! occurrence that isn't skipped (614.10a). With a static skip effect also skipping that
//! step, the player chooses which of the two applies (616.1): if it's the static one, the
//! "next" one is still waiting when the static effect ends; if it's the "next" one, it's
//! used up.

use crate::r614_audit_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Advances to P0's next precombat main phase (through P1's turn).
fn next_p0_main(t: &mut TestGame) {
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
}

/// P0 controls Necropotence; on P1's turn P1 casts Fatigue targeting P0. P0's answers to
/// choices among replacement effects are `picks`.
fn fatigue_under_necropotence(picks: &[usize]) -> (TestGame, ObjectId) {
    supported("Necropotence");
    supported("Fatigue");
    let mut t = TestGame::new(2);
    let necro = t.battlefield(P0, "Necropotence");
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Fatigue", &[Entity::Player(P0)]);
    pick_replacements(&mut t, P0, picks);
    (t, necro)
}

#[test]
fn fatigue_waits_for_the_first_draw_step_necropotence_doesnt_skip() {
    cr!("614.10", "614.10a", "614.1b", "616.1");
    // With no answer, the first listed effect applies: Necropotence's.
    interp!("default-first-listed-choice");
    let (mut t, necro) = fatigue_under_necropotence(&[]);
    let h = t.hand_size(P0);
    // Necropotence's effect skips P0's draw step (P0's choice by default); Fatigue's skip
    // waits for one that isn't skipped.
    next_p0_main(&mut t);
    assert_eq!(t.hand_size(P0), h);
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 1, "{choices:?}");
    assert_eq!(choices[0].0, P0);
    assert!(choices[0].1[0].contains("Necropotence"), "{choices:?}");
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

#[test]
fn applying_fatigues_skip_under_necropotence_uses_it_up() {
    cr!("614.10", "614.10a", "616.1", "616.1e", "614.7");
    // P0 chooses Fatigue's effect for the first draw step: it's used up, and Necropotence's
    // has no step left to skip.
    let (mut t, necro) = fatigue_under_necropotence(&[1]);
    let h = t.hand_size(P0);
    next_p0_main(&mut t);
    assert_eq!(t.hand_size(P0), h);
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 1, "{choices:?}");
    assert_eq!(choices[0].0, P0);
    assert_eq!(choices[0].1.len(), 2);
    assert!(choices[0].1[1].contains("next"), "{choices:?}");
    // Necropotence alone skips the next one: no choice to make.
    next_p0_main(&mut t);
    assert_eq!(t.hand_size(P0), h);
    assert_eq!(replacement_choices(&t).len(), 1);
    // Without Necropotence, P0 draws in the very next draw step.
    cast_resolve(&mut t, P1, "Disenchant", &[obj(necro)]);
    next_p0_main(&mut t);
    assert_eq!(t.hand_size(P0), h + 1);
}
