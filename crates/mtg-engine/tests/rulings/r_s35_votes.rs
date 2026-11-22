//! Rulings batch S35 — voting (CR 701.38): votes are cast as the spell or ability
//! resolves, and each player votes for one of the options.

use crate::r_s01_common::{attack_with, supported, watch, with_subtype};
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

#[test]
fn no_one_votes_until_the_spell_resolves() {
    cr!("701.38a", "608.2");
    ruling!(
        "Council's Judgment",
        "No player votes until the spell or ability resolves. Any responses to that spell or ability must be made without knowing the outcome of the vote."
    );
    supported("Council's Judgment");
    // "Will of the council — Starting with you, each player votes for a nonland permanent
    // you don't control. Exile each permanent with the most votes or tied for most votes."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    // Whenever P1 gets priority, the votes cast so far and whether the spell is on the
    // stack.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g| (votes_reported(g), g.stack.len()),
    );
    let judgment = t.hand(P0, "Council's Judgment");
    t.cast(P0, judgment).go();
    assert_eq!(votes_reported(&t.g), 0);
    assert!(t
        .g
        .run_until(1000, |g| g.stack.is_empty() && !g.is_live(bears)));
    // P1 had priority with the spell on the stack and no votes cast.
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.first(), Some(&(0, 1)), "{seen:?}");
    // Both players voted as it resolved; the Bears got the votes.
    assert_eq!(votes_reported(&t.g), 2);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn each_player_must_vote_for_one_of_the_options() {
    cr!("701.38a");
    ruling!(
        "Emissary Green",
        "Each player must vote for one of the available options. They can't abstain."
    );
    supported("Emissary Green");
    // "Whenever Emissary Green attacks, starting with you, each player votes for profit or
    // security. You create a number of Treasure tokens equal to twice the number of profit
    // votes. Put a number of +1/+1 counters on each creature you control equal to the
    // number of security votes." P0 votes profit; P1 tries not to vote (an answer that
    // isn't an option): P1 still votes for one of them.
    let mut t = TestGame::new(2);
    let eg = t.battlefield(P0, "Emissary Green");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(7));
    attack_with(&mut t, &[(eg, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(votes_reported(&t.g), 2);
    let profit = with_subtype(&t, P0, "Treasure").len() / 2;
    let security = t.counters(eg, "+1/+1") as usize;
    assert!(profit >= 1);
    assert_eq!(profit + security, 2);
    // Each player was asked to vote between exactly the two options.
    let votes: Vec<(PlayerId, usize)> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption {
                prompt, options, ..
            } if prompt == "Vote" => Some((p, options.len())),
            _ => None,
        })
        .collect();
    assert_eq!(votes, vec![(P0, 2), (P1, 2)]);
}
