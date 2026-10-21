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
