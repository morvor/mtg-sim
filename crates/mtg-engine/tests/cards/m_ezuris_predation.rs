//! Ezuri's Predation (hand-written, `src/cards/ezuris_predation.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn each_beast_fights_a_different_creature() {
    cr!("701.14a");
    ruling!("Ezuri's Predation", "You choose which Beast is fighting which creature");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 8);
    let s = t.hand(P0, "Ezuri's Predation");
    t.cast(P0, s).go();
    t.resolve();
    // Both opposing creatures fought a 4/4 Beast.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(!t.on_battlefield(bear));
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert!(!t.on_battlefield(wurm));
    let beasts: Vec<_> = t
        .g
        .permanents_controlled_by(P0)
        .into_iter()
        .filter(|&o| t.g.obj(o).chars.subtypes.iter().any(|s| s == "Beast"))
        .collect();
    // The one that fought the 6/4 Craw Wurm died; the other has 2 damage.
    assert_eq!(beasts.len(), 1);
    assert_eq!(t.g.obj(beasts[0]).damage, 2);
}
