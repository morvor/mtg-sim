//! CR 612.9: a name sticker creates a continuous effect that adds a word to the text that
//! represents the object's name (see also CR 123.6).

use crate::r609_common::*;
use mtg_engine::ability::*;
use mtg_engine::events::MoveCause;
use mtg_engine::object::*;
use mtg_engine::stickers::{self, StickerKind};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn name_of(t: &TestGame, id: ObjectId) -> String {
    t.obj_now(id).chars.name.to_string()
}

fn name_sticker(t: &mut TestGame, p: PlayerId, id: ObjectId, word: &str, position: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(position));
    assert!(stickers::put_sticker(
        &mut t.g,
        p,
        id,
        StickerKind::Name(word.into())
    ));
    t.recompute();
}

#[test]
fn name_sticker_adds_a_word_to_a_permanents_name() {
    // CR 612.9: the word is added to the text of the name; the other words stay. The
    // object's controller chooses where it goes (CR 123.6b).
    cr!("612.9");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    name_sticker(&mut t, P0, bears, "Space", 1);
    assert_eq!(name_of(&t, bears), "Grizzly Space Bears");
    assert!(stickers::is_stickered(&t.g, bears));
    // It's a text-changing effect, not a change to the copiable values (CR 123.1).
    assert_eq!(t.obj_now(bears).copiable.name, "Grizzly Bears");
    // Effects that look for the name see the new one.
    t.custom(
        P1,
        permanent(
            "Space Anthem",
            &[CardType::Enchantment],
            vec![continuous(
                Filter::and(vec![
                    Filter::Type(CardType::Creature),
                    Filter::Named("Grizzly Space Bears".into()),
                ]),
                vec![pt(1, 1)],
            )],
        ),
        Zone::Battlefield,
    );
    t.recompute();
    assert_eq!(t.pt(bears), (3, 3));
    // A second sticker modifies the new name further.
    name_sticker(&mut t, P0, bears, "Big", 0);
    assert_eq!(name_of(&t, bears), "Big Grizzly Space Bears");
    // A player can't put a sticker on an object they don't own (CR 123.3b).
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert!(!stickers::put_sticker(
        &mut t.g,
        P0,
        theirs,
        StickerKind::Name("Space".into())
    ));
}

#[test]
fn name_sticker_on_a_card_not_on_the_battlefield() {
    // CR 612.9: a name sticker on a card in a zone other than the battlefield also adds
    // its word; it stays on as the card moves to another public zone, but not to a
    // hidden zone (CR 123.5).
    cr!("612.9");
    let mut t = TestGame::new(2);
    let elves = t.graveyard(P0, "Llanowar Elves");
    name_sticker(&mut t, P0, elves, "Cool", 2);
    assert_eq!(name_of(&t, elves), "Llanowar Elves Cool");
    let exiled =
        t.g.move_object(elves, Zone::Exile, MoveCause::Effect, Some(P0))
            .unwrap();
    t.recompute();
    assert_eq!(name_of(&t, exiled), "Llanowar Elves Cool");
    let in_hand =
        t.g.move_object(exiled, Zone::Hand(P0), MoveCause::Effect, Some(P0))
            .unwrap();
    t.recompute();
    assert_eq!(name_of(&t, in_hand), "Llanowar Elves");
    assert!(!stickers::is_stickered(&t.g, in_hand));
}

#[test]
fn name_stickers_and_other_text_changes_apply_in_timestamp_order() {
    // CR 612.9 with 123.6c: name stickers and other text-changing effects apply in
    // timestamp order; a sticker's word keeps its position (by word count), or goes at the
    // end if the name now has fewer words.
    cr!("612.9");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    modify_target(&mut t, P1, bears, vec![Modification::SetName("Oko".into())]);
    t.recompute();
    assert_eq!(name_of(&t, bears), "Oko");
    // Placed after the first word of its current name.
    name_sticker(&mut t, P0, bears, "Space", 1);
    assert_eq!(name_of(&t, bears), "Oko Space");
    // A later effect that sets the name overrides it (CR 612.8).
    modify_target(&mut t, P1, bears, vec![Modification::SetName("Bob".into())]);
    t.recompute();
    assert_eq!(name_of(&t, bears), "Bob");
    // A nameless (face-down) object's name becomes the word (CR 123.6b).
    let mut t = TestGame::new(2);
    let fd = t.battlefield(P0, "Grizzly Bears");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, fd));
    t.recompute();
    assert_eq!(name_of(&t, fd), "");
    name_sticker(&mut t, P0, fd, "Space", 0);
    assert_eq!(name_of(&t, fd), "Space");
}
