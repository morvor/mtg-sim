//! Rulings batch S29 — countering (CR 701.6): abilities that trigger on a spell targeting
//! a creature resolve first and even if that spell is countered (CR 603.3, 405.5); spells
//! that can't be countered can still be targeted by counterspells, whose other effects
//! still happen (CR 101.2, 608.2c); countering a modal "choose one that hasn't been
//! chosen" ability (CR 700.2b) or a delayed triggered ability (CR 603.7b).

use crate::r_s01_common::supported;
use crate::r_s04_common::{next_upkeep, top_of_stack};
use crate::r_s07_common::chosen_modes;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether the stack object is a triggered ability from `source`.
fn is_trigger_of(t: &TestGame, id: ObjectId, source: ObjectId) -> bool {
    matches!(
        t.g.obj(id).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { source: s, .. }) if *s == source
    )
}

#[test]
fn a_becomes_the_target_trigger_resolves_first_and_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Tectonic Giant",
        "An ability that triggers when a creature becomes the target of a spell resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    supported("Tectonic Giant");
    supported("Swarm Shambler");
    // Swarm Shambler: "Whenever a creature you control with a +1/+1 counter on it becomes
    // the target of a spell an opponent controls, create a 1/1 green Insect creature
    // token." P1 Shocks the Shambler (a 1/1 with its counter): the Insect comes first.
    let mut t = TestGame::new(2);
    let shambler = t.battlefield(P0, "Swarm Shambler");
    put_counters(&mut t, shambler, counters::PLUS1, 1);
    t.set_step(P1, Step::PrecombatMain);
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Object(shambler)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(is_trigger_of(&t, top_of_stack(&t), shambler));
    t.resolve();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    assert_eq!(t.zone(shock), mtg_engine::object::Zone::Stack);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Swarm Shambler"));
    // Tectonic Giant: "Whenever this creature attacks or becomes the target of a spell an
    // opponent controls, choose one — • This creature deals 3 damage to each opponent.
    // ..." P1's Shock is countered by P0's Counterspell: the trigger still resolves.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Tectonic Giant");
    t.set_step(P1, Step::PrecombatMain);
    choose_modes(&mut t, P0, &[0]);
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Object(giant)]);
    t.settle();
    let trigger = top_of_stack(&t);
    assert!(is_trigger_of(&t, trigger, giant));
    cast_new(&mut t, P0, "Counterspell", &[Entity::Object(shock)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Shock"), "the Shock was countered");
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn a_counterspell_resolves_but_doesnt_counter_an_uncounterable_spell() {
    cr!("101.2", "113.6g", "701.6a");
    ruling!(
        "Blurred Mongoose",
        "Counterspells can be cast that target it, but when they resolve they simply don’t counter it since it can’t be countered."
    );
    supported("Blurred Mongoose");
    supported("Obliterate");
    // Blurred Mongoose: "This spell can't be countered. Shroud".
    let mut t = TestGame::new(2);
    let mongoose = cast_new(&mut t, P0, "Blurred Mongoose", &[]);
    let counterspell = cast_new(&mut t, P1, "Counterspell", &[Entity::Object(mongoose)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Counterspell"));
    assert_eq!(t.zone(counterspell), mtg_engine::object::Zone::Graveyard(P1));
    assert_eq!(t.zone(mongoose), mtg_engine::object::Zone::Stack);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Blurred Mongoose").len(), 1);
    // Obliterate: "This spell can't be countered. Destroy all artifacts, creatures, and
    // lands."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let obliterate = cast_new(&mut t, P0, "Obliterate", &[]);
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(obliterate)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Counterspell"));
    assert!(t.in_graveyard(P0, "Obliterate"));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn a_counterspell_targeting_an_uncounterable_creature_spell_still_does_the_rest() {
    cr!("101.2", "608.2c", "701.6a");
    ruling!(
        "Rhythm of the Wild",
        "A spell or ability that counters spells can still target a creature spell you control. When that spell or ability resolves, the creature spell won't be countered, but any additional effects of that spell or ability will still happen."
    );
    supported("Rhythm of the Wild");
    supported("Surrak Dragonclaw");
    supported("Exclude");
    // "Creature spells you control can't be countered." Exclude: "Counter target
    // creature spell. Draw a card."
    for permanent in ["Rhythm of the Wild", "Surrak Dragonclaw"] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, permanent);
        let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
        let hand = t.hand_size(P1);
        cast_new(&mut t, P1, "Exclude", &[Entity::Object(bears)]);
        t.resolve();
        assert_eq!(t.hand_size(P1), hand + 1, "{permanent}: P1 drew a card");
        assert_eq!(t.zone(bears), mtg_engine::object::Zone::Stack);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    }
}

#[test]
fn a_countered_modal_ability_still_used_up_its_mode() {
    cr!("700.2b", "701.6a");
    ruling!(
        "Demonic Pact",
        "If the ability doesn't resolve (either for having its target become illegal or because a spell or ability counters it), the mode chosen for that instance of the ability still counts as being chosen."
    );
    supported("Demonic Pact");
    supported("Disallow");
    // "At the beginning of your upkeep, choose one that hasn't been chosen — ... • Draw
    // two cards. ..." P0 chooses "Draw two cards"; P1 counters the ability.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Demonic Pact");
    choose_modes(&mut t, P0, &[2]);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let pact = top_of_stack(&t);
    assert_eq!(chosen_modes(&t, pact), vec![2]);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P1, "Disallow", &[Entity::Object(pact)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "the ability was countered");
    // Next upkeep: "Draw two cards" can't be chosen again (P0 asks for it, and gets
    // another mode).
    choose_modes(&mut t, P0, &[2]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    next_upkeep(&mut t, P0);
    let pact = top_of_stack(&t);
    let modes = chosen_modes(&t, pact);
    assert_eq!(modes.len(), 1);
    assert_ne!(modes, vec![2]);
}

#[test]
fn a_countered_next_end_step_trigger_doesnt_trigger_again() {
    cr!("603.7b", "701.6a");
    ruling!(
        "Disallow",
        "If you counter a delayed triggered ability that triggers at the beginning of the \"next\" occurrence of a specified step or phase, that ability won't trigger again the following time that phase or step occurs."
    );
    supported("Sneak Attack");
    // Sneak Attack: "{R}: You may put a creature card from your hand onto the
    // battlefield. That creature gains haste. Sacrifice the creature at the beginning of
    // the next end step."
    let mut t = TestGame::new(2);
    let sneak = t.battlefield(P0, "Sneak Attack");
    t.lands(P0, "Mountain", 1);
    let bears = t.hand(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, sneak, 0, &[]).unwrap();
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    t.advance_to(P0, Step::End);
    t.settle();
    let delayed = top_of_stack(&t);
    assert!(matches!(
        t.g.obj(delayed).stack.as_deref().map(|s| &s.kind),
        Some(StackKind::Triggered { .. })
    ));
    cast_new(&mut t, P1, "Disallow", &[Entity::Object(delayed)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // The next end steps (P1's, then P0's) pass without the ability triggering.
    let from = t.asked().len();
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(bears));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
}
