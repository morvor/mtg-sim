//! Opposition Agent (hand-written, `src/cards/opposition_agent.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::object::Zone;
use mtg_engine::*;

#[test]
fn an_opponents_found_card_is_exiled_and_you_may_play_it() {
    cr!("701.23a");
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

fn tutor_exiles(t: &mut TestGame, name: &str) -> ObjectId {
    t.battlefield(P0, "Opposition Agent");
    t.g.search_finds_by_default = true;
    let found = t.library_top(P1, name);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 2);
    let tutor = t.hand(P1, "Demonic Tutor");
    t.cast(P1, tutor).go();
    t.resolve();
    let exiled = t.g.current(found);
    assert_eq!(t.zone(exiled), Zone::Exile);
    t.set_step(P0, Step::PrecombatMain);
    exiled
}

#[test]
fn mana_may_be_spent_as_though_it_were_any_color() {
    cr!("609.4b");
    ruling!("Opposition Agent", "You'll still pay all costs for a spell cast this way");
    let mut t = TestGame::new(2);
    let bears = tutor_exiles(&mut t, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, bears).go();
    t.resolve();
    let on_bf = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(on_bf.len(), 1);
    assert_eq!(t.obj_now(on_bf[0]).controller, P0);
}

#[test]
fn colorless_mana_symbols_still_need_colorless_mana() {
    cr!("609.4b", "107.4c");
    let mut t = TestGame::new(2);
    // {3}{C}: colored mana can't pay the {C}.
    let seer = tutor_exiles(&mut t, "Thought-Knot Seer");
    t.lands(P0, "Mountain", 4);
    assert!(t.cast(P0, seer).try_go().is_err());
    t.lands(P0, "Wastes", 1);
    t.cast(P0, seer).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Thought-Knot Seer").len(), 1);
}
