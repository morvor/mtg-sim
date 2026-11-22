//! CR 613.7k: sticker timestamps.

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::facedown::{turn_face_down, turn_face_up};
use mtg_engine::object::*;
use mtg_engine::stickers::{self, StickerKind};
use mtg_engine::testing::*;
use mtg_engine::*;

fn pt_sticker(t: &mut TestGame, id: ObjectId, p: i32, tough: i32) {
    assert!(stickers::put_sticker(
        &mut t.g,
        P0,
        id,
        StickerKind::PowerToughness(p, tough)
    ));
    t.recompute();
}

#[test]
fn sticker_gets_a_timestamp_when_put_on_and_after_the_objects_new_timestamp() {
    // CR 613.7k: a sticker receives a new timestamp each time it's put on an object; when
    // the object receives a new timestamp, the sticker receives one immediately after.
    cr!("613.7k");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pt_sticker(&mut t, bears, 4, 4);
    assert_eq!(t.pt(bears), (4, 4));
    // A later effect setting power and toughness wins over the sticker...
    modify_target(&mut t, P1, bears, vec![set_pt(1, 1)]);
    t.recompute();
    assert_eq!(t.pt(bears), (1, 1));
    // ...and a later name-setting effect over a name sticker.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    assert!(stickers::put_sticker(
        &mut t.g,
        P0,
        bears,
        StickerKind::Name("Space".into())
    ));
    t.recompute();
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears Space");
    modify_target(&mut t, P1, bears, vec![Modification::SetName("Oko".into())]);
    t.recompute();
    assert_eq!(t.obj_now(bears).chars.name, "Oko");
    // Turning it face down and back up gives it new timestamps (CR 613.7f); its stickers
    // get new ones right after, so they now apply after those effects. The name sticker
    // goes at the end, as the name has fewer words now (CR 123.6c).
    assert!(turn_face_down(&mut t.g, bears));
    assert!(turn_face_up(&mut t.g, bears, false));
    t.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.obj_now(bears).chars.name, "Oko Space");
}

#[test]
fn stickers_keep_their_relative_order() {
    // CR 613.7k: if an object has more than one sticker on it as it enters a zone, the
    // relative timestamp order of those stickers remains unchanged.
    cr!("613.7k");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    pt_sticker(&mut t, bears, 5, 5);
    pt_sticker(&mut t, bears, 3, 3);
    assert_eq!(t.pt(bears), (3, 3));
    let exiled =
        t.g.move_object(bears, Zone::Exile, MoveCause::Effect, Some(P0))
            .unwrap();
    t.recompute();
    assert_eq!(t.pt(exiled), (3, 3));
    let back =
        t.g.move_object(exiled, Zone::Battlefield, MoveCause::Effect, Some(P0))
            .unwrap();
    t.recompute();
    assert_eq!(t.pt(back), (3, 3));
    assert_eq!(stickers::stickers_on(&t.g, back).len(), 2);
    // And both still come after the permanent's own new timestamp.
    modify_target(&mut t, P1, back, vec![set_pt(1, 1)]);
    t.recompute();
    assert_eq!(t.pt(back), (1, 1));
    assert!(turn_face_down(&mut t.g, back));
    assert!(turn_face_up(&mut t.g, back, false));
    t.recompute();
    assert_eq!(t.pt(back), (3, 3));
}

#[test]
fn sticker_gets_a_new_timestamp_when_its_object_becomes_part_of_a_merged_permanent() {
    // CR 613.7k: if the object a sticker is on becomes part of a merged permanent, the
    // sticker receives a new timestamp at that time (and is on the merged permanent,
    // CR 123.5b).
    cr!("613.7k");
    let mut t = TestGame::new(2);
    let card = t.battlefield(P0, "Llanowar Elves");
    pt_sticker(&mut t, card, 4, 4);
    let target = t.battlefield(P0, "Grizzly Bears");
    modify_target(&mut t, P1, target, vec![set_pt(1, 1)]);
    t.recompute();
    assert_eq!(t.pt(target), (1, 1));
    stickers::merge_into(&mut t.g, card, target);
    t.recompute();
    assert_eq!(t.pt(target), (4, 4));
    assert!(stickers::is_stickered(&t.g, target));
    assert!(!stickers::is_stickered(&t.g, card));
}
