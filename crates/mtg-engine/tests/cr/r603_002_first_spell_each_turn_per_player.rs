//! CR 603.2: "Whenever an opponent casts their first noncreature spell each turn"
//! triggers once for each opponent: another player's earlier spell this turn doesn't make
//! this player's spell not their first.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn esper_sentinel_triggers_for_each_opponents_first_spell() {
    cr!("603.2");
    // "Whenever an opponent casts their first noncreature spell each turn, draw a card
    // unless that player pays {X}, where X is this creature's power." Nobody can pay.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Esper Sentinel");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    for p in [P1, P2] {
        t.lands(p, "Island", 1);
        let opt = t.hand(p, "Opt");
        t.cast(p, opt).go();
        t.resolve_all();
    }
    assert_eq!(t.hand_size(P0), hand + 2);
}
