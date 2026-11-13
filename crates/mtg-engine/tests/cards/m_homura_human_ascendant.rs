//! Homura, Human Ascendant (hand-written, `src/cards/homura_human_ascendant.rs`).

use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn returns_flipped_when_it_dies() {
    cr!("710.2", "603.6c");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Homura, Human Ascendant // Homura's Essence");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(h, None);
    t.resolve_all();
    let e = t.named_on_battlefield("Homura's Essence");
    assert_eq!(e.len(), 1);
    let o = t.g.obj(e[0]);
    assert!(o.flipped);
    assert_eq!(o.face, FaceState::Flipped);
    assert_eq!(o.controller, P0);
    // Homura's Essence: creatures you control get +2/+2.
    assert_eq!(t.pt(bear), (4, 4));
}
