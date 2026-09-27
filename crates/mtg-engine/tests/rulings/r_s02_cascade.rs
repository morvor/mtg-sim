//! Rulings batch S02 — cascade (CR 702.85): "When you cast this spell, exile cards from the
//! top of your library until you exile a nonland card whose mana value is less than this
//! spell's mana value. You may cast that card without paying its mana cost ..."

use crate::r_s01_common::*;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Names of the spells on the stack, bottom first.
fn spell_names(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .filter(|s| t.g.obj(**s).is_spell())
        .map(|s| t.g.obj(*s).chars.name.to_string())
        .collect()
}

/// Casts the real card `name` from P0's hand (with the mana for it).
fn cast_from_hand(t: &mut TestGame, name: &str) -> ObjectId {
    give_mana_for(t, P0, name);
    let c = t.hand(P0, name);
    t.cast(P0, c).go()
}

#[test]
fn a_card_cast_by_cascade_can_be_kicked_but_mandatory_additional_costs_must_be_paid() {
    cr!("702.85a", "118.9", "118.8a", "601.2b", "601.2f");
    ruling!(
        "Bloodbraid Elf",
        "If you cast a card \"without paying its mana cost,\" you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, you must pay those to cast the card."
    );
    supported("Bloodbraid Elf");
    supported("Burst Lightning");
    supported("Village Rites");
    // An optional additional cost (kicker {4}) may be paid.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Burst Lightning"]);
    cast_from_hand(&mut t, "Bloodbraid Elf");
    t.lands(P0, "Mountain", 4);
    t.answer_yes(P0, true); // cast it
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true)); // kicked
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Bloodbraid Elf", "Burst Lightning"]);
    assert_eq!(tapped_lands(&t, P0), 4 + 4);
    t.resolve();
    assert_eq!(t.life(P1), 16);

    // A mandatory additional cost (sacrificing a creature) must be paid.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Village Rites"]);
    let hand = t.hand_size(P0);
    cast_from_hand(&mut t, "Bloodbraid Elf");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Bloodbraid Elf", "Village Rites"]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 2);

    // Without a creature to sacrifice, the card can't be cast; it goes to the bottom.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Village Rites"]);
    cast_from_hand(&mut t, "Bloodbraid Elf");
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Bloodbraid Elf"]);
    let bottom = t.g.player(P0).library[0];
    assert_eq!(t.obj(bottom).chars.name, "Village Rites");

    // An alternative cost can't be used: a card with bestow is cast as a creature spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    stack_library(&mut t, P0, &["Nyxborn Rollicker"]);
    cast_from_hand(&mut t, "Bloodbraid Elf");
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Bloodbraid Elf", "Nyxborn Rollicker"]);
    let top = *t.g.stack.last().unwrap();
    assert!(t.obj(top).is_creature());
    assert!(!t.obj(top).chars.has_subtype("Aura"));
}

#[test]
fn the_next_spell_gains_cascade_as_it_is_cast_and_cascades() {
    cr!("702.85a", "601.2a", "601.2i");
    ruling!(
        "Sloppity Bilepiper",
        "This means that the next spell you cast gains cascade as you begin to cast it by putting it on the stack, and the cascade ability will trigger when you finish casting that spell."
    );
    supported("Sloppity Bilepiper");
    let mut t = TestGame::new(2);
    // "Jolly Gutpipes — {2}, {T}, Sacrifice a creature: The next creature spell you cast
    // this turn has cascade."
    let piper = t.battlefield(P0, "Sloppity Bilepiper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    let n = t
        .obj(piper)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count();
    assert_eq!(n, 1);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, piper, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // A noncreature spell doesn't get cascade.
    cast_from_hand(&mut t, "Divination");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 0);
    t.resolve_all();
    stack_library(&mut t, P0, &["Llanowar Elves", "Grizzly Bears"]);
    // The next creature spell has cascade on the stack, and its cascade triggers.
    let giant = cast_from_hand(&mut t, "Hill Giant");
    assert!(t.obj(giant).chars.has_keyword(KeywordKind::Cascade));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Hill Giant", "Llanowar Elves"]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Only that one spell had cascade.
    cast_from_hand(&mut t, "Hill Giant");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 0);
}
