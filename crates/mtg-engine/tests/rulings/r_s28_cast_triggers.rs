//! Rulings batch S28 — abilities that trigger when a spell is cast (CR 601.2i, 603.2,
//! 603.3): they trigger once the spell's targets are chosen and it becomes cast, go on the
//! stack on top of it, and resolve first — even if the spell is countered (CR 701.6a).
//! Also the Hatchlings, whose two color triggers both trigger for a two-color spell.

use crate::r_s01_common::{supported, tokens};
use crate::r_s04_common::{on_stack, stack_items, top_of_stack};
use crate::r_s28_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::*;

/// `p` casts the counterspell `name` targeting `spell` (lands for it are added).
fn counter_with(t: &mut TestGame, p: PlayerId, name: &str, spell: ObjectId) {
    t.answer_targets(p, &[Entity::Object(spell)]);
    cast_card(t, p, name);
}

/// Whether the object on top of the stack is a triggered ability.
fn trigger_on_top(t: &TestGame) -> bool {
    matches!(
        t.g.obj(top_of_stack(t)).stack.as_deref().map(|s| &s.kind),
        Some(mtg_engine::object::StackKind::Triggered { .. })
    )
}

#[test]
fn beast_whisperers_trigger_resolves_even_if_the_creature_spell_is_countered() {
    cr!("603.3", "601.2i", "701.6a");
    ruling!(
        "Beast Whisperer",
        "An ability that triggers when a player casts a spell resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Beast Whisperer");
    // "Whenever you cast a creature spell, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Beast Whisperer");
    let hand = t.hand_size(P0);
    let bears = cast_card(&mut t, P0, "Grizzly Bears");
    t.settle();
    assert_eq!(stack_items(&t)[0], "Grizzly Bears");
    assert!(trigger_on_top(&t));
    // It resolves before the spell.
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Stack);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // The creature spell is countered: the card is still drawn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Beast Whisperer");
    let hand = t.hand_size(P0);
    let bears = cast_card(&mut t, P0, "Grizzly Bears");
    t.settle();
    counter_with(&mut t, P1, "Essence Scatter", bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn hero_of_the_prides_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Hero of the Pride",
        "An ability that triggers when you cast a spell resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Hero of the Pride");
    // "Whenever you cast a spell that targets this creature, creatures you control get
    // +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of the Pride");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(hero)]);
    let growth = cast_card(&mut t, P0, "Giant Growth");
    t.settle();
    assert!(trigger_on_top(&t));
    counter_with(&mut t, P1, "Negate", growth);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Giant Growth"));
    assert_eq!(t.pt(hero), (3, 2));
    assert_eq!(t.pt(bears), (3, 2));
}

#[test]
fn rockslide_sorcerers_trigger_comes_after_the_spells_targets_and_resolves_first() {
    cr!("601.2c", "601.2i", "603.3", "608.2b");
    ruling!(
        "Rockslide Sorcerer",
        "An ability that triggers when a player casts a spell resolves before the spell that caused it to trigger, but after targets have been chosen for that spell (if it has any targets). The ability resolves even if that spell is countered."
    );
    supported("Rockslide Sorcerer");
    // "Whenever you cast an instant, sorcery, or Wizard spell, this creature deals 1 damage
    // to any target."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rockslide Sorcerer");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let from = t.asked().len();
    // The Bolt's target is chosen as it's cast; the trigger's target afterward.
    t.answer_targets(P0, &[Entity::Object(elves)]);
    cast_card(&mut t, P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.settle();
    let target_prompts = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseTargets { .. }))
        .count();
    assert_eq!(target_prompts, 2);
    assert!(trigger_on_top(&t));
    // The trigger resolves first and kills the Elves, so the Bolt has no legal target.
    t.resolve();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P1), 20);
    // The Bolt is countered: the trigger still deals its damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rockslide Sorcerer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let bolt = cast_card(&mut t, P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.settle();
    counter_with(&mut t, P1, "Counterspell", bolt);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn young_pyromancers_token_comes_even_if_the_spell_is_countered() {
    cr!("601.2c", "603.3", "701.6a");
    ruling!(
        "Young Pyromancer",
        "An ability that triggers when a player casts a spell resolves before the spell that caused it to trigger, but after targets have been chosen for that spell. It resolves even if that spell is countered."
    );
    supported("Young Pyromancer");
    // "Whenever you cast an instant or sorcery spell, create a 1/1 red Elemental creature
    // token."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let bolt = cast_card(&mut t, P0, "Lightning Bolt");
    t.settle();
    assert!(trigger_on_top(&t));
    // The token is created before the Bolt resolves.
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.zone(bolt), mtg_engine::object::Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Countered: the token is still created.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let bolt = cast_card(&mut t, P0, "Lightning Bolt");
    t.settle();
    counter_with(&mut t, P1, "Counterspell", bolt);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_monuments_token_comes_before_the_creature_spell_resolves_even_if_countered() {
    cr!("603.3", "701.6a");
    ruling!(
        "Oketra's Monument",
        "Each Monument's triggered ability triggers as the creature spell is cast and resolves before that creature spell resolves. The ability will resolve even if that creature spell is countered."
    );
    supported("Oketra's Monument");
    // "Whenever you cast a creature spell, create a 1/1 white Warrior creature token with
    // vigilance."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oketra's Monument");
    let bears = cast_card(&mut t, P0, "Grizzly Bears");
    t.settle();
    assert_eq!(on_stack(&t, "Whenever you cast a creature spell"), 1);
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.zone(bears), mtg_engine::object::Zone::Stack);
    // Countered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Oketra's Monument");
    let bears = cast_card(&mut t, P0, "Grizzly Bears");
    t.settle();
    counter_with(&mut t, P1, "Essence Scatter", bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn clarion_spirit_counts_a_countered_first_spell_and_triggers_once() {
    cr!("603.2", "603.3");
    ruling!(
        "Clarion Spirit",
        "The triggered ability can trigger only once each turn. The ability will resolve before the second spell does. It doesn’t matter if the first spell you cast that turn has resolved, was countered, or is still on the stack."
    );
    supported("Clarion Spirit");
    // "Whenever you cast your second spell each turn, create a 1/1 white Spirit creature
    // token with flying."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Clarion Spirit");
    // The first spell is countered.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let first = cast_card(&mut t, P0, "Shock");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    counter_with(&mut t, P1, "Counterspell", first);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shock"));
    // The second spell: the token is created before it resolves.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let second = cast_card(&mut t, P0, "Shock");
    t.settle();
    assert!(trigger_on_top(&t));
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    assert_eq!(t.zone(second), mtg_engine::object::Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // A third spell doesn't trigger it.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_card(&mut t, P0, "Shock");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1);
}

#[test]
fn a_two_color_spell_triggers_both_of_belligerent_hatchlings_abilities() {
    cr!("603.2", "603.3b");
    ruling!(
        "Belligerent Hatchling",
        "If you cast a spell that's both of the listed colors, both abilities will trigger. You'll remove a total of two -1/-1 counters from the Hatchling."
    );
    supported("Belligerent Hatchling");
    supported("Lightning Helix");
    let mut t = TestGame::new(2);
    let hatchling = t.enter(P0, "Belligerent Hatchling");
    assert_eq!(t.counters(hatchling, "-1/-1"), 4);
    assert_eq!(t.pt(hatchling), (2, 2));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_card(&mut t, P0, "Lightning Helix");
    t.settle();
    assert_eq!(on_stack(&t, "remove a -1/-1 counter"), 2);
    t.resolve_all();
    assert_eq!(t.counters(hatchling, "-1/-1"), 2);
    assert_eq!(t.pt(hatchling), (4, 4));
}

#[test]
fn a_two_color_spell_triggers_both_of_noxious_hatchlings_abilities() {
    cr!("603.2", "603.3b");
    ruling!(
        "Noxious Hatchling",
        "If you cast a spell that’s both of the listed colors, both abilities will trigger. You’ll remove a total of two -1/-1 counters from the Hatchling."
    );
    supported("Noxious Hatchling");
    supported("Putrefy");
    let mut t = TestGame::new(2);
    let hatchling = t.enter(P0, "Noxious Hatchling");
    assert_eq!(t.counters(hatchling, "-1/-1"), 4);
    let victim = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(victim)]);
    cast_card(&mut t, P0, "Putrefy");
    t.settle();
    assert_eq!(on_stack(&t, "remove a -1/-1 counter"), 2);
    t.resolve_all();
    assert_eq!(t.counters(hatchling, "-1/-1"), 2);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn a_hatchling_trigger_with_no_counters_to_remove_does_nothing() {
    cr!("603.2", "608.2");
    ruling!(
        "Belligerent Hatchling",
        "If there are no -1/-1 counters on it when the triggered ability resolves, the ability does nothing. There is no penalty for not being able to remove a counter."
    );
    let mut t = TestGame::new(2);
    let hatchling = t.enter(P0, "Belligerent Hatchling");
    t.g.remove_counters(Entity::Object(hatchling), "-1/-1", 4);
    t.g.recompute();
    assert_eq!(t.pt(hatchling), (6, 6));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    cast_card(&mut t, P0, "Shock");
    t.settle();
    assert_eq!(on_stack(&t, "remove a -1/-1 counter"), 1);
    t.resolve_all();
    assert!(t.on_battlefield(hatchling));
    assert_eq!(t.counters(hatchling, "-1/-1"), 0);
    assert_eq!(t.pt(hatchling), (6, 6));
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn hidden_gibbons_becomes_a_creature_even_if_the_instant_is_countered() {
    cr!("603.3", "603.4", "701.6a");
    ruling!(
        "Hidden Gibbons",
        "It changes into a creature even if the spell is countered."
    );
    supported("Hidden Gibbons");
    // "When an opponent casts an instant spell, if this permanent is an enchantment, it
    // becomes a 4/4 Ape creature."
    let mut t = TestGame::new(2);
    let gibbons = t.battlefield(P0, "Hidden Gibbons");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    let bolt = cast_card(&mut t, P1, "Lightning Bolt");
    t.settle();
    assert!(trigger_on_top(&t));
    counter_with(&mut t, P0, "Counterspell", bolt);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    assert_eq!(t.life(P0), 20);
    assert!(t.obj_now(gibbons).is(mtg_engine::types::CardType::Creature));
    assert_eq!(t.pt(gibbons), (4, 4));
    assert!(t.obj_now(gibbons).chars.has_subtype("Ape"));
}
