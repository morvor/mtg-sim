//! Rulings batch P079 review — Osseous Sticktwister: "each opponent may sacrifice a
//! nonland permanent of their choice or discard a card. Then this creature deals damage
//! equal to its power to each opponent who didn't sacrifice a permanent or discard a card
//! this way." The either-or choice is planned with the other opponents' choices (CR 101.4)
//! like Aetherspouts' top-or-bottom choice (`simultaneous.rs`).

use crate::r_s01_common::supported;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Four players: P0's Osseous Sticktwister with delirium (a land, creature, instant and
/// sorcery card in P0's graveyard), in P0's precombat main phase.
fn sticktwister() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(4);
    let st = t.battlefield(P0, "Osseous Sticktwister");
    for c in ["Forest", "Grizzly Bears", "Lightning Bolt", "Divination"] {
        t.graveyard(P0, c);
    }
    t.advance_to(P0, Step::PrecombatMain);
    (t, st)
}

#[test]
fn sticktwister_each_opponent_chooses_in_turn_order_then_all_act() {
    cr!("101.4", "701.21a");
    ruling!(
        "Osseous Sticktwister",
        "the next opponent in turn order chooses a card to be discarded without revealing it, chooses a nonland permanent to be sacrificed, or chooses to do neither. Then each other opponent in turn order does the same."
    );
    supported("Osseous Sticktwister");
    let (mut t, _) = sticktwister();
    // P1 sacrifices its Grizzly Bears (not its Forest), P2 discards its Lightning Bolt,
    // P3 does neither.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P1, "Forest");
    let bolt = t.hand(P2, "Lightning Bolt");
    let p3_bears = t.battlefield(P3, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    t.answer_choose(P1, &[Entity::Object(bears)]);
    t.answer_yes(P2, true);
    t.answer(P2, DecisionKind::Option, Answer::Index(1));
    t.answer_choose(P2, &[Entity::Object(bolt)]);
    t.answer_yes(P3, false);
    let from = t.asked().len();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "P1 sacrificed the Bears");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(forest), "a land can't be chosen");
    assert!(t.in_graveyard(P2, "Lightning Bolt"));
    assert!(t.on_battlefield(p3_bears));
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (20, 20, 18));
    // Whether to do either is settled for all first; then the opponents choose in turn
    // order: all of P1's choices before P2's.
    let choosers: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. } | Decision::YesNo { .. }))
        .map(|(p, _)| *p)
        .filter(|p| *p != P0)
        .collect();
    let last_p1 = choosers.iter().rposition(|p| *p == P1).unwrap();
    let first_p2 = choosers.iter().position(|p| *p == P2).unwrap();
    assert!(last_p1 < first_p2, "{choosers:?}");
}

#[test]
fn sticktwister_uses_its_last_known_power() {
    cr!("608.2h");
    ruling!(
        "Osseous Sticktwister",
        "If Osseous Sticktwister is no longer on the battlefield at that time, use its power as it last existed on the battlefield."
    );
    let (mut t, st) = sticktwister();
    for p in [P1, P2, P3] {
        t.answer_yes(p, false);
    }
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.destroy(st, None);
    t.resolve_all();
    assert!(!t.on_battlefield(st));
    assert_eq!((t.life(P1), t.life(P2), t.life(P3)), (18, 18, 18));
}
