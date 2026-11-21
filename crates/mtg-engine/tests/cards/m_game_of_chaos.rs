//! Game of Chaos (hand-written, `src/cards/game_of_chaos.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_stakes_double_and_the_winner_decides() {
    cr!("705.1");
    ruling!("Game of Chaos", "To “double the life stakes” means to double the amount of life lost or gained");
    let mut t = TestGame::new(2);
    // Won (heads called and flipped), then lost.
    t.g.dice.loaded_coins.extend([true, false]);
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Game of Chaos");
    t.answer_yes(P0, true);
    t.answer_yes(P1, false);
    t.cast(P0, s).target(P1).go();
    t.resolve();
    // +1/-1, then -2/+2.
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 21);
}

#[test]
fn stops_when_the_winner_declines() {
    cr!("705.1");
    let mut t = TestGame::new(2);
    t.g.dice.loaded_coins.extend([true]);
    t.lands(P0, "Mountain", 3);
    let s = t.hand(P0, "Game of Chaos");
    t.answer_yes(P0, false);
    t.cast(P0, s).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 19);
}
