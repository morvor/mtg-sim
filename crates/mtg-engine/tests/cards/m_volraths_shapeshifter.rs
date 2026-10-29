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
