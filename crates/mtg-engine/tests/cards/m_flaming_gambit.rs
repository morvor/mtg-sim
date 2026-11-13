//! Flaming Gambit (hand-written, `src/cards/flaming_gambit.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_player_may_have_a_creature_take_the_damage() {
    cr!("120.3");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let s = t.hand(P0, "Flaming Gambit");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(bear)]);
    t.cast(P0, s).x(3).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn or_take_it_themselves() {
    cr!("120.3");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let s = t.hand(P0, "Flaming Gambit");
    t.answer_yes(P1, false);
    t.cast(P0, s).x(3).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
}
