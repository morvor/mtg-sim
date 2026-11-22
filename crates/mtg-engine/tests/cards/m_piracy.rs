//! Piracy (hand-written, `src/cards/piracy.rs`).

use mtg_engine::mana::ManaRestriction;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pirate(t: &mut TestGame) -> Vec<ObjectId> {
    t.lands(P0, "Island", 2);
    let forests = t.lands(P1, "Forest", 2);
    let piracy = t.hand(P0, "Piracy");
    t.cast(P0, piracy).go();
    t.resolve_all();
    forests
}

#[test]
fn you_may_tap_opponents_lands_to_cast_spells() {
    cr!("605.3a", "106.6");
    let mut t = TestGame::new(2);
    let forests = pirate(&mut t);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(forests.iter().all(|f| t.obj(*f).tapped));
}

#[test]
fn that_mana_can_only_be_spent_on_spells() {
    cr!("106.6");
    let mut t = TestGame::new(2);
    let forests = pirate(&mut t);
    t.activate(P0, forests[0], 0, &[]).unwrap();
    let pool = &t.g.players[0].mana_pool.mana;
    assert_eq!(pool.len(), 1);
    assert_eq!(pool[0].restriction, Some(ManaRestriction::SpellsOnly));
}

#[test]
fn only_until_end_of_turn() {
    cr!("605.3a");
    let mut t = TestGame::new(2);
    let forests = pirate(&mut t);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(t.activate(P0, forests[0], 0, &[]).is_err());
}
