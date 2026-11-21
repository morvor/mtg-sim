//! CR 701.22: scry.

use crate::a701_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn scry_askers(t: &TestGame) -> Vec<PlayerId> {
    t.asked()
        .iter()
        .filter_map(|(p, d)| matches!(d, Decision::Scry { .. }).then_some(*p))
        .collect()
}

#[test]
fn players_scrying_at_once_look_at_the_same_time_and_decide_in_apnap_order() {
    cr!("701.22c");
    ruling!(
        "Eager Construct",
        "those players each look at the top card of their library at the same time"
    );
    supported("Eager Construct");
    let mut t = TestGame::new(2);
    let mine = t.library_top(P0, "Island");
    let theirs = t.library_top(P1, "Forest");
    // P1 decides while P0's card is still on top of P0's library: nothing has moved yet.
    let log = spy(&mut t, P1, move |g, _p, d| match d {
        Decision::Scry { .. } => Some(format!("{:?}", g.player(P0).library.last() == Some(&mine))),
        _ => None,
    });
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.answer(P0, DecisionKind::Scry, Answer::Split(vec![], vec![mine]));
    t.answer(P1, DecisionKind::Scry, Answer::Split(vec![theirs], vec![]));
    // "When this creature enters, each player may scry 1."
    t.enter(P0, "Eager Construct");
    t.resolve_all();
    assert_eq!(scry_askers(&t), vec![P0, P1]);
    assert_eq!(probe_lines(&log), vec!["true".to_string()]);
    // Then the cards moved.
    assert_eq!(t.g.player(P0).library.first(), Some(&mine));
    assert_eq!(t.g.player(P1).library.last(), Some(&theirs));
}

#[test]
fn a_player_who_doesnt_scry_isnt_asked_and_doesnt_scry() {
    cr!("701.22c", "701.22d");
    supported("Eager Construct");
    supported("Chance-Met Elves");
    let mut t = TestGame::new(2);
    // "Whenever you scry or surveil, put a +1/+1 counter on this creature."
    let elves = t.battlefield(P1, "Chance-Met Elves");
    let mine = t.battlefield(P0, "Chance-Met Elves");
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    t.enter(P0, "Eager Construct");
    t.resolve_all();
    assert_eq!(scry_askers(&t), vec![P1]);
    assert_eq!(t.counters(elves, "+1/+1"), 1);
    assert_eq!(t.counters(mine, "+1/+1"), 0);
}

#[test]
fn scrying_with_an_empty_library_is_still_scrying() {
    cr!("701.22d");
    supported("Chance-Met Elves");
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P0, "Chance-Met Elves");
    clear_library(&mut t, P0);
    run(
        &mut t,
        P0,
        None,
        Effect::Scry {
            who: PlayerRef::You,
            n: Value::c(2),
        },
    );
    t.resolve_all();
    assert_eq!(t.counters(elves, "+1/+1"), 1);
}
