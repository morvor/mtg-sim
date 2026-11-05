//! Rulings batch P019 — abilities that grant cascade (CR 702.85): Wild-Magic Sorcerer ("The
//! first spell you cast from exile each turn has cascade.") and Sloppity Bilepiper ("The
//! next creature spell you cast this turn has cascade."). Cascade compares mana values
//! (CR 202.3), casts without paying the mana cost (CR 601.2b, 118.9) and each instance
//! triggers separately (CR 702.85c).

use crate::r_s01_common::{stack_library, supported, triggers_on_stack};
use crate::r_s18_common::ADVENTURE;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const SORCERER: &str = "Wild-Magic Sorcerer";
const GIANT: &str = "Bonecrusher Giant // Stomp";

/// Names of the spells on the stack, bottom first.
fn spell_names(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .filter(|s| t.g.obj(**s).is_spell())
        .map(|s| t.g.obj(*s).chars.name.to_string())
        .collect()
}

/// P0 casts Stomp ("Stomp deals 2 damage to any target.", {1}{R}) from Bonecrusher Giant
/// at P1; it resolves and the card goes on an adventure (exile). Returns the card.
fn stomp(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Mountain", 2);
    let card = t.hand(P0, GIANT);
    t.cast(P0, card)
        .method(ADVENTURE)
        .target(Entity::Player(P1))
        .go();
    t.settle();
    assert_eq!(triggers_on_stack(t, "Cascade"), 0);
    t.resolve_all();
    let card = t.g.current(card);
    assert_eq!(t.zone(card), Zone::Exile);
    card
}

/// P0 casts Bonecrusher Giant ({2}{R}, mana value 3) from exile; the triggers go on the
/// stack.
fn giant_from_exile(t: &mut TestGame, card: ObjectId) -> ObjectId {
    t.lands(P0, "Mountain", 3);
    let s = t.cast(P0, card).go();
    t.settle();
    s
}

#[test]
fn the_first_spell_cast_from_exile_has_cascade_even_if_not_the_first_spell() {
    cr!("702.85a");
    ruling!(
        "Wild-Magic Sorcerer",
        "The first spell you cast from exile each turn will have cascade, even if it's not the first spell you've cast that turn."
    );
    supported(SORCERER);
    supported(GIANT);
    // Stomp (cast from hand) gets no cascade; then the Giant, from exile, cascades into
    // the Bears.
    let mut t = TestGame::new(2);
    t.battlefield(P0, SORCERER);
    let card = stomp(&mut t);
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    let giant = giant_from_exile(&mut t, card);
    assert!(t.obj(giant).chars.has_keyword(KeywordKind::Cascade));
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Bonecrusher Giant", "Grizzly Bears"]);
}

#[test]
fn wild_magic_sorcerer_arriving_after_a_spell_was_cast_from_exile_grants_nothing_that_turn() {
    cr!("702.85a");
    ruling!(
        "Wild-Magic Sorcerer",
        "However, if you cast a spell from exile and then later that turn Wild-Magic Sorcerer comes under your control, Wild-Mage Sorcerer won't give cascade to any other spell you cast from exile that turn."
    );
    let mut t = TestGame::new(2);
    let a = stomp(&mut t);
    let b = stomp(&mut t);
    giant_from_exile(&mut t, a);
    t.resolve_all();
    t.battlefield(P0, SORCERER);
    let second = giant_from_exile(&mut t, b);
    assert!(!t.obj(second).chars.has_keyword(KeywordKind::Cascade));
    assert_eq!(triggers_on_stack(&t, "Cascade"), 0);
}

#[test]
fn two_wild_magic_sorcerers_give_two_instances_of_cascade() {
    cr!("702.85a", "702.85c");
    ruling!(
        "Wild-Magic Sorcerer",
        "If you control more than one Wild-Magic Sorcerer, the first spell you cast from exile will have that many instances of cascade, each of which will trigger separately."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, SORCERER);
    t.battlefield(P0, SORCERER);
    let card = stomp(&mut t);
    stack_library(&mut t, P0, &["Grizzly Bears", "Llanowar Elves"]);
    giant_from_exile(&mut t, card);
    assert_eq!(triggers_on_stack(&t, "Cascade"), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
}

/// P0 foretells Doomskar ({3}{W}{W}, mana value 5; "Foretell {1}{W}{W}") on turn 1, and on
/// P0's next turn (Wild-Magic Sorcerer on the battlefield) casts it from exile for its
/// foretell cost, with `library` on top of the library. Returns the spell.
fn foretold_doomskar(library: &[&str]) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Doomskar");
    let card = crate::r_p216_common::foretell(&mut t, P0, card);
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, SORCERER);
    stack_library(&mut t, P0, library);
    t.lands(P0, "Plains", 3);
    let s = t
        .cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Foretell))
        .go();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
    (t, s)
}

#[test]
fn cascade_uses_the_mana_value_not_the_cost_paid() {
    cr!("202.3", "702.85a", "118.9c");
    ruling!(
        "Wild-Magic Sorcerer",
        "A spell's mana value is determined only by its mana cost. Ignore any alternative costs, additional costs, cost increases, or cost reductions. For example, Wild-Magic Sorcerer's mana value is always 4."
    );
    supported("Doomskar");
    // Doomskar was cast for {1}{W}{W}, but its mana value is 5: Hill Giant (4) is cast.
    let (mut t, _) = foretold_doomskar(&["Hill Giant"]);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Doomskar", "Hill Giant"]);
}

#[test]
fn cascade_cant_cast_a_face_with_a_greater_mana_value() {
    cr!("702.85a", "712.11b");
    ruling!(
        "Wild-Magic Sorcerer",
        "Due to a recent rules change to cascade, not only do you stop exiling cards if you exile a nonland card with lesser mana value than the spell with cascade, but the resulting spell you cast must also have lesser mana value. Previously, in cases where a card's mana value differed from the resulting spell, such as with some modal double-faced cards or cards with an Adventure, you could cast a spell with a higher mana value than the exiled card."
    );
    // Selfless Glyphweaver ({2}{W}, 3) // Deadly Vanity ({5}{B}{B}, 7): only the
    // Glyphweaver can be cast (7 isn't less than Doomskar's 5).
    let (mut t, _) = foretold_doomskar(&["Selfless Glyphweaver"]);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve();
    assert!(!t.asked()[from..].iter().any(|(_, d)| matches!(d,
        Decision::ChooseOption { options, .. } if options.iter().any(|o| o.contains("Deadly Vanity")))));
    assert_eq!(spell_names(&t), vec!["Doomskar", "Selfless Glyphweaver"]);
}

#[test]
fn cascading_into_a_second_wild_magic_sorcerer_doesnt_cascade_again() {
    cr!("702.85a");
    ruling!(
        "Wild-Magic Sorcerer",
        "If, while resolving the cascade ability from the first spell you cast from exile in a turn, you cascade into a second Wild-Magic Sorcerer, the spell will not cascade again."
    );
    // The Sorcerer (4) is cast from exile, but it's the second spell cast from exile.
    let (mut t, _) = foretold_doomskar(&[SORCERER]);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Doomskar", SORCERER]);
    let top = *t.g.stack.last().unwrap();
    assert!(!t.obj(top).chars.has_keyword(KeywordKind::Cascade));
    assert_eq!(triggers_on_stack(&t, "Cascade"), 0);
}

#[test]
fn a_card_cast_by_its_cascade_pays_mandatory_additional_costs_but_no_alternative_cost() {
    cr!("702.85a", "118.9", "601.2b", "601.2f");
    ruling!(
        "Wild-Magic Sorcerer",
        "If you cast a card “without paying its mana cost,” you can't choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, you must pay those to cast the card."
    );
    supported("Village Rites");
    // Village Rites ("As an additional cost to cast this spell, sacrifice a creature.
    // Draw two cards."): the Bears are sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, SORCERER);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = stomp(&mut t);
    stack_library(&mut t, P0, &["Village Rites"]);
    giant_from_exile(&mut t, card);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Bonecrusher Giant", "Village Rites"]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // Nyxborn Rollicker (bestow {1}{R}, an alternative cost) is cast as a creature.
    let mut t = TestGame::new(2);
    t.battlefield(P0, SORCERER);
    t.battlefield(P0, "Grizzly Bears");
    let card = stomp(&mut t);
    stack_library(&mut t, P0, &["Nyxborn Rollicker"]);
    giant_from_exile(&mut t, card);
    t.answer_yes(P0, true);
    t.resolve();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(top).chars.name, "Nyxborn Rollicker");
    assert!(t.obj(top).is_creature());
    assert!(!t.obj(top).chars.has_subtype("Aura"));
}

/// P0 activates Sloppity Bilepiper's "Jolly Gutpipes — {2}, {T}, Sacrifice a creature: The
/// next creature spell you cast this turn has cascade.", sacrificing a new Grizzly Bears.
fn gutpipes(t: &mut TestGame, piper: ObjectId) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, piper, 0, &[]).unwrap();
}

#[test]
fn bilepipers_effect_applies_only_to_a_creature_spell_cast_after_it_resolves() {
    cr!("611.2c", "702.85a");
    ruling!(
        "Sloppity Bilepiper",
        "As Sloppity Bilepiper's activated ability resolves, it creates a continuous effect that won't begin to apply until you start casting a creature spell this turn. It applies to that spell and the timestamp of that effect is the time that the object was put on the stack."
    );
    // A creature spell already on the stack as the ability resolves doesn't get cascade;
    // the next one cast does.
    let mut t = TestGame::new(2);
    let piper = t.battlefield(P0, "Sloppity Bilepiper");
    let first = cast_new(&mut t, P0, "Hill Giant", &[]);
    gutpipes(&mut t, piper);
    t.resolve();
    assert!(t.g.stack.contains(&first));
    assert!(!t.obj(first).chars.has_keyword(KeywordKind::Cascade));
    t.resolve_all();
    stack_library(&mut t, P0, &["Llanowar Elves"]);
    let second = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    assert!(t.obj(second).chars.has_keyword(KeywordKind::Cascade));
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
}

#[test]
fn two_bilepiper_activations_give_two_instances_of_cascade() {
    cr!("702.85a", "702.85c");
    ruling!(
        "Sloppity Bilepiper",
        "If you activate the ability of more than one Sloppity Bilepiper (or the same one more than once) in the same turn, the next creature spell you cast will have that many instances of cascade, each of which will trigger separately."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Sloppity Bilepiper");
    let b = t.battlefield(P0, "Sloppity Bilepiper");
    gutpipes(&mut t, a);
    gutpipes(&mut t, b);
    t.resolve_all();
    stack_library(&mut t, P0, &["Llanowar Elves", "Grizzly Bears"]);
    cast_new(&mut t, P0, "Hill Giant", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Llanowar Elves").len(), 1);
    // The Bears: one sacrificed for each activation, one cast by cascade.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
