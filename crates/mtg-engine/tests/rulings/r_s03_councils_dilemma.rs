//! Rulings batch S03 — council's dilemma (an ability word; votes, CR 701.38):
//! "Council's dilemma — [When this creature enters,] starting with you, each player votes
//! for [A] or [B]. [Effect] for each [A] vote and [effect] for each [B] vote."

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::game::Game;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const LIEUTENANTS: &str = "Lieutenants of the Guard";
const ORCHARD: &str = "Orchard Elemental";

/// `p` will vote for option `i`.
fn vote(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

/// The votes asked so far: (player, options).
fn votes_asked(t: &TestGame) -> Vec<(PlayerId, Vec<String>)> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Vote" => Some((p, options)),
            _ => None,
        })
        .collect()
}

/// The votes cast so far, as the game reported them (`Event::Custom` "vote").
fn votes_reported(g: &Game) -> usize {
    g.turn_events
        .iter()
        .chain(g.events.iter())
        .filter(|e| matches!(e, Event::Custom { name, .. } if name == mtg_engine::kwa::vote::VOTED))
        .count()
}

/// Lieutenants of the Guard ("... votes for strength or numbers. Put a +1/+1 counter on
/// this creature for each strength vote and create a 1/1 white Soldier creature token for
/// each numbers vote.") enters under P0's control in a three-player game, with these
/// votes (0 = strength, 1 = numbers) for P0, P1, P2.
fn lieutenants(votes: [usize; 3]) -> (TestGame, ObjectId) {
    supported(LIEUTENANTS);
    let mut t = TestGame::new(3);
    for (p, v) in [P0, P1, P2].into_iter().zip(votes) {
        vote(&mut t, p, v);
    }
    let lt = t.enter(P0, LIEUTENANTS);
    (t, lt)
}

#[test]
fn votes_are_cast_in_turn_order_after_the_previous_votes_are_known() {
    cr!("701.38a", "101.4");
    ruling!(
        "Lieutenants of the Guard",
        "Because the votes are made in turn order, each player will know the votes of players who voted beforehand."
    );
    // P1 controls the ability: P1 votes first, then P2, then P0 (turn order).
    supported(LIEUTENANTS);
    let mut t = TestGame::new(3);
    let seen: Vec<_> = [P0, P1, P2]
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
    t.enter(P1, LIEUTENANTS);
    t.resolve_all();
    let order: Vec<PlayerId> = votes_asked(&t).into_iter().map(|(p, _)| p).collect();
    assert_eq!(order, vec![P1, P2, P0]);
    // When each player voted, the votes of those before them had been cast and reported.
    assert_eq!(seen[1].lock().unwrap().clone(), vec![0]);
    assert_eq!(seen[2].lock().unwrap().clone(), vec![1]);
    assert_eq!(seen[0].lock().unwrap().clone(), vec![2]);
    assert!(t.dump_log().contains("votes for"));
}

#[test]
fn the_vote_comes_first_then_the_first_effect_then_the_second() {
    cr!("608.2c", "701.38a");
    ruling!(
        "Orchard Elemental",
        "The effects of each council’s dilemma ability happen in the stated order. First the vote occurs, then the first effect, and finally the second effect."
    );
    supported(ORCHARD);
    // Orchard Elemental: "... votes for sprout or harvest. Put two +1/+1 counters on this
    // creature for each sprout vote. You gain 3 life for each harvest vote."
    let mut t = TestGame::new(3);
    vote(&mut t, P0, 1);
    vote(&mut t, P1, 0);
    vote(&mut t, P2, 1);
    let before = t.g.turn_events.len();
    let orchard = t.enter(P0, ORCHARD);
    t.resolve_all();
    let order: Vec<&str> = t.g.turn_events[before..]
        .iter()
        .filter_map(|e| match e {
            Event::Custom { name, .. } if name == mtg_engine::kwa::vote::VOTED => Some("vote"),
            Event::CountersAdded { .. } => Some("counters"),
            Event::LifeGained { .. } => Some("life"),
            _ => None,
        })
        .collect();
    assert_eq!(order, vec!["vote", "vote", "vote", "counters", "life"]);
    assert_eq!(t.counters(orchard, "+1/+1"), 2);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn nobody_can_act_between_the_vote_and_the_end_of_the_resolution() {
    cr!("608.2c", "117.3b", "117.4");
    ruling!(
        "Lieutenants of the Guard",
        "Players can’t do anything between voting and finishing the resolution of the spell or ability that included the vote."
    );
    supported(LIEUTENANTS);
    // P0 casts Lieutenants of the Guard in the game's priority loop; everyone passes and
    // its trigger resolves: P0 votes strength, P1 and P2 numbers.
    let mut t = TestGame::new(3);
    let lt = t.hand(P0, LIEUTENANTS);
    t.lands(P0, "Plains", 5);
    let seen: Vec<_> = [P0, P1, P2]
        .into_iter()
        .map(|p| {
            watch(
                &mut t,
                p,
                |d| matches!(d, Decision::Priority { .. }),
                |g: &Game| {
                    let lt = g.find_in_zone(Zone::Battlefield, "Lieutenants of the Guard");
                    (
                        votes_reported(g),
                        lt.first().map_or(0, |o| g.obj(*o).counter("+1/+1")),
                        g.permanents().filter(|o| o.is_token()).count(),
                    )
                },
            )
        })
        .collect();
    t.answer(
        P0,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: lt,
            method: CastMethod::Normal,
        }),
    );
    vote(&mut t, P0, 0);
    vote(&mut t, P1, 1);
    vote(&mut t, P2, 1);
    assert!(t.g.run_until(10_000, |g| g.turn.step == Step::BeginningOfCombat));
    assert_eq!(tokens(&t, P0).len(), 2);
    // The three votes were asked one after another, with no priority in between, and
    // every time a player had priority the ability had either not begun resolving or
    // had fully resolved.
    let asked = t.asked();
    let votes: Vec<usize> = asked
        .iter()
        .enumerate()
        .filter(|(_, (_, d))| matches!(d, Decision::ChooseOption { prompt, .. } if prompt == "Vote"))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(votes.len(), 3);
    assert_eq!(votes[2] - votes[0], 2);
    for s in &seen {
        for state in s.lock().unwrap().iter() {
            assert!(
                *state == (0, 0, 0) || *state == (3, 1, 2),
                "priority during the vote's resolution: {state:?}"
            );
        }
    }
    assert!(seen[1].lock().unwrap().contains(&(3, 1, 2)));
}

#[test]
fn players_can_vote_for_counters_on_a_creature_that_has_left_the_battlefield() {
    cr!("701.38a", "608.2h");
    ruling!(
        "Lieutenants of the Guard",
        "If a creature with an enters-the-battlefield council’s dilemma ability leaves the battlefield before that ability resolves, players can still vote for any option that would put +1/+1 counters on that creature, even though—or perhaps especially because—those votes won’t generate an effect."
    );
    // P0 and P1 vote strength, P2 numbers.
    let (mut t, lt) = lieutenants([0, 0, 1]);
    t.settle();
    destroy(&mut t, lt);
    assert!(t.in_graveyard(P0, LIEUTENANTS));
    t.resolve_all();
    let asked = votes_asked(&t);
    assert_eq!(asked.len(), 3);
    for (_, options) in &asked {
        assert_eq!(options, &vec!["strength".to_string(), "numbers".to_string()]);
    }
    // The strength votes did nothing; the numbers vote made a Soldier.
    assert_eq!(tokens(&t, P0).len(), 1);
    assert!(t.g.permanents().all(|o| o.counter("+1/+1") == 0));
}

#[test]
fn each_vote_adds_to_the_effect() {
    cr!("701.38a");
    ruling!(
        "Lieutenants of the Guard",
        "Unlike the will of the council cards from the original Conspiracy set, where a majority of votes determined what happened, each vote made for a council’s dilemma card adds to the ultimate effect."
    );
    // Two strength votes and one numbers vote: two counters and a Soldier.
    let (mut t, lt) = lieutenants([0, 1, 0]);
    t.resolve_all();
    assert_eq!(t.counters(lt, "+1/+1"), 2);
    assert_eq!(tokens(&t, P0).len(), 1);
    // Orchard Elemental: one sprout vote and two harvest votes.
    let mut t = TestGame::new(3);
    vote(&mut t, P0, 0);
    vote(&mut t, P1, 1);
    vote(&mut t, P2, 1);
    let orchard = t.enter(P0, ORCHARD);
    t.resolve_all();
    assert_eq!(t.counters(orchard, "+1/+1"), 2);
    assert_eq!(t.life(P0), 26);
}

#[test]
fn every_player_votes_for_one_of_the_options() {
    cr!("701.38a");
    ruling!(
        "Lieutenants of the Guard",
        "You must vote for one of the available options. You can’t abstain."
    );
    // P1 gives no valid answer: they still vote (for strength).
    supported(LIEUTENANTS);
    let mut t = TestGame::new(3);
    vote(&mut t, P0, 1);
    t.answer(P1, DecisionKind::Option, Answer::Bool(false));
    vote(&mut t, P2, 1);
    let lt = t.enter(P0, LIEUTENANTS);
    t.resolve_all();
    let asked = votes_asked(&t);
    assert_eq!(asked.len(), 3);
    // Only the two options are offered.
    assert!(asked.iter().all(|(_, o)| o.len() == 2));
    assert_eq!(t.counters(lt, "+1/+1"), 1);
    assert_eq!(tokens(&t, P0).len(), 2);
    assert_eq!(votes_reported(&t.g), 3);
}

#[test]
fn fateful_tempest_votes_in_turn_order_and_adds_up_each_vote() {
    cr!("701.38a", "701.17a", "608.2c");
    ruling!(
        "Fateful Tempest",
        "Votes are cast in turn order, and each player will know the votes of players who voted beforehand."
    );
    supported("Fateful Tempest");
    // Fateful Tempest: "Council's dilemma — Starting with you, each player votes for past
    // or present. You mill a card for each past vote, then Fateful Tempest deals damage to
    // each opponent equal to the total mana value of cards milled this way. Exile the top
    // card of your library for each present vote. Until the end of your next turn, you
    // may play the exiled cards."
    let mut t = TestGame::new(3);
    stack_library(&mut t, P0, &["Hill Giant", "Grizzly Bears", "Lightning Bolt"]);
    let seen: Vec<_> = [P0, P1, P2]
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
    // P0 and P2 vote past, P1 present.
    vote(&mut t, P0, 0);
    vote(&mut t, P1, 1);
    vote(&mut t, P2, 0);
    let c = in_hand_with_mana(&mut t, P0, "Fateful Tempest");
    t.cast(P0, c).go();
    t.resolve_all();
    let order: Vec<PlayerId> = votes_asked(&t).into_iter().map(|(p, _)| p).collect();
    assert_eq!(order, vec![P0, P1, P2]);
    assert_eq!(seen[0].lock().unwrap().clone(), vec![0]);
    assert_eq!(seen[1].lock().unwrap().clone(), vec![1]);
    assert_eq!(seen[2].lock().unwrap().clone(), vec![2]);
    // Two cards milled (Hill Giant and Grizzly Bears: total mana value 6) ...
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P2), 14);
    assert_eq!(t.life(P0), 20);
    // ... and one exiled, which P0 may play.
    assert!(t.in_exile("Lightning Bolt"));
    let bolt = t.g.find_in_zone(Zone::Exile, "Lightning Bolt")[0];
    t.lands(P0, "Mountain", 1);
    assert!(crate::r_s02_common::can_cast(
        &mut t,
        P0,
        bolt,
        CastMethod::Normal
    ));
}
