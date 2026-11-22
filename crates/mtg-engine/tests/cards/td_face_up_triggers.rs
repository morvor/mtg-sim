//! "Whenever you turn a permanent face up" (Growing Dread): turned face up by you — the
//! special action (CR 116.2b, 701.40b) or an effect you control.

use crate::combat2_common::*;
use crate::td_next_turn_durations::mana;
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::kwa::manifest::{put_face_down, MANIFESTED};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn manifest(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let card = t.hand(p, name);
    put_face_down(&mut t.g, card, p, MANIFESTED, None).expect("manifested")
}

fn turn_up(t: &mut TestGame, p: PlayerId, obj: ObjectId) {
    t.g.turn.priority = Some(p);
    t.g.perform_action(p, Action::Special(SpecialAction::TurnFaceUp { obj }))
        .expect("turned face up");
}

#[test]
fn growing_dread_counters_permanents_you_turn_face_up() {
    cr!("701.40b", "116.2b", "603.2");
    ruling!(
        "Growing Dread",
        "you can turn a manifested permanent you control face up by revealing that it's a creature card"
    );
    supported("Growing Dread");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Growing Dread");
    let mine = manifest(&mut t, P0, "Grizzly Bears");
    let theirs = manifest(&mut t, P1, "Grizzly Bears");
    t.lands(P0, "Forest", 2);
    turn_up(&mut t, P0, mine);
    t.resolve_all();
    assert!(!t.obj_now(mine).face_down);
    assert_eq!(t.counters(mine, "+1/+1"), 1);
    // The opponent turning theirs face up: nothing.
    t.lands(P1, "Forest", 2);
    turn_up(&mut t, P1, theirs);
    t.resolve_all();
    assert!(!t.obj_now(theirs).face_down);
    assert_eq!(t.counters(theirs, "+1/+1"), 0);
}

#[test]
fn growing_dread_counts_your_effects_turning_an_opponents_permanent_face_up() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Growing Dread");
    let theirs = manifest(&mut t, P1, "Grizzly Bears");
    let open = t.hand(P0, "Break Open");
    mana(&mut t, P0);
    t.cast_with(P0, open, &[Entity::Object(theirs)]).unwrap();
    t.resolve_all();
    assert!(!t.obj_now(theirs).face_down);
    assert_eq!(t.counters(theirs, "+1/+1"), 1);
}
