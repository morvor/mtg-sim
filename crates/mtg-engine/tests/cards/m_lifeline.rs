//! Lifeline (hand-written, `src/cards/lifeline.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn any_players_creature_returns_at_the_next_end_step() {
    cr!("603.4", "603.7");
    ruling!("Lifeline", "It works for all players");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lifeline");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bear).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P1);
}

#[test]
fn nothing_returns_if_no_other_creature_is_on_the_battlefield() {
    cr!("603.4");
    ruling!("Lifeline", "there must be at least one creature on the battlefield at the time the ability resolves");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lifeline");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(bear).go();
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}
