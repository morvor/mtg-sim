//! roundtrip-pairs-2 (docs/agents/items/roundtrip-pairs-2.md): the 3+-card round-trip
//! clusters after paper-scope-1, and the next 25 two-card clusters.

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn hellfire_deals_three_plus_the_creatures_that_died() {
    cr!("107.3c", "700.4");
    // "Destroy all nonblack creatures. Hellfire deals X plus 3 damage to you, where X is
    // the number of creatures that died this way."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Craw Wurm");
    t.battlefield(P1, "Vampire Nighthawk");
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Hellfire");
    t.cast(P0, spell).go();
    t.resolve_all();
    // Three nonblack creatures died; the black one didn't.
    assert_eq!(t.life(P0), 20 - 6, "{}", t.dump_log());
}
