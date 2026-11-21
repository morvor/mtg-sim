//! Unbound Flourishing (hand-written, `src/cards/unbound_flourishing.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

fn walker_with(t: &mut TestGame, x: i64) -> ObjectId {
    t.lands(P0, "Wastes", 2 * x as usize);
    let w = t.hand(P0, "Hangarback Walker");
    t.cast(P0, w).x(x).go();
    t.resolve_all();
    t.named_on_battlefield("Hangarback Walker")[0]
}

#[test]
fn doubles_x_of_a_permanent_spell() {
    cr!("107.3m");
    ruling!("Unbound Flourishing", "raises the value of X for permanent spells, but you don't pay any more mana");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    let w = walker_with(&mut t, 1);
    assert_eq!(t.counters(w, "+1/+1"), 2);
}

#[test]
fn two_of_them_multiply_x_by_four() {
    cr!("107.3m");
    ruling!("Unbound Flourishing", "the value of X for permanent spells will be multiplied by four");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    t.battlefield(P0, "Unbound Flourishing");
    let w = walker_with(&mut t, 1);
    assert_eq!(t.counters(w, "+1/+1"), 4);
}

#[test]
fn x_of_0_stays_0_and_the_doubled_x_changes_the_mana_value_on_the_stack() {
    cr!("202.3e");
    ruling!("Unbound Flourishing", "will double the value to 0 for permanent spells");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Unbound Flourishing");
    t.lands(P0, "Wastes", 2);
    let w = t.hand(P0, "Hangarback Walker");
    let spell = t.cast(P0, w).x(1).go();
    assert_eq!(t.g.mana_value_of(spell), 2);
    // Resolve the trigger only.
    t.resolve();
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    let w0 = t.hand(P0, "Hangarback Walker");
    t.cast(P0, w0).x(0).go();
    t.resolve_all();
    // The 0/0 dies to state-based actions with no counters.
    assert_eq!(t.named_on_battlefield("Hangarback Walker").len(), 1);
}
