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
    ruling!(
        "Agate Instigator",
        "You can pay an offspring cost only once as you cast a spell with offspring. You can't try to pay it multiple times to get more token copies."
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
    ruling!(
        "Agate Instigator",
        "If the spell resolves but the creature with offspring leaves the battlefield before the offspring ability resolves, you'll still create a token copy of it."
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
    ruling!(
        "Agate Instigator",
        "Any \"enters\" abilities of the copied creature will trigger when the token enters. Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the copied creature will also work."
    );
    ruling!(
        "Agate Instigator",
        "The token created by the offspring ability isn't \"cast\", so abilities that trigger when a creature spell is cast won't trigger for the copy."
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

#[test]
fn this_creature_means_each_of_them_itself() {
    cr!("702.175a");
    assert_supported("Coruscation Mage");
    assert_supported("Agate Instigator");
    ruling!(
        "Coruscation Mage",
        "Many creatures with offspring abilities have other abilities that refer to them as “this creature” rather than referring to them by name. This difference is for clarity purposes and does not change the function of any of these abilities."
    );
    ruling!(
        "Agate Instigator",
        "Many creatures with offspring abilities have other abilities that refer to them as \"this creature\" rather than referring to them by name."
    );
    ruling!(
        "Coruscation Mage",
        "Coruscation Mage’s last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    // Coruscation Mage: {1}{R} 2/2, offspring {2}, "Whenever you cast a noncreature spell,
    // this creature deals 1 damage to each opponent."
    let mut t = TestGame::new(2);
    let mage = t.hand(P0, "Coruscation Mage");
    add_mana(&mut t, P0, ManaType::R, 4);
    pay_optional(&mut t, P0, true);
    t.cast(P0, mage).go();
    t.resolve_all();
    let token = tokens_named(&t, P0, "Coruscation Mage");
    assert_eq!(token.len(), 1);
    assert_eq!(t.pt(token[0]), (1, 1));
    // A noncreature spell: each of them deals 1 damage, before the spell resolves.
    let shock = t.hand(P0, "Shock");
    add_mana(&mut t, P0, ManaType::R, 1);
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.settle();
    assert_eq!(t.stack_len(), 3);
    t.resolve();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Agate Instigator: {1}{R} 1/3, offspring {1}{R}, "Whenever another creature you control
    // enters, this creature deals 1 damage to each opponent.": its token is another
    // creature for the original, and not for itself.
    let mut t = TestGame::new(2);
    let agate = t.hand(P0, "Agate Instigator");
    add_mana(&mut t, P0, ManaType::R, 4);
    pay_optional(&mut t, P0, true);
    t.cast(P0, agate).go();
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Agate Instigator").len(), 1);
    assert_eq!(t.life(P1), 19);
    // Another creature enters: both deal 1 damage.
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

/// Casts Rust-Shield Rampager paying its offspring cost and resolves the spell (not the
/// offspring ability). Returns the Rampager.
fn rampager_with_trigger(t: &mut TestGame) -> ObjectId {
    let spell = cast_with_offspring(t, RAMPAGER, true);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.g.current(spell)
}

#[test]
fn the_token_copies_only_the_copiable_values_except_its_1_1() {
    cr!("702.175a", "707.2");
    ruling!(
        "Coruscation Mage",
        "The token copies exactly what was printed on the original creature and nothing else, except it’s a 1/1 (unless that creature is copying something else; see below). It doesn’t copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its types, color, or so on."
    );
    ruling!(
        "Agate Instigator",
        "The token copies exactly what was printed on the original creature and nothing else, except it's a 1/1 (unless that creature is copying something else; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its types, color, or so on."
    );
    let mut t = TestGame::new(2);
    let rampager = rampager_with_trigger(&mut t);
    // In response: a +1/+1 counter, Giant Growth, and it's tapped.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(rampager)]];
    t.g.exec(
        &mtg_engine::ability::Effect::AddCounters {
            what: mtg_engine::ability::Sel::Target(0),
            kind: "+1/+1".into(),
            n: mtg_engine::ability::Value::c(1),
        },
        &mut ctx,
    );
    let growth = t.hand(P0, "Giant Growth");
    add_mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, growth).target(rampager).go();
    t.resolve();
    t.g.tap(rampager);
    t.g.recompute();
    assert_eq!(t.pt(rampager), (8, 8));
    t.resolve_all();
    let token = tokens_named(&t, P0, RAMPAGER);
    assert_eq!(token.len(), 1);
    let o = t.obj(token[0]);
    assert_eq!(t.pt(token[0]), (1, 1));
    assert!(!o.tapped);
    assert_eq!(t.counters(token[0], "+1/+1"), 0);
}

#[test]
fn if_it_copies_something_else_the_token_copies_that() {
    cr!("702.175a", "707.2");
    ruling!(
        "Coruscation Mage",
        "In the rare case where the original creature is copying something else when the offspring ability resolves, the token enters as whatever that creature copied, except it’s a 1/1."
    );
    ruling!(
        "Agate Instigator",
        "In the rare case where the original creature is copying something else when the offspring ability resolves, the token enters as whatever that creature copied, except it's a 1/1."
    );
    let mut t = TestGame::new(2);
    let rampager = rampager_with_trigger(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(rampager)], vec![Entity::Object(giant)]];
    t.g.exec(
        &mtg_engine::ability::Effect::BecomeCopy {
            what: mtg_engine::ability::Sel::Target(0),
            of: mtg_engine::ability::Sel::Target(1),
            duration: mtg_engine::ability::Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert_eq!(t.obj(rampager).chars.name, "Hill Giant");
    t.resolve_all();
    let token = tokens_named(&t, P0, "Hill Giant");
    assert_eq!(token.len(), 1);
    assert_eq!(t.pt(token[0]), (1, 1));
    assert!(tokens_named(&t, P0, RAMPAGER).is_empty());
}

#[test]
fn without_offspring_as_it_enters_the_paid_cost_does_nothing() {
    cr!("702.175a");
    ruling!(
        "Coruscation Mage",
        "In the rare case where the creature doesn’t have the offspring ability when it enters, the ability won’t trigger even if you paid the offspring cost."
    );
    ruling!(
        "Agate Instigator",
        "In the rare case where the creature doesn't have the offspring ability when it enters, the ability won't trigger even if you paid the offspring cost."
    );
    // Humility: "All creatures lose all abilities and have base power and toughness 1/1."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Humility");
    let spell = cast_with_offspring(&mut t, RAMPAGER, true);
    assert_eq!(pool(&t, P0), 2);
    t.resolve();
    assert!(t.on_battlefield(t.g.current(spell)));
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(tokens_named(&t, P0, RAMPAGER).is_empty());
    // Losing its abilities after the ability triggered doesn't stop it: the cost was paid.
    let mut t = TestGame::new(2);
    rampager_with_trigger(&mut t);
    t.battlefield(P1, "Humility");
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, RAMPAGER).len(), 1);
}

#[test]
fn creature_spells_you_cast_gain_offspring_as_you_cast_them() {
    cr!("702.175a", "610.5", "400.7b");
    assert_supported("Zinnia, Valley's Voice");
    ruling!(
        "Zinnia, Valley's Voice",
        "You can pay an offspring cost only once as you cast a spell with offspring."
    );
    // Zinnia, Valley's Voice: 1/3 flying, "Zinnia gets +X/+0, where X is the number of
    // other creatures you control with base power 1." "Creature spells you cast gain
    // offspring {2} as you cast them."
    let mut t = TestGame::new(2);
    let zinnia = t.battlefield(P0, "Zinnia, Valley's Voice");
    assert_eq!(t.pt(zinnia), (1, 3));
    // Grizzly Bears ({1}{G}) with offspring {2}.
    let bears = cast_with_offspring(&mut t, "Grizzly Bears", true);
    assert_eq!(optional_costs_asked(&t, P0).len(), 1);
    assert_eq!(pool(&t, P0), 4);
    t.resolve_all();
    let all = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(all.len(), 2);
    let token = *all.iter().find(|o| t.g.obj(**o).is_token()).unwrap();
    assert_eq!(t.pt(token), (1, 1));
    // The permanent the spell became still has offspring (it functions on the
    // battlefield); the 1/1 token has base power 1.
    let permanent = t.g.current(bears);
    assert!(t
        .obj(permanent)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Offspring));
    assert_eq!(t.pt(zinnia), (2, 3));
    // A noncreature spell doesn't gain offspring.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zinnia, Valley's Voice");
    let spell = t.hand(P0, "Divination");
    add_mana(&mut t, P0, ManaType::U, 3);
    t.cast(P0, spell).go();
    assert!(optional_costs_asked(&t, P0).is_empty());
}
