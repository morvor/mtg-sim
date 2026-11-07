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

#[test]
fn chaotic_transformation_each_controller_reveals_for_each_exiled_permanent() {
    cr!("701.20a", "608.2c");
    ruling!("Chaotic Transformation", "The target permanents are exiled all at the same time");
    assert_supported("Chaotic Transformation");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // P0 reveals until an artifact; P1 until a creature.
    t.library_top(P0, "Ornithopter");
    t.library_top(P0, "Hill Giant");
    t.library_top(P1, "Craw Wurm");
    t.library_top(P1, "Sol Ring");
    t.lands(P0, "Mountain", 6);
    let c = t.hand(P0, "Chaotic Transformation");
    t.cast(P0, c)
        .target(Entity::Object(ring))
        .target(Entity::Object(bears))
        .go();
    t.resolve();
    assert!(t.in_exile("Sol Ring") && t.in_exile("Grizzly Bears"));
    let thopter = t.named_on_battlefield("Ornithopter");
    assert_eq!(thopter.len(), 1);
    assert_eq!(t.obj_now(thopter[0]).controller, P0);
    let wurm = t.named_on_battlefield("Craw Wurm");
    assert_eq!(wurm.len(), 1);
    assert_eq!(t.obj_now(wurm[0]).controller, P1);
    // The cards revealed before them stayed in the libraries.
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    assert!(t.named_on_battlefield("Sol Ring").is_empty());
}

#[test]
fn indomitable_creativity_exiled_cards_enter_together_under_their_owners_control() {
    cr!("701.20a", "101.4");
    ruling!("Indomitable Creativity", "All of the exiled cards are put onto the battlefield at the same time");
    ruling!("Indomitable Creativity", "That player doesn't choose one type");
    assert_supported("Indomitable Creativity");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P0, "Craw Wurm");
    t.library_top(P0, "Island");
    t.library_top(P1, "Sol Ring");
    t.library_top(P1, "Island");
    t.lands(P0, "Mountain", 5);
    let c = t.hand(P0, "Indomitable Creativity");
    t.answer(P0, DecisionKind::X, decision::Answer::Number(2));
    t.cast(P0, c)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Ornithopter") && t.in_graveyard(P1, "Grizzly Bears"));
    let wurm = t.named_on_battlefield("Craw Wurm");
    let ring = t.named_on_battlefield("Sol Ring");
    assert_eq!((wurm.len(), ring.len()), (1, 1));
    assert_eq!(t.obj_now(wurm[0]).controller, P0);
    assert_eq!(t.obj_now(ring[0]).controller, P1);
    // The Islands revealed first stayed in the libraries (then shuffled).
    let islands_in = |p| t.g.player(p).library.iter().filter(|o| t.g.obj(**o).name() == "Island").count();
    assert_eq!((islands_in(P0), islands_in(P1)), (1, 1));
}

#[test]
fn kaboom_reveals_and_deals_damage_for_each_target() {
    cr!("701.20a", "608.2c");
    ruling!("Kaboom!", "those targets will be handled one at a time");
    assert_supported("Kaboom!");
    let mut t = TestGame::new(2);
    let jace = t.battlefield(P1, "Jace Beleren");
    // Bottom to top: Hill Giant (4), Island, Grizzly Bears (2), Island.
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Island");
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.lands(P0, "Mountain", 5);
    let k = t.hand(P0, "Kaboom!");
    t.cast(P0, k)
        .targets(&[Entity::Player(P1), Entity::Object(jace)])
        .go();
    t.resolve();
    // P1 first: Grizzly Bears (2); then Jace: Hill Giant (4) — Bears and the Island were
    // put on the bottom.
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    let lib = &t.g.player(P0).library;
    let bottom: Vec<&str> = lib.iter().take(4).map(|o| t.g.obj(*o).name()).collect();
    assert!(bottom.contains(&"Grizzly Bears") && bottom.contains(&"Hill Giant"));
}
