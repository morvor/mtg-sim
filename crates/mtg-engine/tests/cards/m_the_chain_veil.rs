//! The Chain Veil (hand-written, `src/cards/the_chain_veil.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn planeswalkers_may_activate_one_more_loyalty_ability() {
    cr!("606.3");
    ruling!("The Chain Veil", "essentially be able to activate a loyalty ability of each planeswalker you control a total of twice");
    let mut t = TestGame::new(2);
    let veil = t.battlefield(P0, "The Chain Veil");
    let jace = t.battlefield(P0, "Jace Beleren");
    t.activate(P0, jace, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.activate(P0, jace, 0, &[]).is_err());
    t.lands(P0, "Island", 4);
    t.activate(P0, veil, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, jace, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.activate(P0, jace, 0, &[]).is_err());
}
