//! Ertai's Meddling (hand-written, `src/cards/ertais_meddling.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn the_spell_comes_back_when_its_delay_counters_run_out() {
    cr!("608.2", "707.10");
    ruling!("Ertai's Meddling", "it does not have its controller cast the spell again");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Island", 2);
    let em = t.hand(P0, "Ertai's Meddling");
    t.cast(P0, em).x(1).target(spell).go();
    t.resolve();
    assert!(t.in_exile("Lightning Bolt"));
    assert_eq!(t.life(P0), 20);
    // P1's next upkeep: the last delay counter comes off and the Bolt is put back.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn more_counters_wait_more_upkeeps() {
    cr!("608.2");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(P0).go();
    t.lands(P0, "Island", 3);
    let em = t.hand(P0, "Ertai's Meddling");
    t.cast(P0, em).x(2).target(spell).go();
    t.resolve();
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.in_exile("Lightning Bolt"));
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}
