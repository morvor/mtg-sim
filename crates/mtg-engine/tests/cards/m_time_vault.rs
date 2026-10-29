//! Time Vault (hand-written, `src/cards/time_vault.rs`): "If you would begin your turn
//! while ~ is tapped, you may skip that turn instead. If you do, untap ~." (CR 614.10).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn skipping_a_turn_untaps_it_and_the_next_player_takes_a_turn() {
    cr!("614.10", "614.10a");
    ruling!("Time Vault", "Time Vault untaps before anything else happens in the next turn");
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Time Vault");
    t.g.tap(vault);
    t.set_step(P1, Step::PrecombatMain);
    let turns = t.g.player(P0).turns_taken;
    t.answer_yes(P0, true);
    // P0's turn is skipped: the next turn is P1's again.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.player(P0).turns_taken, turns);
    assert!(!t.obj_now(vault).tapped);
}

#[test]
fn declining_takes_the_turn_and_it_stays_tapped() {
    cr!("614.10");
    ruling!("Time Vault", "The choice of whether or not to skip a turn is made as that turn would begin");
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Time Vault");
    t.g.tap(vault);
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    // "doesn't untap during your untap step".
    assert!(t.obj_now(vault).tapped);
}

#[test]
fn an_untapped_vault_offers_no_skip() {
    cr!("614.10");
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Time Vault");
    t.set_step(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(vault).tapped);
}

#[test]
fn tap_for_an_extra_turn_then_skip_the_next_turn_to_untap_it() {
    cr!("614.10", "500.7");
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Time Vault");
    t.set_step(P0, Step::PrecombatMain);
    t.activate(P0, vault, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(vault).tapped);
    let turns = t.g.player(P0).turns_taken;
    // The extra turn would begin with the vault tapped: skip it.
    t.answer_yes(P0, true);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.player(P0).turns_taken, turns);
    assert!(!t.obj_now(vault).tapped);
}
