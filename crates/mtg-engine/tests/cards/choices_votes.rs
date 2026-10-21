//! Votes beyond the open word and object votes (CR 701.38; `kwa/vote.rs`,
//! `oracle/patterns/a701_secret_votes.rs`, `a701_votes_results_more.rs`): secret votes on
//! words and for players, "up to one" object, per-vote instructions with "the voter",
//! results by color and by card, and "whenever players finish voting" follow-ups.

use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

fn vote_player(t: &mut TestGame, voter: PlayerId, p: PlayerId) {
    t.answer_choose(voter, &[Entity::Player(p)]);
}

fn vote_word(t: &mut TestGame, voter: PlayerId, i: usize) {
    t.answer(voter, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn mob_verdict_damage_for_each_vote_an_opponent_received() {
    cr!("701.38a", "701.38b");
    ruling!("Mob Verdict", "Each player must vote for another player");
    compiles("Mob Verdict");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 4);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Mob Verdict");
    t.advance_to(P0, Step::PrecombatMain);
    vote_player(&mut t, P0, P1);
    vote_player(&mut t, P1, P2);
    vote_player(&mut t, P2, P1);
    let hand = t.hand_size(P0);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!((t.life(P1), t.life(P2)), (16, 18));
    assert!(!t.on_battlefield(bears), "4 damage");
    assert_eq!(t.hand_size(P0), hand - 1, "no votes for P0: no cards");
    // P0 couldn't vote for themself.
    let p0_candidates = t.asked().into_iter().find_map(|(p, d)| match d {
        Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
        _ => None,
    });
    assert_eq!(
        p0_candidates,
        Some(vec![Entity::Player(P1), Entity::Player(P2)])
    );
}

#[test]
fn mob_verdict_draw_for_each_vote_you_received() {
    cr!("701.38a");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 4);
    let spell = t.hand(P0, "Mob Verdict");
    t.advance_to(P0, Step::PrecombatMain);
    vote_player(&mut t, P0, P1);
    vote_player(&mut t, P1, P0);
    vote_player(&mut t, P2, P0);
    let hand = t.hand_size(P0);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn truth_or_consequences_secret_word_votes() {
    cr!("701.38a");
    compiles("Truth or Consequences");
    let mut t = TestGame::new(2);
    t.lands(P0, "Volcanic Island", 4);
    let spell = t.hand(P0, "Truth or Consequences");
    t.advance_to(P0, Step::PrecombatMain);
    vote_word(&mut t, P0, 0); // truth
    vote_word(&mut t, P1, 1); // consequences
    let hand = t.hand_size(P0);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 1, "one truth vote");
    assert_eq!(t.life(P1), 17, "one consequences vote");
}

#[test]
fn cirdan_each_player_draws_for_votes_received_and_the_others_may_put_a_permanent() {
    cr!("701.38a");
    ruling!(
        "Círdan the Shipwright",
        "first the player whose turn it is draws a card for each vote they received"
    );
    compiles("Círdan the Shipwright");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.hand(P0, "Grizzly Bears");
    vote_player(&mut t, P0, P1);
    vote_player(&mut t, P1, P1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.enter(P0, "Círdan the Shipwright");
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 + 2, "two votes");
    assert!(
        !t.named_on_battlefield("Grizzly Bears").is_empty(),
        "no votes: P0 may put one"
    );
    assert_eq!(t.hand_size(P0), h0 - 1);
}

#[test]
fn council_guardian_protection_from_each_color_tied_for_most_votes() {
    cr!("701.38a", "702.16a");
    ruling!(
        "Council Guardian",
        "You must vote for one of the available options"
    );
    compiles("Council Guardian");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    vote_word(&mut t, P0, 2); // red
    vote_word(&mut t, P1, 3); // green
    vote_word(&mut t, P2, 2); // red
    let g = t.enter(P0, "Council Guardian");
    t.resolve_all();
    let prot: Vec<String> = t
        .obj_now(g)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Protection)
        .map(|k| format!("{:?}", k.filter))
        .collect();
    assert_eq!(prot.len(), 1, "only red has the most votes: {prot:?}");
    assert!(prot[0].contains("Red"));
}

#[test]
fn custodi_squire_returns_the_card_with_the_most_votes() {
    cr!("701.38a", "701.38b");
    compiles("Custodi Squire");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P1, &[Entity::Object(thopter)]);
    t.enter(P0, "Custodi Squire");
    t.resolve_all();
    assert!(
        t.in_hand(P0, "Grizzly Bears") && t.in_hand(P0, "Ornithopter"),
        "tied"
    );
}

/// Vault 11 at chapter II with P1's Grizzly Bears; votes for creatures (`None`: no
/// vote). Returns (bears alive, cards P0 drew).
fn vault(votes: [Option<bool>; 2]) -> (bool, usize) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let saga = t.battlefield(P0, "Vault 11: Voter's Dilemma");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 1);
    for (p, v) in [P0, P1].into_iter().zip(votes) {
        match v {
            Some(_) => t.answer_choose(p, &[Entity::Object(bears)]),
            None => t.answer_choose(p, &[]),
        };
    }
    let hand = t.hand_size(P0);
    t.g.add_counters(Entity::Object(saga), "lore", 1, None);
    t.settle();
    t.resolve_all();
    (t.on_battlefield(bears), t.hand_size(P0) - hand)
}

#[test]
fn vault_11_secret_votes_for_up_to_one_creature() {
    cr!("701.38a", "714.2b");
    compiles("Vault 11: Voter's Dilemma");
    assert_eq!(
        vault([None, None]),
        (true, 1),
        "no creature got votes: draw"
    );
    assert_eq!(
        vault([Some(true), None]),
        (false, 0),
        "destroy the most voted"
    );
}

#[test]
fn expropriate_for_each_money_vote_you_gain_a_permanent_the_voter_owns() {
    cr!("701.38a", "701.38d");
    ruling!(
        "Expropriate",
        "each vote made for a council's dilemma card adds to the ultimate effect"
    );
    compiles("Expropriate");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 9);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let lions = t.battlefield(P2, "Savannah Lions");
    let mine = t.battlefield(P0, "Hill Giant");
    let spell = t.hand(P0, "Expropriate");
    t.advance_to(P0, Step::PrecombatMain);
    vote_word(&mut t, P0, 0); // time
    vote_word(&mut t, P1, 1); // money
    vote_word(&mut t, P2, 1); // money
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(lions)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(
        t.obj_now(bears).controller,
        P0,
        "P1's money vote: their permanent"
    );
    assert_eq!(t.obj_now(lions).controller, P0, "P2's money vote: theirs");
    assert_eq!(t.obj_now(mine).controller, P0);
    // Only permanents the voter owns were offered for each money vote.
    let offered: Vec<Vec<Entity>> = t
        .asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if p == P0 => Some(candidates),
            _ => None,
        })
        .collect();
    assert_eq!(offered.len(), 2, "one choice per money vote");
    assert!(offered[0]
        .iter()
        .all(|e| e.object().is_some_and(|o| t.g.obj(o).owner == P1)));
    assert!(offered[1]
        .iter()
        .all(|e| e.object().is_some_and(|o| t.g.obj(o).owner == P2)));
    assert_eq!(t.g.extra_turns.len(), 1, "one time vote");
}

#[test]
fn erestor_rewards_opponents_who_voted_like_you_and_scries_for_the_others() {
    cr!("701.38a");
    ruling!(
        "Erestor of the Council",
        "you'll draw a card at the end no matter how other players voted"
    );
    compiles("Erestor of the Council");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Erestor of the Council");
    vote_word(&mut t, P0, 2); // red
    vote_word(&mut t, P1, 2); // red
    vote_word(&mut t, P2, 3); // green
    let hand = t.hand_size(P0);
    t.enter(P0, "Council Guardian");
    t.resolve_all();
    let treasures = |p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|id| {
                t.g.obj(**id).controller == p && t.g.obj(**id).chars.has_subtype("Treasure")
            })
            .count()
    };
    assert_eq!((treasures(P1), treasures(P2)), (1, 0));
    assert_eq!(t.hand_size(P0), hand + 1);
    let scried = t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Scry { cards } if cards.len() == 1));
    assert!(scried, "scry 1: one opponent voted otherwise");
}

#[test]
fn selvalas_stampede_creatures_for_wild_votes_permanents_from_hand_for_free_votes() {
    cr!("701.38a");
    compiles("Selvala's Stampede");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    let spell = t.hand(P0, "Selvala's Stampede");
    let thopter = t.hand(P0, "Ornithopter");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.advance_to(P0, Step::PrecombatMain);
    vote_word(&mut t, P0, 0); // wild
    vote_word(&mut t, P1, 1); // free
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(
        t.named_on_battlefield("Grizzly Bears").len(),
        1,
        "one wild vote"
    );
    assert_eq!(
        t.named_on_battlefield("Ornithopter").len(),
        1,
        "one free vote"
    );
    assert!(!t.in_graveyard(P0, "Island"), "the rest shuffled in");
}
