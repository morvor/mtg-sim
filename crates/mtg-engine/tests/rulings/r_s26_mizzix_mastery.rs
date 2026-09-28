//! Rulings batch S26 — Mizzix's Mastery ("Exile target card that's an instant or sorcery
//! from your graveyard. For each card exiled this way, copy it, and you may cast the copy
//! without paying its mana cost. Exile Mizzix's Mastery." Overload {5}{R}{R}{R}): copies of
//! cards cast during its resolution (CR 707.12), without paying their mana costs, so X is 0
//! (CR 107.3b); a copy that isn't cast ceases to exist (CR 707.12a, 704.5e).

use crate::r_s01_common::supported;
use crate::r_s25_common::x_of;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const OVERLOAD: CastMethod = CastMethod::Keyword(KeywordKind::Overload);

/// Copies of cards that still exist.
fn card_copies_left(t: &TestGame) -> usize {
    t.g.objects
        .iter()
        .filter(|o| o.kind == ObjKind::CardCopy && t.g.is_live(o.id) && o.zone != Zone::Nowhere)
        .count()
}

/// P0 casts Mizzix's Mastery (with enough Mountains) targeting `card` in their graveyard.
fn mastery_targeting(t: &mut TestGame, card: ObjectId) -> ObjectId {
    supported("Mizzix's Mastery");
    t.lands(P0, "Mountain", 4);
    let m = t.hand(P0, "Mizzix's Mastery");
    t.cast(P0, m).target(card).go()
}

#[test]
fn a_copy_cast_without_paying_its_mana_cost_has_x_0() {
    cr!("107.3b", "707.12", "118.9");
    ruling!(
        "Mizzix's Mastery",
        "If the copy has {X} in its mana cost, you must choose 0 as the value of X."
    );
    let mut t = TestGame::new(2);
    let blaze = t.graveyard(P0, "Blaze");
    let mastery = mastery_targeting(&mut t, blaze);
    // Cast the copy of Blaze at P1, trying X = 3.
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).chars.name, "Blaze");
    assert_eq!(x_of(&t, top), Some(0));
    assert!(!t.g.stack.contains(&mastery));
    assert!(t.in_exile("Mizzix's Mastery"));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn the_copy_is_cast_while_mastery_resolves() {
    cr!("707.12", "608.2", "307.1");
    ruling!(
        "Mizzix's Mastery",
        "The copies are created and cast during the resolution of Mizzix's Mastery. You can't wait to cast them later in the turn. Timing restrictions based on the copy's type are ignored."
    );
    ruling!(
        "Mizzix's Mastery",
        "The cards remain exiled no matter what happens to the copies."
    );
    let mut t = TestGame::new(2);
    let whisper = t.graveyard(P0, "Night's Whisper");
    mastery_targeting(&mut t, whisper);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve();
    // A sorcery copy was cast though Mizzix's Mastery was on the stack.
    assert_eq!(t.stack_len(), 1);
    let copy = t.g.stack[0];
    assert_eq!(t.obj(copy).chars.name, "Night's Whisper");
    assert_eq!(t.obj(copy).kind, ObjKind::CardCopy);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.zone(t.g.current(whisper)), Zone::Exile);
    assert_eq!(card_copies_left(&t), 0);
}

#[test]
fn a_copy_you_dont_cast_ceases_to_exist() {
    cr!("707.12a", "704.5e");
    ruling!(
        "Mizzix's Mastery",
        "If you don't cast one of the copies (perhaps because there are no legal targets available or you don't want to), the copy will cease to exist."
    );
    let mut t = TestGame::new(2);
    let whisper = t.graveyard(P0, "Night's Whisper");
    mastery_targeting(&mut t, whisper);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(card_copies_left(&t), 0);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.zone(t.g.current(whisper)), Zone::Exile);
}

#[test]
fn overloaded_mastery_copies_each_card_but_not_itself() {
    cr!("702.96a", "707.12", "405.5");
    ruling!(
        "Mizzix's Mastery",
        "Mizzix's Mastery is still on the stack as it resolves. If you pay the overload cost, Mizzix's Mastery won't copy itself."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Night's Whisper");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 8);
    let m = t.hand(P0, "Mizzix's Mastery");
    t.cast(P0, m).method(OVERLOAD).go();
    for _ in 0..2 {
        t.answer_yes(P0, true);
    }
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let hand = t.hand_size(P0);
    t.resolve();
    // Both instant and sorcery cards were copied and cast; Mizzix's Mastery wasn't.
    let names: Vec<String> = t
        .g
        .stack
        .iter()
        .map(|s| t.obj(*s).chars.name.to_string())
        .collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"Night's Whisper".to_string()));
    assert!(names.contains(&"Lightning Bolt".to_string()));
    assert!(t.in_exile("Mizzix's Mastery"));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 2);
}
