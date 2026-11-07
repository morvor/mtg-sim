//! Rulings batch S15 — secret council (secret votes, CR 701.38): Trap the Trespassers,
//! with Council's Judgment's open vote for comparison.

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s08_common::is_tapped;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::kwa::vote::VOTED;
use mtg_engine::testing::*;
use mtg_engine::*;

fn is_vote(d: &Decision) -> bool {
    matches!(d, Decision::ChooseEntities { prompt, .. } if prompt == "Vote")
}

/// The number of votes revealed (reported) so far this turn, including those reported
/// by the resolving spell.
fn votes_revealed(g: &mtg_engine::game::Game) -> usize {
    g.turn_events
        .iter()
        .chain(g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name, .. } if name == VOTED))
        .count()
}

#[test]
fn secret_votes_stay_secret_until_all_are_revealed_together() {
    cr!("701.38a", "701.38b");
    ruling!(
        "Trap the Trespassers",
        "To secretly vote, each player writes down their chosen option without showing it to anyone else. Each player then keeps their vote secret until all players simultaneously reveal their votes."
    );
    ruling!(
        "Trap the Trespassers",
        "Before secret votes are revealed, players may announce how they intend to vote, but they can't reveal what they actually wrote down until all votes are simultaneously revealed."
    );
    supported("Trap the Trespassers");
    // Trap the Trespassers: "Secret council — Each player secretly votes for a creature
    // you don't control, then those votes are revealed. For each creature with one or
    // more votes, put that many stun counters on it, then tap it."
    let mut t = TestGame::new(3);
    let mine = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P2, "Hill Giant");
    let seen: Vec<_> = [P0, P1, P2]
        .into_iter()
        .map(|p| watch(&mut t, p, is_vote, votes_revealed))
        .collect();
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    t.answer_choose(P2, &[Entity::Object(bears)]);
    let from = t.asked().len();
    let trap = in_hand_with_mana(&mut t, P0, "Trap the Trespassers");
    t.cast(P0, trap).go();
    t.resolve_all();
    // No player saw any vote while voting; then all three were revealed.
    for s in &seen {
        assert_eq!(*s.lock().unwrap(), vec![0]);
    }
    assert_eq!(votes_revealed(&t.g), 3);
    // Only creatures P0 doesn't control could be voted for.
    let offered: Vec<Vec<Entity>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt == "Vote" => Some(candidates.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 3);
    assert!(offered.iter().all(|c| !c.contains(&Entity::Object(mine))));
    // Two votes: two stun counters; one vote: one.
    assert_eq!(t.counters(bears, "stun"), 2);
    assert_eq!(t.counters(giant, "stun"), 1);
    assert!(is_tapped(&t, bears) && is_tapped(&t, giant));
    assert_eq!(t.counters(mine, "stun"), 0);
    assert!(!is_tapped(&t, mine));
}

#[test]
fn an_open_vote_is_seen_by_the_players_who_vote_after() {
    cr!("701.38a");
    supported("Council's Judgment");
    // Council's Judgment: "Will of the council — Starting with you, each player votes for
    // a nonland permanent you don't control. Exile each permanent with the most votes or
    // tied for most votes." Each vote is known as it's cast.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Hill Giant");
    let seen: Vec<_> = [P0, P1, P2]
        .into_iter()
        .map(|p| watch(&mut t, p, is_vote, votes_revealed))
        .collect();
    for p in [P0, P1, P2] {
        t.answer_choose(p, &[Entity::Object(bears)]);
    }
    let cj = in_hand_with_mana(&mut t, P0, "Council's Judgment");
    t.cast(P0, cj).go();
    t.resolve_all();
    let seen: Vec<Vec<usize>> = seen.iter().map(|s| s.lock().unwrap().clone()).collect();
    assert_eq!(seen, vec![vec![0], vec![1], vec![2]]);
    assert!(t.in_exile("Grizzly Bears"));
}
