//! Guesses (`oracle/patterns/choice_grammar_hidden.rs`): "[player] guesses whether
//! [proposition]" and "If they guessed right/wrong, ...".

use mtg_engine::decision::Answer;
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

/// `p` guesses yes (index 0) or no (index 1).
fn guess(t: &mut TestGame, p: PlayerId, yes: bool) {
    t.answer(p, DecisionKind::Option, Answer::Index(if yes { 0 } else { 1 }));
}

fn clues(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.g.obj(**id).controller == p && t.g.obj(**id).chars.has_subtype("Clue"))
        .count()
}

/// The Seventh Doctor attacks with Grizzly Bears (mana value 2) chosen from hand and no
/// artifacts; the defending player guesses `yes` (greater than 0). Returns (Bears on the
/// battlefield, Clues).
fn seventh_doctor(yes: bool) -> (bool, usize) {
    let mut t = TestGame::new(2);
    let doctor = t.battlefield(P0, "The Seventh Doctor");
    let bears = t.hand(P0, "Grizzly Bears");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    guess(&mut t, P1, yes);
    t.answer_yes(P0, true); // cast it if allowed
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(doctor, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let on_bf = !t.named_on_battlefield("Grizzly Bears").is_empty();
    (on_bf, clues(&t, P0))
}

#[test]
fn the_seventh_doctor_wrong_guess_lets_you_cast_it_free_otherwise_investigate() {
    cr!("608.2c", "701.16a");
    compiles("The Seventh Doctor");
    assert_eq!(seventh_doctor(false), (true, 0), "guessed wrong: cast it");
    assert_eq!(seventh_doctor(true), (false, 1), "guessed right: investigate");
}

#[test]
fn master_of_predicaments_the_damaged_player_guesses() {
    cr!("608.2c");
    compiles("Master of Predicaments");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Predicaments");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    guess(&mut t, P1, true); // greater than 4: wrong
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.attack(&[(master, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(!t.named_on_battlefield("Grizzly Bears").is_empty());
    let asked_p1 = t
        .asked()
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, mtg_engine::decision::Decision::ChooseOption { .. }));
    assert!(asked_p1, "that player guesses");
}

/// Liar's Pendulum naming Grizzly Bears (in hand or not), P1 guessing `yes`, P0 revealing
/// or not: the cards P0 drew.
fn pendulum(in_hand: bool, yes: bool, reveal: bool) -> usize {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let pendulum = t.battlefield(P0, "Liar's Pendulum");
    t.lands(P0, "Plains", 2);
    if in_hand {
        t.hand(P0, "Grizzly Bears");
    }
    t.answer(P0, DecisionKind::Name, Answer::Text("Grizzly Bears".into()));
    guess(&mut t, P1, yes);
    t.answer_yes(P0, reveal);
    let hand = t.hand_size(P0);
    t.activate(P0, pendulum, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    t.hand_size(P0) - hand
}

#[test]
fn liars_pendulum_draws_only_if_you_reveal_and_they_guessed_wrong() {
    cr!("608.2c");
    ruling!("Liar's Pendulum", "If your opponent was wrong, you draw a card");
    ruling!(
        "Liar's Pendulum",
        "You can’t draw a card if you don’t reveal your hand"
    );
    compiles("Liar's Pendulum");
    assert_eq!(pendulum(true, false, true), 1, "wrong guess, revealed");
    assert_eq!(pendulum(true, true, true), 0, "right guess");
    assert_eq!(pendulum(false, true, true), 1, "wrong guess: it isn't there");
    assert_eq!(pendulum(true, false, false), 0, "not revealed");
}

/// Gollum attacks with a Forest on top of P0's library (or an empty library); P0 chooses
/// land, P1 guesses `yes`. Returns (Gollum still attacking, cards drawn).
fn gollum(top: Option<&str>, yes: bool) -> (bool, usize) {
    let mut t = TestGame::new(2);
    let gollum = t.battlefield(P0, "Gollum, Scheming Guide");
    t.advance_to(P0, Step::BeginningOfCombat);
    if top.is_none() {
        t.g.players[0].library.clear();
    }
    if let Some(c) = top {
        t.library_top(P0, c);
    }
    t.answer(P0, DecisionKind::Option, Answer::Index(0)); // land
    guess(&mut t, P1, yes);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(gollum, Entity::Player(P1))]),
    );
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let attacking = t
        .g
        .combat
        .as_ref()
        .is_some_and(|c| c.attacker(gollum).is_some());
    (attacking, t.hand_size(P0) - hand)
}

#[test]
fn gollum_right_guess_removes_it_from_combat_otherwise_you_draw() {
    cr!("506.4", "608.2c");
    ruling!(
        "Gollum, Scheming Guide",
        "You reveal the card even if your opponent guessed correctly"
    );
    compiles("Gollum, Scheming Guide");
    assert_eq!(gollum(Some("Forest"), true), (false, 0), "right: removed");
    assert_eq!(gollum(Some("Forest"), false), (true, 1), "wrong: draw");
}

#[test]
fn gollum_with_an_empty_library_no_guess_is_right() {
    cr!("608.2c");
    ruling!(
        "Gollum, Scheming Guide",
        "your opponent's guess will not be right no matter what they guess"
    );
    let (attacking, _) = gollum(None, true);
    assert!(attacking, "the guess wasn't right");
}
