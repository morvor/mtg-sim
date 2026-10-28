//! Reveal-until run by another player (round-4 card flow, CR 701.20): "The controller of
//! target artifact sacrifices it, then reveals cards ... until they reveal an artifact
//! card" (Shape Anew), "Target permanent's controller sacrifices it. If the player does,
//! they reveal cards ... until they reveal a permanent card that shares a card type with
//! the sacrificed permanent ..." (Reweave).

use crate::basic_effects_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn shape_anew_its_controller_sacrifices_and_reveals_until_an_artifact() {
    cr!("701.20a", "701.21a");
    ruling!("Shape Anew", "they will still have to shuffle their library");
    assert_supported("Shape Anew");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.library_top(P1, "Sol Ring");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.library_top(P1, "Island");
    let lib = t.library_size(P1);
    t.lands(P0, "Island", 4);
    let s = t.hand(P0, "Shape Anew");
    t.cast(P0, s).target(Entity::Object(thopter)).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Ornithopter"));
    let ring = t.named_on_battlefield("Sol Ring");
    assert_eq!(ring.len(), 1);
    assert_eq!(t.obj_now(ring[0]).controller, P1);
    // The other revealed cards were shuffled back in.
    assert_eq!(t.library_size(P1), lib - 1);
    assert_eq!(t.zone(t.g.current(bears)), Zone::Library(P1));
}

#[test]
fn reweave_reveals_until_a_card_sharing_a_type_with_the_sacrificed_permanent() {
    cr!("701.20a", "701.21a");
    ruling!("Reweave", "it uses the type(s) that the permanent had right before it was sacrificed");
    assert_supported("Reweave");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P1, "Hill Giant");
    t.library_top(P1, "Sol Ring");
    t.library_top(P1, "Island");
    t.lands(P0, "Island", 6);
    let s = t.hand(P0, "Reweave");
    t.cast(P0, s).target(Entity::Object(bears)).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // A creature: not the land or the artifact revealed before it.
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    assert_eq!(t.obj_now(giant[0]).controller, P1);
    assert!(t.named_on_battlefield("Sol Ring").is_empty());
}
