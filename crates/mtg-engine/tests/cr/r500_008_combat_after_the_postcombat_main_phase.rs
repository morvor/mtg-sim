//! CR 500.8: an additional combat phase "after this main phase" cast in the postcombat main
//! phase comes right after it, before the ending phase.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn relentless_assault_in_the_postcombat_main_phase_adds_combat_before_the_end_step() {
    cr!("500.8", "505.1a");
    let mut t = TestGame::new(2);
    let assault = t.hand(P0, "Relentless Assault");
    t.lands(P0, "Mountain", 4);
    t.set_step(P0, Step::PostcombatMain);
    t.cast(P0, assault).go();
    t.resolve_all();
    assert_eq!(
        t.g.turn.schedule.first().copied(),
        Some(Step::BeginningOfCombat)
    );
    // The ending phase comes after the added combat and main phases.
    let end = t.g.turn.schedule.iter().position(|s| *s == Step::End);
    let main = t
        .g
        .turn
        .schedule
        .iter()
        .position(|s| *s == Step::PostcombatMain);
    assert!(main.is_some() && end.is_some() && main < end);
}
