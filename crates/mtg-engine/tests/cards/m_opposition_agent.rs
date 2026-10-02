//! Opposition Agent (hand-written, `src/cards/opposition_agent.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::object::Zone;
use mtg_engine::*;

#[test]
fn an_opponents_found_card_is_exiled_and_you_may_play_it() {
    cr!("701.23a", "723.2");
    ruling!("Opposition Agent", "The cards found in the search will be exiled rather than be put wherever");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opposition Agent");
    t.g.search_finds_by_default = true;
    let forest = t.library_top(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    let rg = t.hand(P1, "Rampant Growth");
    t.cast(P1, rg).go();
    t.resolve();
    let exiled = t.g.current(forest);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.g.permanents_controlled_by(P1).len(), 2);
    // P0 may play it.
    t.set_step(P0, Step::PrecombatMain);
    t.play_land(P0, exiled).unwrap();
    let on_bf = t.g.current(exiled);
    assert!(t.on_battlefield(on_bf));
    assert_eq!(t.obj_now(on_bf).controller, P0);
}

#[test]
fn your_own_searches_are_unaffected() {
    cr!("701.23a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Opposition Agent");
    t.g.search_finds_by_default = true;
    t.library_top(P0, "Forest");
    t.lands(P0, "Forest", 2);
    let rg = t.hand(P0, "Rampant Growth");
    t.cast(P0, rg).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Forest").len(), 3);
}
