//! Charismatic Conqueror: "Whenever an artifact or creature an opponent controls enters
//! untapped, they may tap that permanent. If they don't, you create a 1/1 white Vampire
//! creature token with lifelink." The every-card sweep found two players' Conquerors
//! feeding each other's tokens without end when the opponents always declined to tap.

use mtg_engine::testing::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::*;

fn vampires(t: &TestGame, p: PlayerId) -> usize {
    t.named_on_battlefield("Vampire Token")
        .iter()
        .filter(|&&v| t.obj_now(v).controller == p)
        .count()
}

#[test]
fn the_opponent_chooses_as_it_resolves_between_tapping_and_a_token() {
    cr!("603.5", "608.2c", "701.26a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Charismatic Conqueror");
    // Tapped: no token.
    t.answer_yes(P1, true);
    let bears = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(vampires(&t, P0), 0);
    // Not tapped: a token.
    t.clear_answers();
    t.answer_yes(P1, false);
    let bears = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    assert_eq!(vampires(&t, P0), 1);
}

#[test]
fn a_permanent_already_tapped_gives_a_token() {
    cr!("701.26a", "608.2c");
    ruling!(
        "Charismatic Conqueror",
        "If the permanent is tapped as the ability resolves, the opponent can't choose to tap it"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Charismatic Conqueror");
    t.answer_yes(P1, true);
    let bears = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.tap(bears);
    t.resolve_all();
    assert_eq!(vampires(&t, P0), 1);
}

#[test]
fn a_permanent_gone_from_the_battlefield_gives_a_token() {
    cr!("608.2c", "400.7");
    ruling!(
        "Charismatic Conqueror",
        "if it's no longer on the battlefield as the ability resolves, they can't choose to tap it"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Charismatic Conqueror");
    t.answer_yes(P1, true);
    let bears = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.g.move_object(bears, Zone::Graveyard(P1), MoveCause::Effect, None)
        .unwrap();
    t.resolve_all();
    assert_eq!(vampires(&t, P0), 1, "{}", t.dump_log());
    // The card left behind wasn't tapped either.
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn only_one_of_two_triggers_can_tap_the_permanent() {
    // Why two Conquerors on each side loop without end: tapping the entering permanent
    // for the first trigger leaves it tapped for the second, which makes a token.
    cr!("608.2c");
    ruling!(
        "Charismatic Conqueror",
        "If the permanent is tapped as the ability resolves, the opponent can't choose to tap it"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Charismatic Conqueror");
    t.battlefield(P0, "Charismatic Conqueror");
    t.answer_yes(P1, true);
    let bears = t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert_eq!(vampires(&t, P0), 1);
}

#[test]
fn two_players_conquerors_trigger_on_each_others_tokens() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Charismatic Conqueror");
    t.battlefield(P1, "Charismatic Conqueror");
    // P1 declines to tap its Bears: P0 gets a Vampire, which enters untapped, so P1's
    // Conqueror triggers; P0 taps that Vampire, ending the chain.
    t.answer_yes(P1, false);
    t.answer_yes(P0, true);
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(vampires(&t, P0), 1);
    assert_eq!(vampires(&t, P1), 0);
    let v = t.named_on_battlefield("Vampire Token")[0];
    assert!(t.obj_now(v).tapped);
}

