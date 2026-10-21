//! Twinning Staff (hand-written, `src/cards/twinning_staff.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn copying_a_spell_makes_an_additional_copy() {
    cr!("707.10", "614.1a");
    ruling!("Twinning Staff", "If you have a second Twinning Staff, copying a spell results in two additional copies");
    for (staves, life) in [(0, 14), (1, 11), (2, 8)] {
        let mut t = TestGame::new(2);
        for _ in 0..staves {
            t.battlefield(P0, "Twinning Staff");
        }
        t.lands(P0, "Mountain", 1);
        let bolt = t.hand(P0, "Lightning Bolt");
        let spell = t.cast(P0, bolt).target(P1).go();
        t.lands(P0, "Mountain", 2);
        let rev = t.hand(P0, "Reverberate");
        t.cast(P0, rev).target(spell).go();
        t.resolve_all();
        assert_eq!(t.life(P1), life, "{staves} staves");
    }
}

#[test]
fn opponents_copies_arent_affected() {
    cr!("707.10");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Twinning Staff");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.lands(P1, "Mountain", 2);
    let rev = t.hand(P1, "Reverberate");
    t.cast(P1, rev).target(spell).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
}
