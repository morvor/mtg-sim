//! Quakebringer (hand-written, `src/cards/quakebringer.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn upkeep(t: &mut TestGame) {
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn triggers_on_the_battlefield() {
    cr!("603.2");
    ruling!("Quakebringer", "its ability will trigger even if it’s no longer a Giant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Quakebringer");
    upkeep(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn triggers_from_your_graveyard_if_you_control_a_giant() {
    cr!("113.6", "603.2");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Quakebringer");
    t.battlefield(P0, "Hill Giant");
    upkeep(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn no_trigger_from_the_graveyard_without_a_giant() {
    cr!("113.6");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Quakebringer");
    t.battlefield(P0, "Grizzly Bears");
    upkeep(&mut t);
    assert_eq!(t.life(P1), 20);
}
