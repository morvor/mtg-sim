//! CR 701.9a, 404.3: the cards one instruction has a player discard are discarded at the
//! same time, so their owner arranges those that go to the same graveyard.

use crate::a701_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn cards_discarded_together_go_to_the_graveyard_in_the_order_their_owner_chooses() {
    cr!("701.9a", "404.3");
    supported("Mind Rot");
    let mut tops = vec![];
    for order in [vec![0usize, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        t.hand(P0, "Grizzly Bears");
        t.hand(P0, "Hill Giant");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Swamp", 3);
        let rot = t.hand(P1, "Mind Rot");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        let from = t.asked().len();
        t.cast(P1, rot).target(Entity::Player(P0)).go();
        t.resolve_all();
        // One choice of order, by the cards' owner.
        let orders: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::Order { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(orders, vec![P0]);
        let gy = &t.g.player(P0).graveyard;
        assert_eq!(gy.len(), 2);
        tops.push(t.g.obj(*gy.last().unwrap()).chars.name.clone());
    }
    // Either card can end up on top.
    assert_ne!(tops[0], tops[1]);
}
