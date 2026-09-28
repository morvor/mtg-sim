//! Rulings batch S18 — will of the council: "Starting with you, each player votes for
//! [option] or [option]. ..." (CR 701.38).

use crate::r_s01_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::kwa::vote::VOTED;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The players who voted, in order.
fn voters(g: &mtg_engine::game::Game) -> Vec<PlayerId> {
    g.turn_events
        .iter()
        .chain(g.events.iter())
        .filter_map(|e| match e {
            Event::Custom {
                name, player: Some(p), ..
            } if name.as_str() == VOTED => Some(*p),
            _ => None,
        })
        .collect()
}

#[test]
fn every_player_must_vote_for_one_of_the_options() {
    cr!("701.38a");
    ruling!(
        "Tyrant's Choice",
        "You must vote for one of the available options. You can't abstain."
    );
    supported("Tyrant's Choice");
    // Tyrant's Choice: "each player votes for death or torture. If death gets more votes,
    // each opponent sacrifices a creature of their choice. If torture gets more votes or
    // the vote is tied, each opponent loses 4 life." P0 votes torture; P1 tries not to
    // vote, and is counted as voting for death: a tie.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Tyrant's Choice");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P1, DecisionKind::Option, Answer::Default);
    let spell = t.hand(P0, "Tyrant's Choice");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(voters(&t.g), vec![P0, P1]);
    let options: Vec<(PlayerId, Vec<String>)> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption { options, .. } => Some((p, options)),
            _ => None,
        })
        .collect();
    let both = vec!["death".to_string(), "torture".to_string()];
    assert_eq!(options, vec![(P0, both.clone()), (P1, both)]);
    assert_eq!(t.life(P1), 16);
    assert!(t.on_battlefield(bears));
}

#[test]
fn no_one_can_act_between_the_votes_and_the_end_of_the_resolution() {
    cr!("701.38a", "608.2", "117.3");
    ruling!(
        "Bite of the Black Rose",
        "Players can't do anything after they finish voting but before the spell or ability that included the vote finishes resolving."
    );
    supported("Bite of the Black Rose");
    // Bite of the Black Rose: sickness (creatures P0's opponents control get -2/-2 until
    // end of turn) or psychosis (each opponent discards two cards). Both vote sickness.
    // Each time a player gets priority, record whether the spell is on the stack and how
    // many votes have been cast: never the spell with votes cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Bite of the Black Rose");
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P1, DecisionKind::Option, Answer::Index(0));
    let spell = t.hand(P0, "Bite of the Black Rose");
    t.cast(P0, spell).go();
    let look = |g: &mtg_engine::game::Game| (g.stack.len(), voters(g).len());
    let is_priority = |d: &Decision| matches!(d, Decision::Priority { .. });
    let p0 = watch(&mut t, P0, is_priority, look);
    let p1 = watch(&mut t, P1, is_priority, look);
    assert!(t.g.run_until(1000, |g| g.stack.is_empty() && voters(g).len() == 2));
    t.settle();
    let mut seen = p0.lock().unwrap().clone();
    seen.extend(p1.lock().unwrap().iter().copied());
    assert!(seen.contains(&(1, 0)), "{seen:?}");
    assert!(!seen.iter().any(|(stack, votes)| *stack > 0 && *votes > 0), "{seen:?}");
    assert!(!t.on_battlefield(bears));
}
