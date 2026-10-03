//! CR 701.38: vote.

use crate::a701_028_071_common::*;
use mtg_engine::kwa::vote::VOTED;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn voters(t: &TestGame) -> Vec<PlayerId> {
    custom_events(t, VOTED)
        .into_iter()
        .filter_map(|(p, _, _)| p)
        .collect()
}

/// Casts Plea for Power for P0: "Will of the council — Starting with you, each player
/// votes for time or knowledge. If time gets more votes, take an extra turn after this
/// one. If knowledge gets more votes or the vote is tied, draw three cards."
fn plea(t: &mut TestGame) {
    supported("Plea for Power");
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Plea for Power");
    t.cast(P0, spell).go();
    t.resolve_all();
}

#[test]
fn each_player_votes_in_turn_order_starting_with_the_specified_player() {
    cr!("701.38a");
    ruling!(
        "Plea for Power",
        "Because the votes are cast in turn order, each player will know the votes of players who voted beforehand."
    );
    // Options: 0 = time, 1 = knowledge.
    let mut t = TestGame::new(3);
    option(&mut t, P0, 0);
    option(&mut t, P1, 1);
    option(&mut t, P2, 0);
    let hand = t.hand_size(P0);
    plea(&mut t);
    assert_eq!(voters(&t), vec![P0, P1, P2]);
    // Time got more votes: an extra turn, no cards.
    assert_eq!(t.g.extra_turns, vec![P0]);
    assert_eq!(t.hand_size(P0), hand);
    // Knowledge got more votes: draw three.
    let mut t = TestGame::new(3);
    option(&mut t, P0, 0);
    option(&mut t, P1, 1);
    option(&mut t, P2, 1);
    let hand = t.hand_size(P0);
    plea(&mut t);
    assert!(t.g.extra_turns.is_empty());
    assert_eq!(t.hand_size(P0), hand + 3);
    // Starting with another player than the active player: P2, then P0, then P1.
    let mut t = TestGame::new(3);
    let mut spec = mtg_engine::kwa::Spec::new(
        mtg_engine::ability::KeywordAction::Vote,
        mtg_engine::ability::PlayerRef::You,
        mtg_engine::ability::Sel::None,
        mtg_engine::ability::Value::c(1),
    );
    spec.options = vec![
        ("time".into(), mtg_engine::ability::Effect::Noop),
        ("knowledge".into(), mtg_engine::ability::Effect::Noop),
    ];
    run(&mut t, P2, None, spec.effect(), &[]);
    assert_eq!(voters(&t), vec![P2, P0, P1]);
}

#[test]
fn a_tied_vote() {
    cr!("701.38a");
    ruling!(
        "Plea for Power",
        "The phrase \"the vote is tied\" refers only to when there is more than one choice that received the most votes."
    );
    let mut t = TestGame::new(2);
    option(&mut t, P0, 0);
    option(&mut t, P1, 1);
    let hand = t.hand_size(P0);
    plea(&mut t);
    assert!(t.g.extra_turns.is_empty());
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn players_can_vote_for_objects() {
    cr!("701.38b");
    ruling!(
        "Council's Judgment",
        "None of the candidate permanents are targeted. Players may vote for a permanent with protection from white, for example."
    );
    supported("Council's Judgment");
    // "Starting with you, each player votes for a nonland permanent you don't control.
    // Exile each permanent with the most votes or tied for most votes."
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    // Protection from white doesn't stop votes.
    let c = t.battlefield(P2, "White Knight");
    let mine = t.battlefield(P0, "Llanowar Elves");
    t.lands(P0, "Plains", 3);
    choose(&mut t, P0, &[c]);
    choose(&mut t, P1, &[c]);
    choose(&mut t, P2, &[a]);
    let spell = t.hand(P0, "Council's Judgment");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Exile);
    assert!(t.on_battlefield(a) && t.on_battlefield(b) && t.on_battlefield(mine));
    // The candidates: nonland permanents P0 doesn't control.
    let cands: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(cands.len(), 3);
    assert!(cands.iter().all(|c| !c.contains(&Entity::Object(mine))));
    // A tie: each permanent tied for most votes is exiled.
    let mut t = TestGame::new(3);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Hill Giant");
    t.lands(P0, "Plains", 3);
    choose(&mut t, P0, &[a]);
    choose(&mut t, P1, &[b]);
    choose(&mut t, P2, &[a]);
    let spell = t.hand(P0, "Council's Judgment");
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert!(t.on_battlefield(b));
}

#[test]
fn voting_refers_only_to_actual_votes() {
    cr!("701.38c");
    ruling!(
        "Brago's Representative",
        "The ability only affects spells and abilities that use the word “vote.” Other cards that involve choices, such as Archangel of Strife, are unaffected."
    );
    supported("Brago's Representative");
    // "While voting, you get an additional vote."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Brago's Representative");
    option(&mut t, P0, 0);
    option(&mut t, P1, 1);
    option(&mut t, P1, 1);
    let hand = t.hand_size(P0);
    plea(&mut t);
    assert_eq!(voters(&t), vec![P0, P1, P1]);
    assert_eq!(t.hand_size(P0), hand + 3);
    // A villainous choice isn't a vote: no additional choice.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Brago's Representative");
    t.battlefield(P0, "The Dalek Emperor");
    t.advance_to(P0, mtg_engine::turn::Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(
        custom_events(&t, mtg_engine::kwa::villainous::FACED).len(),
        1
    );
    assert!(voters(&t).is_empty());
}

#[test]
fn a_players_multiple_votes_happen_when_they_would_vote() {
    cr!("701.38d");
    ruling!(
        "Brago's Representative",
        "You make all your votes at the same time. Players who vote after you will know all of your votes when making their own."
    );
    supported("Ballot Broker");
    // "While voting, you may vote an additional time."
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Ballot Broker");
    t.answer_yes(P1, true);
    option(&mut t, P0, 0);
    option(&mut t, P1, 1);
    option(&mut t, P1, 1);
    option(&mut t, P2, 0);
    let hand = t.hand_size(P0);
    plea(&mut t);
    assert_eq!(voters(&t), vec![P0, P1, P1, P2]);
    // Knowledge 2, time 2: tied.
    assert_eq!(t.hand_size(P0), hand + 3);
}
