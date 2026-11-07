//! Rulings batch P108 — Triskaidekaphobia ("At the beginning of your upkeep, choose one —
//! • Each player with exactly 13 life loses the game, then each player gains 1 life.
//! • Each player with exactly 13 life loses the game, then each player loses 1 life."):
//! the mode is chosen as the trigger is put on the stack (CR 700.2b); players who lose at
//! the same time make the game a draw if no one is left (CR 104.4a); in Two-Headed Giant
//! a team loses together and its shared life total changes for each player
//! (CR 810.8a, 810.9).

use crate::r_p108_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::game::GameResult;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Sets the life totals, puts P0's Triskaidekaphobia onto the battlefield, and advances to
/// P0's upkeep with `mode` chosen for its trigger; resolves it.
fn upkeep(t: &mut TestGame, lives: &[i32], mode: usize) {
    supported("Triskaidekaphobia");
    for (i, l) in lives.iter().enumerate() {
        t.g.players[i].life = *l;
    }
    t.battlefield(P0, "Triskaidekaphobia");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![mode]));
    let last = PlayerId(t.g.players.len() as u8 - 1);
    t.set_step(last, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn everyone_at_thirteen_is_a_draw() {
    cr!("104.4a", "104.3e");
    ruling!(
        "Triskaidekaphobia",
        "If each player has 13 life as Triskaidekaphobia’s ability resolves, the game ends in a draw."
    );
    let mut t = TestGame::new(2);
    upkeep(&mut t, &[13, 13], 0);
    assert!(matches!(t.g.result, Some(GameResult::Draw)));
}

#[test]
fn the_opponent_loses_before_you_lose_life() {
    cr!("104.3e", "608.2c", "104.2a");
    ruling!(
        "Triskaidekaphobia",
        "If you choose Triskaidekaphobia’s second mode and begin to resolve it while an opponent’s life total is 13 and your life total is 1, that opponent will lose the game before you lose 1 life."
    );
    let mut t = TestGame::new(2);
    upkeep(&mut t, &[1, 13], 1);
    assert!(t.has_lost(P1));
    assert!(!t.has_lost(P0));
    assert!(matches!(&t.g.result, Some(GameResult::Win(w)) if w == &vec![P0]));
}

#[test]
fn a_mode_is_chosen_even_if_no_one_loses() {
    cr!("700.2b", "603.3c");
    ruling!(
        "Triskaidekaphobia",
        "You choose one of the modes even if no player will lose the game. Players will still gain or lose life as appropriate."
    );
    let mut t = TestGame::new(3);
    upkeep(&mut t, &[20, 12, 14], 1);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (19, 11, 13));
    assert!(t.g.result.is_none());
    let mut t = TestGame::new(3);
    upkeep(&mut t, &[20, 12, 14], 0);
    assert_eq!((t.life(P0), t.life(P1), t.life(P2)), (21, 13, 15));
}

#[test]
fn in_two_headed_giant_teams_lose_and_life_changes_twice() {
    cr!("810.8a", "810.9");
    ruling!(
        "Triskaidekaphobia",
        "In a Two-Headed Giant game, each team with 13 life loses the game, then each player on each team gains or loses 1 life, causing the team’s life total to go up or down by 2."
    );
    // No team at 13: each team's life total goes up by 2.
    let mut t = two_headed_giant();
    upkeep(&mut t, &[20, 20, 25, 25], 0);
    assert_eq!((t.life(P0), t.life(P2)), (22, 27));
    assert_eq!((t.life(P1), t.life(P3)), (22, 27));
    // The opposing team at 13 loses.
    let mut t = two_headed_giant();
    upkeep(&mut t, &[20, 20, 13, 13], 1);
    assert!(t.has_lost(P2) && t.has_lost(P3));
    assert!(!t.has_lost(P0) && !t.has_lost(P1));
}
