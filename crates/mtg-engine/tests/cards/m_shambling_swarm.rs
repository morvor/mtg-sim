//! Shambling Swarm (hand-written, `src/cards/shambling_swarm.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn the_counters_are_removed_at_the_next_end_step() {
    cr!("601.2d", "603.7");
    ruling!("Shambling Swarm", "don’t have to be the same counters");
    let mut t = TestGame::new(2);
    let swarm = t.battlefield(P0, "Shambling Swarm");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(wurm), Entity::Object(giant)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.g.destroy(swarm, None);
    t.resolve_all();
    assert_eq!(t.counters(wurm, "-1/-1"), 2);
    assert_eq!(t.counters(giant, "-1/-1"), 1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(wurm, "-1/-1"), 0);
    assert_eq!(t.counters(giant, "-1/-1"), 0);
    assert_eq!(t.pt(wurm), (6, 4));
}
