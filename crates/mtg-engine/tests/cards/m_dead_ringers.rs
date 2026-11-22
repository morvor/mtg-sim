//! Dead Ringers (hand-written, `src/cards/dead_ringers.rs`): destroys the two targets
//! only if they're exactly the same colors.

use mtg_engine::testing::*;
use mtg_engine::*;

fn ringers(t: &mut TestGame, a: ObjectId, b: ObjectId) {
    t.lands(P0, "Swamp", 5);
    let s = t.hand(P0, "Dead Ringers");
    t.cast(P0, s).targets(&[a.into(), b.into()]).go();
    t.resolve();
}

#[test]
fn same_colors_are_destroyed() {
    cr!("608.2b", "701.8a");
    ruling!("Dead Ringers", "Both of the target creatures must be exactly the same color");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Llanowar Elves");
    ringers(&mut t, a, b);
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn different_colors_do_nothing() {
    cr!("608.2b");
    ruling!("Dead Ringers", "If they differ in any way, you can still cast the spell, but it does not do anything");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    // Green and white: has a color the other doesn't.
    let b = t.battlefield(P1, "Watchwolf");
    ringers(&mut t, a, b);
    assert!(t.on_battlefield(a));
    assert!(t.on_battlefield(b));
}
