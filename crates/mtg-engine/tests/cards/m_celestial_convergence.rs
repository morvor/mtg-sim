//! Celestial Convergence (hand-written, `src/cards/celestial_convergence.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn convergence_with_one_counter(t: &mut TestGame) {
    let c = t.battlefield(P0, "Celestial Convergence");
    t.g.objects[c.0 as usize].counters.insert("omen".into(), 1);
    t.g.turn.number = 1;
    t.set_step(P1, Step::End);
}

#[test]
fn highest_life_total_wins_when_the_last_counter_is_removed() {
    cr!("104.2b", "603.2");
    let mut t = TestGame::new(2);
    convergence_with_one_counter(&mut t);
    t.g.players[1].life = 25;
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P1])));
}

#[test]
fn a_tie_for_highest_is_a_draw() {
    cr!("104.4a");
    let mut t = TestGame::new(2);
    convergence_with_one_counter(&mut t);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

#[test]
fn nothing_happens_while_counters_remain() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Celestial Convergence");
    t.g.objects[c.0 as usize].counters.insert("omen".into(), 2);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.g.result.is_none());
    assert_eq!(t.counters(c, "omen"), 1);
}
