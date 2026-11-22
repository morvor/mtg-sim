//! CR 702.19b: lethal damage for trample counts damage other creatures are assigning in
//! the same combat damage step, whatever order they were declared in; the attacking player
//! orders the assignments of tramplers that share a blocker.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn trampler_declared_first_counts_a_later_attackers_damage() {
    cr!("702.19b");
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let guard = t.battlefield(P1, "Palace Guard");
    // The Dreadmaw is declared before the bears; both are blocked by the 1/4 guard.
    attack_with(
        &mut t,
        &[(maw, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(guard, maw), (guard, bears)]);
    // The bears' 2 damage counts: 2 more from the Dreadmaw is lethal, 4 trample over.
    assert!(!t.on_battlefield(guard));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn attacking_player_orders_tramplers_sharing_a_blocker() {
    cr!("702.19b");
    let mut t = TestGame::new(2);
    let maw1 = t.battlefield(P0, "Colossal Dreadmaw");
    let maw2 = t.battlefield(P0, "Colossal Dreadmaw");
    let guard = t.battlefield(P1, "Palace Guard");
    // The second Dreadmaw assigns first: lethal 4 to the guard, 2 to the player; then the
    // first Dreadmaw may assign all 6 to the player.
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    assign_damage(&mut t, P0, &[4, 2]);
    assign_damage(&mut t, P0, &[0, 6]);
    attack_with(
        &mut t,
        &[(maw1, Entity::Player(P1)), (maw2, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(guard, maw1), (guard, maw2)]);
    let d: Vec<_> = damage_decisions(&t)
        .into_iter()
        .filter(|(p, _, _)| *p == P0)
        .collect();
    assert_eq!(d.len(), 2);
    assert_eq!(d[0].1, maw2, "assigned in the chosen order");
    assert_eq!(d[1].1, maw1);
    assert!(!t.on_battlefield(guard));
    assert_eq!(t.life(P1), 12);
}

#[test]
fn no_order_is_asked_for_tramplers_without_shared_blockers() {
    cr!("702.19b");
    let mut t = TestGame::new(2);
    let maw1 = t.battlefield(P0, "Colossal Dreadmaw");
    let maw2 = t.battlefield(P0, "Colossal Dreadmaw");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Hill Giant");
    attack_with(
        &mut t,
        &[(maw1, Entity::Player(P1)), (maw2, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[(b1, maw1), (b2, maw2)]);
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::Order { prompt, .. } if prompt.contains("trample"))));
    assert_eq!(t.life(P1), 20 - 4 - 3);
}
