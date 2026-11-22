//! Rulings batch S26 — casting a copy of a card (CR 707.12, 707.12a) with Kaervek, the
//! Punisher ("Whenever you commit a crime, exile up to one target black card from your
//! graveyard and copy it. You may cast the copy. If you do, you lose 2 life."): the copy
//! is cast while the ability resolves, or not at all; a copy that isn't cast ceases to
//! exist (CR 707.10a, 704.5e).

use crate::r_s01_common::supported;
use mtg_engine::object::{ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Copies of cards that still exist.
fn card_copies_left(t: &TestGame) -> usize {
    t.g.objects
        .iter()
        .filter(|o| o.kind == ObjKind::CardCopy && t.g.is_live(o.id) && o.zone != Zone::Nowhere)
        .count()
}

/// P0 controls Kaervek with Night's Whisper in their graveyard, and commits a crime by
/// casting Lightning Bolt at P1: Kaervek's trigger (targeting Night's Whisper) is on top
/// of the Bolt. Returns (the Bolt, Night's Whisper).
fn kaervek_trigger(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Kaervek, the Punisher");
    supported("Night's Whisper");
    t.battlefield(P0, "Kaervek, the Punisher");
    let whisper = t.graveyard(P0, "Night's Whisper");
    t.lands(P0, "Badlands", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(P1).go();
    t.answer_targets(P0, &[Entity::Object(whisper)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    (bolt, whisper)
}

#[test]
fn kaervek_casts_the_copy_while_its_ability_resolves() {
    cr!("707.12", "608.2", "601.2");
    ruling!(
        "Kaervek, the Punisher",
        "You cast the copy while the ability is resolving and still on the stack. You can’t wait to cast it later in the turn."
    );
    let mut t = TestGame::new(2);
    let (bolt, whisper) = kaervek_trigger(&mut t);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve();
    // Night's Whisper (a sorcery) was cast above the Bolt, as the ability resolved.
    assert_eq!(t.zone(t.g.current(whisper)), Zone::Exile);
    assert_eq!(t.stack_len(), 2);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).chars.name, "Night's Whisper");
    assert!(t.g.stack.contains(&bolt));
    assert_eq!(t.life(P0), 18);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.life(P1), 17);
    assert_eq!(card_copies_left(&t), 0);
}

#[test]
fn a_kaervek_copy_you_dont_cast_ceases_to_exist() {
    cr!("707.12a", "707.10a", "704.5e");
    ruling!(
        "Kaervek, the Punisher",
        "If you don’t want to cast the copy, you can choose not to; the copy ceases to exist the next time state-based actions are performed."
    );
    let mut t = TestGame::new(2);
    let (_, whisper) = kaervek_trigger(&mut t);
    t.answer_yes(P0, false);
    t.resolve();
    // The card is exiled; no copy was cast, and none is left anywhere; no life lost.
    assert_eq!(t.zone(t.g.current(whisper)), Zone::Exile);
    assert_eq!(t.stack_len(), 1);
    assert_eq!(card_copies_left(&t), 0);
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}
