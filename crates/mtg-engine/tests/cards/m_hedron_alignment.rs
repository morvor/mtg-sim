//! Hedron Alignment (hand-written, `src/cards/hedron_alignment.rs`).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn setup(t: &mut TestGame, in_exile: bool) {
    t.battlefield(P0, "Hedron Alignment");
    t.hand(P0, "Hedron Alignment");
    t.graveyard(P0, "Hedron Alignment");
    if in_exile {
        t.exile(P0, "Hedron Alignment");
    }
    t.set_step(P1, Step::End);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn wins_with_one_in_each_of_the_four_zones() {
    cr!("104.2b");
    let mut t = TestGame::new(2);
    setup(&mut t, true);
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P0])));
}

#[test]
fn no_win_without_one_in_exile() {
    cr!("104.2b");
    let mut t = TestGame::new(2);
    setup(&mut t, false);
    assert!(t.g.result.is_none());
}

#[test]
fn the_exiled_one_must_be_face_up() {
    cr!("104.2b");
    ruling!("Hedron Alignment", "The Hedron Alignment in exile must be face up");
    let mut t = TestGame::new(2);
    let e = t.exile(P0, "Hedron Alignment");
    t.g.objects[e.0 as usize].face_down = true;
    setup(&mut t, false);
    assert!(t.g.result.is_none());
}
