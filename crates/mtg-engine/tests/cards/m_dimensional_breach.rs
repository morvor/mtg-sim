//! Dimensional Breach (hand-written, `src/cards/dimensional_breach.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn each_player_returns_one_card_each_upkeep() {
    cr!("603.7");
    ruling!("Dimensional Breach", "Each player chooses which one of their exiled cards to return");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Plains", 7);
    let s = t.hand(P0, "Dimensional Breach");
    t.cast(P0, s).go();
    t.resolve();
    assert!(t.g.permanents().next().is_none());
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let back: Vec<_> = t.g.permanents().map(|o| o.owner).collect();
    assert_eq!(back, vec![P1]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.permanents().filter(|o| o.owner == P0).count(), 1);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.permanents().filter(|o| o.owner == P1).count(), 2);
}
