//! Rulings batch S25 — a copy of a permanent spell becomes a token as it resolves
//! (CR 707.10f, 608.3f): the rules for a permanent spell becoming a permanent apply, and
//! the token isn't "created" (CR 701.7a), so effects that modify token creation don't
//! apply to it.

use crate::r_s01_common::supported;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// Tokens named `name` on the battlefield.
fn tokens_named(t: &TestGame, name: &str) -> usize {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|id| t.obj(*id).is_token())
        .count()
}

#[test]
fn a_storm_copy_of_an_aura_becomes_a_token_that_isnt_created() {
    cr!("707.10f", "608.3f", "701.7a", "702.40a");
    ruling!(
        "Tempest Technique",
        "A resolving copy of a permanent spell becomes a token. That token isn’t “created” and won’t interact with abilities that care about tokens being created."
    );
    supported("Tempest Technique");
    supported("Parallel Lives");
    // Storm; "Enchant creature you control"; "Enchanted creature gets +1/+1 for each
    // enchantment you control." Parallel Lives: "If an effect would create one or more
    // tokens under your control, it creates twice that many of those tokens instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Parallel Lives");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    cast_new(&mut t, P0, "Tempest Technique", &[Entity::Object(bears)]);
    // The storm ability copies it once (one spell was cast before it this turn).
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Tempest Technique"), 1);
    assert_eq!(t.named_on_battlefield("Tempest Technique").len(), 2);
    // Both Auras enchant the Bears: +3/+3 each (three enchantments).
    assert_eq!(t.pt(bears), (2 + 6, 2 + 6));
}

#[test]
fn a_conspired_copy_of_an_enchantment_becomes_a_token_that_isnt_created() {
    cr!("707.10f", "608.3f", "701.7a", "702.78a");
    ruling!(
        "Raiding Schemes",
        "A resolving copy of a permanent spell becomes a token. That token isn't \"created\" and won't interact with abilities that care about tokens being created."
    );
    supported("Raiding Schemes");
    supported("Glorious Anthem");
    // "Each noncreature spell you cast has conspire." Glorious Anthem (white): "Creatures
    // you control get +1/+1."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raiding Schemes");
    t.battlefield(P0, "Parallel Lives");
    let l1 = t.battlefield(P0, "Savannah Lions");
    let l2 = t.battlefield(P0, "Savannah Lions");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(l1), Entity::Object(l2)]);
    cast_new(&mut t, P0, "Glorious Anthem", &[]);
    t.resolve_all();
    assert_eq!(tokens_named(&t, "Glorious Anthem"), 1);
    assert_eq!(t.named_on_battlefield("Glorious Anthem").len(), 2);
    assert_eq!(t.pt(l1), (4, 3));
}

#[test]
fn a_copy_of_a_creature_spell_enters_as_a_token_with_its_enters_abilities() {
    cr!("707.10f", "608.3f", "603.6a");
    ruling!(
        "Reflections of Littjara",
        "As a copy of a permanent spell resolves, it's put onto the battlefield as a token rather than putting a copy of the spell onto the battlefield. The rules that apply to a permanent spell becoming a permanent apply to a copy of a spell becoming a token."
    );
    supported("Reflections of Littjara");
    supported("Elvish Visionary");
    // "As this enchantment enters, choose a creature type. Whenever you cast a spell of the
    // chosen type, copy that spell." Elvish Visionary (an Elf): "When this creature
    // enters, draw a card."
    let mut t = TestGame::new(2);
    choose_creature_type(&mut t, P0, "Elf");
    t.enter(P0, "Reflections of Littjara");
    t.settle();
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Elvish Visionary", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Elvish Visionary").len(), 2);
    assert_eq!(tokens_named(&t, "Elvish Visionary"), 1);
    // Both entered and drew.
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn a_double_down_copy_isnt_created() {
    cr!("707.10f", "608.3f", "701.7a", "614.1a");
    ruling!(
        "Double Down",
        "The token that a resolving copy of a permanent spell becomes isn't \"created.\" Abilities that refer to a token being created won't interact with the copy resolving."
    );
    supported("Double Down");
    supported("Royal Assassin");
    // "Whenever you cast an outlaw spell, copy that spell." Royal Assassin is an Assassin.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Double Down");
    t.battlefield(P0, "Parallel Lives");
    cast_new(&mut t, P0, "Royal Assassin", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Royal Assassin").len(), 2);
    assert_eq!(tokens_named(&t, "Royal Assassin"), 1);
}
