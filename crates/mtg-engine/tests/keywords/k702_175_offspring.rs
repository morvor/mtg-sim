//! CR 702.175 Offspring.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_052_066::destroy;
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

const RAMPAGER: &str = "Rust-Shield Rampager";

/// Casts `name` from P0's hand, paying (or not) its offspring cost, with enough mana.
fn cast_with_offspring(t: &mut TestGame, name: &str, pay: bool) -> ObjectId {
    let card = t.hand(P0, name);
    add_mana(t, P0, ManaType::G, 8);
    pay_optional(t, P0, pay);
    t.cast(P0, card).go()
}

#[test]
fn paying_the_offspring_cost_creates_a_1_1_token_copy() {
    cr!("702.175", "702.175a");
    assert_supported(RAMPAGER);
    // Rust-Shield Rampager: {3}{G} 4/4, offspring {2}.
    let mut t = TestGame::new(2);
    let spell = cast_with_offspring(&mut t, RAMPAGER, true);
    assert_eq!(
        optional_costs_asked(&t, P0),
        vec!["offspring#1".to_string()]
    );
    // An additional cost: {3}{G} + {2} was paid; the mana value is still 4.
    assert_eq!(pool(&t, P0), 2);
    assert_eq!(t.g.mana_value_of(spell), 4);
    t.resolve_all();
    let rampagers = t.named_on_battlefield(RAMPAGER);
    assert_eq!(rampagers.len(), 2);
    let token = tokens_named(&t, P0, RAMPAGER);
    assert_eq!(token.len(), 1);
    assert_eq!(t.pt(token[0]), (1, 1));
    // It copies the original's abilities.
    assert_eq!(
        t.obj(token[0]).chars.abilities.len(),
        t.obj(t.g.current(spell)).chars.abilities.len()
    );
    // Not paid: no token.
    let mut t = TestGame::new(2);
    cast_with_offspring(&mut t, RAMPAGER, false);
    assert_eq!(pool(&t, P0), 4);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield(RAMPAGER).len(), 1);
}

#[test]
fn the_offspring_cost_can_be_paid_only_once() {
    cr!("702.175a");
    ruling!(
        "Pawpatch Recruit",
        "You can pay an offspring cost only once as you cast a spell with offspring. You can’t try to pay it multiple times to get more token copies."
    );
    let mut t = TestGame::new(2);
    pay_optional(&mut t, P0, true);
    cast_with_offspring(&mut t, RAMPAGER, true);
    assert_eq!(optional_costs_asked(&t, P0).len(), 1);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, RAMPAGER).len(), 1);
}

#[test]
fn a_countered_spell_makes_no_token() {
    cr!("702.175a");
    ruling!(
        "Pawpatch Recruit",
        "If the spell is countered, the offspring ability will not trigger, and no token will be created."
    );
    let mut t = TestGame::new(2);
    let spell = cast_with_offspring(&mut t, RAMPAGER, true);
    let counter = t.hand(P1, "Counterspell");
    add_mana(&mut t, P1, ManaType::U, 2);
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert!(t.named_on_battlefield(RAMPAGER).is_empty());
}

#[test]
fn the_token_is_created_even_if_the_creature_left_the_battlefield() {
    cr!("702.175a");
    ruling!(
        "Pawpatch Recruit",
        "If the spell resolves but the creature with offspring leaves the battlefield before the offspring ability resolves, you’ll still create a token copy of it."
    );
    let mut t = TestGame::new(2);
    let spell = cast_with_offspring(&mut t, RAMPAGER, true);
    t.resolve();
    // The offspring ability is on the stack; the Rampager dies in response.
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, spell);
    assert!(t.in_graveyard(P0, RAMPAGER));
    t.resolve_all();
    let token = tokens_named(&t, P0, RAMPAGER);
    assert_eq!(token.len(), 1);
    assert_eq!(t.pt(token[0]), (1, 1));
}

#[test]
fn the_tokens_enters_abilities_trigger_but_it_wasnt_cast() {
    cr!("702.175a");
    assert_supported("Intrepid Rabbit");
    ruling!(
        "Pawpatch Recruit",
        "Any “enters” abilities of the copied creature will trigger when the token enters."
    );
    ruling!(
        "Pawpatch Recruit",
        "The token created by the offspring ability isn’t “cast”, so abilities that trigger when a creature spell is cast won’t trigger for the copy."
    );
    // Intrepid Rabbit: {2}{W} 3/2, offspring {1}, "When this creature enters, target
    // creature you control gets +1/+1 until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let card = t.hand(P0, "Intrepid Rabbit");
    add_mana(&mut t, P0, ManaType::W, 4);
    pay_optional(&mut t, P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, card).go();
    t.resolve_all();
    // Both the Rabbit's and the token's enters abilities resolved.
    assert_eq!(t.pt(bears), (4, 4));
    let token = tokens_named(&t, P0, "Intrepid Rabbit");
    assert_eq!(token.len(), 1);
    // The token wasn't cast: its own offspring ability doesn't trigger.
    assert_eq!(t.named_on_battlefield("Intrepid Rabbit").len(), 2);
    assert!(t.obj(token[0]).cast.as_ref().is_none_or(|c| !c.was_cast));
}

#[test]
fn a_copy_of_a_creature_whose_offspring_cost_was_paid_makes_no_token() {
    cr!("702.175a", "400.7d");
    let mut t = TestGame::new(2);
    cast_with_offspring(&mut t, RAMPAGER, true);
    t.resolve_all();
    let original = t
        .named_on_battlefield(RAMPAGER)
        .into_iter()
        .find(|id| !t.obj(*id).is_token())
        .unwrap();
    let clone = t.hand(P0, "Clone");
    add_mana(&mut t, P0, ManaType::U, 4);
    t.answer_choose(P0, &[Entity::Object(original)]);
    t.cast(P0, clone).go();
    t.resolve_all();
    // The Clone wasn't cast paying an offspring cost.
    assert_eq!(t.named_on_battlefield(RAMPAGER).len(), 3);
    assert_eq!(tokens_named(&t, P0, RAMPAGER).len(), 1);
}

#[test]
fn each_instance_of_offspring_is_paid_and_triggers_separately() {
    cr!("702.175b");
    // A {0} creature with two offspring abilities (as a creature spell with offspring
    // cast with Zinnia, Valley's Voice would have).
    let mut def = custom_card(
        "Twin Litter",
        "Creature — Rabbit",
        Some((3, 3)),
        "Offspring {1}\nOffspring {2}",
    );
    def.faces[0].chars.mana_cost = Some(mtg_engine::mana::ManaCost::generic(0));
    for (first, second, tokens) in [
        (false, false, 0),
        (true, false, 1),
        (false, true, 1),
        (true, true, 2),
    ] {
        let mut t = TestGame::new(2);
        let card = t.custom(P0, def.clone(), Zone::Hand(P0));
        add_mana(&mut t, P0, ManaType::G, 3);
        pay_optional(&mut t, P0, first);
        pay_optional(&mut t, P0, second);
        t.cast(P0, card).go();
        assert_eq!(
            optional_costs_asked(&t, P0),
            vec!["offspring#1".to_string(), "offspring#2".to_string()]
        );
        // Each payment costs its own instance's cost.
        let spent = [first as usize, 2 * second as usize].iter().sum::<usize>();
        assert_eq!(pool(&t, P0), 3 - spent);
        t.resolve_all();
        assert_eq!(
            tokens_named(&t, P0, "Twin Litter").len(),
            tokens,
            "paid {first} {second}"
        );
    }
}
