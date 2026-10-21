//! Copy exceptions (CR 707.9b): a serial list without repeated subjects (The Apprentice's
//! Folly) and a power and toughness taken from the entering object itself (Hulking
//! Metamorph, printed or prototype values).

use crate::combat2_common::supported;
use mtg_engine::object::CastMethod;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn hulking_metamorph_copies_with_its_printed_power_and_toughness() {
    cr!("707.9b");
    ruling!(
        "Hulking Metamorph",
        "Use the appropriate printed power and toughness"
    );
    supported("Hulking Metamorph");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let h = t.enter(P0, "Hulking Metamorph");
    let now = t.g.current(h);
    let o = t.g.obj(now);
    assert_eq!(o.chars.name.as_str(), "Grizzly Bears");
    assert!(o.chars.is(CardType::Artifact));
    assert!(o.chars.is(CardType::Creature));
    assert_eq!(t.pt(h), (7, 7));
}

#[test]
fn a_prototyped_hulking_metamorph_copies_as_a_3_3() {
    cr!("707.9b", "718.3b");
    ruling!(
        "Hulking Metamorph",
        "Use the appropriate printed power and toughness"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    let h = t.hand(P0, "Hulking Metamorph");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.cast(P0, h)
        .method(CastMethod::Keyword(KeywordKind::Prototype))
        .go();
    t.resolve_all();
    let now = t.g.current(h);
    assert!(t.on_battlefield(now));
    assert_eq!(t.g.obj(now).chars.name.as_str(), "Grizzly Bears");
    assert!(t.g.obj(now).chars.is(CardType::Artifact));
    assert_eq!(t.pt(now), (3, 3));
}

#[test]
fn the_apprentices_folly_makes_hasty_nonlegendary_reflections() {
    cr!("707.9b", "714.2b");
    supported("The Apprentice's Folly");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.set_step(P0, Step::PrecombatMain);
    t.enter(P0, "The Apprentice's Folly");
    t.resolve_all();
    let copies: Vec<_> = t
        .named_on_battlefield("Isamaru, Hound of Konda")
        .into_iter()
        .filter(|o| *o != t.g.current(isamaru))
        .collect();
    assert_eq!(copies.len(), 1);
    let c = t.g.obj(copies[0]);
    assert!(c.is_token());
    assert!(!c.chars.supertypes.contains(Supertype::Legendary));
    assert!(c.chars.subtypes.iter().any(|s| s == "Reflection"));
    assert!(c.chars.subtypes.iter().any(|s| s == "Dog"));
    assert!(c.has_keyword(KeywordKind::Haste));
    // Both stay: the copy isn't legendary (CR 704.5j).
    assert!(t.on_battlefield(isamaru));
}
