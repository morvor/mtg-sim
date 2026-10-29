//! Tyvar the Bellicose (hand-written, `src/cards/tyvar_the_bellicose.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn creatures_get_counters_for_mana_they_produce_once_each_turn() {
    cr!("605.3b", "603.2");
    ruling!("Tyvar the Bellicose", "the ability can trigger once each turn for each creature you control");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tyvar the Bellicose");
    let elf = t.battlefield(P0, "Llanowar Elves");
    t.activate(P0, elf, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 1);
    // Again this turn: no more.
    t.g.objects[elf.0 as usize].tapped = false;
    t.activate(P0, elf, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(elf, "+1/+1"), 1);
}
