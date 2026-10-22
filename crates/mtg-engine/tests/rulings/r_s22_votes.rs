//! Rulings batch S22 — Emissary Green ("Whenever Emissary Green attacks, starting with
//! you, each player votes for profit or security. You create a number of Treasure tokens
//! equal to twice the number of profit votes. Put a number of +1/+1 counters on each
//! creature you control equal to the number of security votes."): a vote in turn order
//! (CR 701.38a), each player knowing the earlier votes.

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::Game;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The votes cast so far, as the game reported them.
fn votes_reported(g: &Game) -> usize {
    g.turn_events
        .iter()
        .chain(g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name, .. } if name == mtg_engine::kwa::vote::VOTED))
        .count()
}

fn treasures(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Treasure"))
        .count()
}

#[test]
fn emissary_green_votes_are_cast_in_turn_order_knowing_the_earlier_votes() {
    cr!("701.38a", "608.2c");
    ruling!(
        "Emissary Green",
        "In the case of non-secret voting, votes are cast in turn order, and each player will know the votes of players who voted beforehand."
    );
    supported("Emissary Green");
    // Three players; P0's Emissary Green attacks P1. P0 and P2 vote profit (0), P1
    // security (1).
    let mut t = TestGame::new(3);
    let eg = t.battlefield(P0, "Emissary Green");
    let seen: Vec<_> = [P0, P1, PlayerId(2)]
        .into_iter()
        .map(|p| {
            watch(
                &mut t,
                p,
                |d| matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Vote"),
                votes_reported,
            )
        })
        .collect();
    for (p, v) in [(P0, 0), (P1, 1), (PlayerId(2), 0)] {
        t.answer(p, DecisionKind::Option, Answer::Index(v));
    }
    attack_with(&mut t, &[(eg, Entity::Player(P1))]);
    t.resolve_all();
    let order: Vec<PlayerId> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption { prompt, .. } if prompt == "Vote" => Some(p),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec![P0, P1, PlayerId(2)]);
    // When each player voted, the votes before theirs had been cast and reported.
    assert_eq!(seen[0].lock().unwrap().clone(), vec![0]);
    assert_eq!(seen[1].lock().unwrap().clone(), vec![1]);
    assert_eq!(seen[2].lock().unwrap().clone(), vec![2]);
    // Two profit votes: four Treasures; one security vote: a +1/+1 counter.
    assert_eq!(treasures(&t, P0), 4);
    assert_eq!(t.counters(eg, "+1/+1"), 1);
}
