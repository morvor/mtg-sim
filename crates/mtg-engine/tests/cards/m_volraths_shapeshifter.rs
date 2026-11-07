//! Volrath's Shapeshifter (hand-written, `src/cards/volraths_shapeshifter.rs`).

use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

fn discard_abilities(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains("Discard a card"))
        .count()
}

#[test]
fn it_has_the_full_text_of_the_top_creature_card_and_keeps_discarding() {
    cr!("612.6");
    ruling!("Volrath's Shapeshifter", "this card keeps the granted ability in addition to those from the card being copied");
    ruling!("Volrath's Shapeshifter", "then it's just a 0/1 blue creature");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Volrath's Shapeshifter");
    t.g.recompute();
    assert_eq!(t.pt(s), (0, 1));
    assert_eq!(discard_abilities(&t, s), 1);
    t.graveyard(P0, "Serra Angel");
    t.g.recompute();
    let o = t.obj_now(s);
    assert_eq!(o.chars.name, "Serra Angel");
    assert_eq!((o.power(), o.toughness()), (4, 4));
    assert!(o.has_keyword(KeywordKind::Flying));
    assert_eq!(discard_abilities(&t, s), 1);
    // The discard ability works: discarding a noncreature card puts it on top.
    t.lands(P0, "Island", 2);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.answer_choose(P0, &[Entity::Object(bolt)]);
    // Serra Angel has no activated ability of its own: index 0 is the discard.
    t.activate(P0, s, 0, &[]).unwrap();
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.obj_now(s).chars.name, "Volrath's Shapeshifter");
    assert_eq!(t.pt(s), (0, 1));
}

#[test]
fn only_its_controllers_graveyard_counts() {
    cr!("612.6");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Volrath's Shapeshifter");
    t.graveyard(P1, "Serra Angel");
    t.g.recompute();
    assert_eq!(t.obj_now(s).chars.name, "Volrath's Shapeshifter");
}

#[test]
fn a_copy_has_the_shapeshifting_ability_not_the_text_it_took() {
    cr!("707.2", "613.1a", "613.1c");
    ruling!("Volrath's Shapeshifter", "A copy of a Volrath's Shapeshifter will have the shapeshifting ability.");
    ruling!("Volrath's Shapeshifter", "The ability that sets its characteristics is a text-changing effect.");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Volrath's Shapeshifter");
    t.graveyard(P0, "Serra Angel");
    t.g.recompute();
    assert_eq!(t.obj_now(s).chars.name, "Serra Angel");
    // P1's Clone copies it: the copiable values are the Shapeshifter's printed ones (a
    // text-changing effect isn't copied), and its own ability looks at P1's graveyard.
    t.g.turn.active = P1;
    t.g.turn.priority = Some(P1);
    t.lands(P1, "Island", 4);
    let clone = t.hand(P1, "Clone");
    t.cast(P1, clone).go();
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(s)]);
    t.resolve();
    let c = t.g.current(clone);
    t.g.recompute();
    assert_eq!(t.obj_now(c).chars.name, "Volrath's Shapeshifter");
    assert_eq!(t.pt(c), (0, 1));
    assert_eq!(discard_abilities(&t, c), 1);
    t.graveyard(P1, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.obj_now(c).chars.name, "Hill Giant");
    assert_eq!(t.pt(c), (3, 3));
}

#[test]
fn changing_form_doesnt_trigger_enters_abilities() {
    cr!("603.6a");
    ruling!("Volrath's Shapeshifter", "When it changes forms, any \"enters\" abilities of the card it \"copies\" do not trigger.");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Volrath's Shapeshifter");
    t.settle();
    let hand = t.hand_size(P0);
    // Elvish Visionary: "When this creature enters, draw a card."
    t.graveyard(P0, "Elvish Visionary");
    t.g.recompute();
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(s).chars.name, "Elvish Visionary");
    assert_eq!(t.hand_size(P0), hand);
}
