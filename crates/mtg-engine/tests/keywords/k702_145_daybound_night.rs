//! CR 702.145c: a front-face-up permanent with daybound while it's night is transformed
//! immediately.

use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const RUFFIAN: &str = "Tavern Ruffian // Tavern Smasher";

#[test]
fn a_daybound_permanent_phasing_in_at_night_transforms_immediately() {
    cr!("702.145c");
    let mut t = TestGame::new(2);
    let id = t.battlefield(P0, RUFFIAN);
    t.settle();
    assert_eq!(t.g.day, Some(true));
    // It phases out; while it's phased out it becomes night, so it isn't transformed as
    // it becomes night (phased-out permanents are treated as though they don't exist).
    mtg_engine::kw::phasing::phase_out(&mut t.g, vec![id]);
    t.g.set_day(false);
    assert_eq!(t.obj_now(id).face, FaceState::Front);
    // It phases in at night, front face up: its controller transforms it at once.
    mtg_engine::kw::phasing::phase_in(&mut t.g, id);
    t.settle();
    assert!(!t.obj_now(id).phased_out);
    assert_eq!(t.obj_now(id).face, FaceState::Back);
    assert_eq!(t.obj_now(id).chars.name, "Tavern Smasher");
}

#[test]
fn a_daybound_permanent_turned_face_up_at_night_transforms_immediately() {
    cr!("702.145c");
    let mut t = TestGame::new(2);
    t.g.set_day(false);
    // Soul Summons manifests the Ruffian: a face-down 2/2 without daybound.
    let card = t.library_top(P0, RUFFIAN);
    t.lands(P0, "Plains", 2);
    let spell = t.hand(P0, "Soul Summons");
    t.cast(P0, spell).go();
    t.resolve_all();
    let m = t.g.current(card);
    assert!(t.obj(m).face_down);
    assert_eq!(t.g.day, Some(false));
    // Turned face up for its mana cost at night, it's front face up with daybound: it's
    // transformed at once.
    t.g.players[P0.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::R, 4);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        mtg_engine::decision::Action::Special(mtg_engine::decision::SpecialAction::TurnFaceUp {
            obj: m,
        }),
    )
    .expect("turn face up");
    t.settle();
    assert!(!t.obj_now(m).face_down);
    assert_eq!(t.obj_now(m).face, FaceState::Back);
    assert_eq!(t.obj_now(m).chars.name, "Tavern Smasher");
}
