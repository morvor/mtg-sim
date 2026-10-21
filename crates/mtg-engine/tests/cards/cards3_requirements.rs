//! Combat grammar IV: attack and block requirements, restrictions and evasion scoped to a
//! combat (CR 508.1c-d, 509.1b-c): Noctis, Heir Apparent.

use crate::combat2_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn noctis_warp_strike_returns_attacking_and_unblockable_that_combat() {
    cr!("508.4", "509.1b");
    supported("Noctis, Heir Apparent");
    let mut t = TestGame::new(2);
    let noctis = t.battlefield(P0, "Noctis, Heir Apparent");
    t.lands(P0, "Mountain", 3);
    let blocker = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, noctis, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_exile("Noctis, Heir Apparent"));
    // No attackers are declared; Noctis returns attacking as the step begins.
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let n = t.named_on_battlefield("Noctis, Heir Apparent");
    assert_eq!(n.len(), 1);
    assert!(t.g.obj(n[0]).tapped);
    assert!(t.g.is_attacking(n[0]));
    assert!(!legal_blocks(&mut t, &[(blocker, n[0])]));
    to_end_of_combat(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn slicer_goes_to_the_opponent_goaded_and_unsacrificeable() {
    cr!("701.15a");
    supported("Slicer, Hired Muscle // Slicer, High-Speed Antagonist");
    let mut t = TestGame::new(2);
    let slicer = t.battlefield(P0, "Slicer, Hired Muscle // Slicer, High-Speed Antagonist");
    t.g.objects[slicer.0 as usize].tapped = true;
    t.set_step(P0, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let o = t.g.obj(slicer);
    assert_eq!(o.controller, P1);
    assert!(!o.tapped);
    assert!(t.g.goaders(slicer).contains(&P0));
    assert!(t.g.cant_be_sacrificed(slicer));
    // Back under P0's control next turn, and sacrificeable again.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.g.obj(slicer).controller, P0);
    assert!(!t.g.cant_be_sacrificed(slicer));
}
