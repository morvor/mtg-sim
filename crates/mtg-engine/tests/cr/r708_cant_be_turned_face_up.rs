//! "[Permanents] can't be turned face up" (CR 708.7; `rule_statics::face_up`): Karlov
//! Watchdog, Unable to Scream. Also "Turn target face-down creature face up" (Break Open,
//! CR 708.8).

use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn turn_up(id: ObjectId) -> Action {
    Action::Special(SpecialAction::TurnFaceUp { obj: id })
}

fn can_turn_up(t: &mut TestGame, p: PlayerId, id: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p).contains(&turn_up(id))
}

/// `p` casts Scornful Egotist (morph {U}) face down during their turn; returns the
/// face-down permanent.
fn face_down_egotist(t: &mut TestGame, p: PlayerId) -> ObjectId {
    t.set_step(p, Step::PrecombatMain);
    t.lands(p, "Island", 3);
    let card = t.hand(p, "Scornful Egotist");
    t.cast(p, card)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    t.resolve();
    let id = t.g.current(card);
    assert!(t.obj_now(id).face_down);
    id
}

fn untapped_islands(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.chars.name.as_str() == "Island" && !o.tapped
        })
        .count()
}

#[test]
fn opponents_cant_turn_morphs_face_up_during_your_turn() {
    cr!("708.7", "702.37e");
    ruling!("Karlov Watchdog", "opponents can't attempt to turn face-down creatures face up");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karlov Watchdog");
    let id = face_down_egotist(&mut t, P1);
    t.lands(P1, "Island", 1);
    // During P1's own turn they may.
    assert!(can_turn_up(&mut t, P1, id));
    // During P0's turn they can't, and attempting it pays nothing.
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_turn_up(&mut t, P1, id));
    let before = untapped_islands(&t, P1);
    assert!(t.g.perform_action(P1, turn_up(id)).is_err());
    assert!(t.obj_now(id).face_down);
    assert_eq!(untapped_islands(&t, P1), before);
    // P0's own face-down permanents aren't affected.
    let mine = face_down_egotist(&mut t, P0);
    t.lands(P0, "Island", 1);
    assert!(can_turn_up(&mut t, P0, mine));
    // Once it's P1's turn again, P1 may turn theirs face up.
    t.set_step(P1, Step::PrecombatMain);
    t.g.perform_action(P1, turn_up(id)).unwrap();
    assert!(!t.obj_now(id).face_down);
}

#[test]
fn manifested_creatures_cant_be_turned_face_up_for_their_mana_cost_either() {
    cr!("708.7", "701.40b");
    ruling!("Karlov Watchdog", "by paying the mana cost of a cloaked or manifested creature");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.library_top(P1, "Grizzly Bears");
    t.lands(P1, "Plains", 2);
    let summons = t.hand(P1, "Soul Summons");
    t.cast(P1, summons).go();
    t.resolve();
    let id = t.g.current(bears);
    assert!(t.obj_now(id).face_down && t.on_battlefield(id));
    t.lands(P1, "Forest", 2);
    assert!(can_turn_up(&mut t, P1, id));
    t.battlefield(P0, "Karlov Watchdog");
    t.set_step(P0, Step::PrecombatMain);
    assert!(!can_turn_up(&mut t, P1, id));
    assert!(t.g.perform_action(P1, turn_up(id)).is_err());
    assert!(t.obj_now(id).face_down);
}

#[test]
fn an_enchanted_face_down_creature_cant_be_turned_face_up_at_all() {
    cr!("708.7", "708.8");
    let mut t = TestGame::new(2);
    let id = face_down_egotist(&mut t, P1);
    // Break Open turns a face-down creature face up.
    let other = face_down_egotist(&mut t, P1);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let open = t.hand(P0, "Break Open");
    t.cast(P0, open).target(other).go();
    t.resolve();
    assert!(!t.obj_now(other).face_down);
    // Enchanted by Unable to Scream, neither its controller nor an effect can.
    t.lands(P0, "Island", 1);
    let aura = t.hand(P0, "Unable to Scream");
    t.cast(P0, aura).target(id).go();
    t.resolve();
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 1);
    assert!(!can_turn_up(&mut t, P1, id));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 2);
    let open = t.hand(P0, "Break Open");
    t.cast(P0, open).target(id).go();
    t.resolve();
    assert!(t.obj_now(id).face_down);
    assert!(matches!(t.zone(id), Zone::Battlefield));
}
