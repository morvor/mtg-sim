//! Permissions to look at hidden cards (`oracle/patterns/look_permissions.rs`; CR 406.3,
//! 708.5, 607.2a).

use mtg_engine::ability::*;
use mtg_engine::facedown::can_look_at;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// `p` manifests the top card of their library.
fn manifest_top(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let top = t.g.library_top(p).expect("empty library");
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::KeywordAction {
            action: KeywordAction::Manifest,
            who: PlayerRef::You,
            what: Sel::None,
            n: Value::c(1),
        },
        &mut ctx,
    );
    t.g.recompute();
    t.settle();
    t.g.current(top)
}

#[test]
fn keeper_of_the_lens_sees_face_down_creatures_it_doesnt_control() {
    cr!("708.5");
    assert_supported(&["Keeper of the Lens", "Lens of Clarity"]);
    let mut t = TestGame::new(2);
    t.library_top(P1, "Grizzly Bears");
    let fd = manifest_top(&mut t, P1);
    assert!(!can_look_at(&t.g, P0, fd));
    let keeper = t.battlefield(P0, "Keeper of the Lens");
    t.g.recompute();
    assert!(can_look_at(&t.g, P0, fd));
    // Only its controller.
    assert!(t.g.obj(keeper).controller == P0);
    let mut t2 = TestGame::new(3);
    t2.library_top(P1, "Grizzly Bears");
    let fd = manifest_top(&mut t2, P1);
    t2.battlefield(P0, "Keeper of the Lens");
    t2.g.recompute();
    assert!(!can_look_at(&t2.g, P2, fd));
}

#[test]
fn lumbering_laundry_until_end_of_turn() {
    cr!("708.5");
    assert_supported(&["Lumbering Laundry"]);
    let mut t = TestGame::new(2);
    t.library_top(P1, "Grizzly Bears");
    let fd = manifest_top(&mut t, P1);
    let ll = t.battlefield(P0, "Lumbering Laundry");
    t.lands(P0, "Island", 2);
    t.activate(P0, ll, 0, &[]).unwrap();
    t.resolve_all();
    assert!(can_look_at(&t.g, P0, fd));
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    assert!(!can_look_at(&t.g, P0, fd));
}

#[test]
fn bane_alley_broker_looks_at_its_exiled_cards() {
    cr!("406.3", "607.2a");
    assert_supported(&["Bane Alley Broker"]);
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bane Alley Broker");
    t.library_top(P0, "Island");
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, b, 0, &[]).unwrap();
    t.resolve_all();
    let exiled = t.g.current(bears);
    assert!(t.g.obj(exiled).face_down);
    assert!(can_look_at(&t.g, P0, exiled));
    assert!(!can_look_at(&t.g, P1, exiled));
}
