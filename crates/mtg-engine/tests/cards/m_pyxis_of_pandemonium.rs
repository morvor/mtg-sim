//! Pyxis of Pandemonium (hand-written, `src/cards/pyxis_of_pandemonium.rs`).

use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_exiled_permanent_cards_enter_under_their_owners_control() {
    cr!("406.3", "607.2a");
    let mut t = TestGame::new(2);
    let pyxis = t.battlefield(P0, "Pyxis of Pandemonium");
    t.library_top(P0, "Lightning Bolt");
    t.library_top(P1, "Grizzly Bears");
    t.activate(P0, pyxis, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 2);
    assert!(t.g.exile.iter().all(|e| t.obj(*e).face_down));
    t.g.objects[pyxis.0 as usize].tapped = false;
    t.lands(P0, "Wastes", 7);
    t.activate(P0, pyxis, 1, &[]).unwrap();
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert_eq!(t.obj(bears[0]).controller, P1);
    // The instant stays in exile, face up.
    assert!(t.in_exile("Lightning Bolt"));
    assert!(t.g.exile.iter().all(|e| !t.obj(*e).face_down));
}
