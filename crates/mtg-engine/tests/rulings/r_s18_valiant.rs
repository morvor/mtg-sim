//! Rulings batch S18 — valiant: "Valiant — Whenever this creature becomes the target of
//! a spell or ability you control for the first time each turn, [effect]."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Valiant triggers on the stack.
fn valiant_on_stack(t: &TestGame) -> usize {
    triggers_on_stack(t, "for the first time each turn")
}

#[test]
fn valiant_triggers_when_a_target_is_changed_to_the_creature() {
    cr!("115.7", "603.2");
    ruling!(
        "Heartfire Hero",
        "If a spell or ability you control has one or more of its targets changed to a creature you control with a valiant ability, that ability will trigger if that creature hasn't yet been the target of a spell or ability you control this turn."
    );
    supported("Heartfire Hero");
    supported("Swerve");
    // P0's Giant Growth targets the Bears; Swerve changes its target to Heartfire Hero.
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Heartfire Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Giant Growth");
    give_mana_for(&mut t, P0, "Swerve");
    let growth = t.hand(P0, "Giant Growth");
    let gg = t.cast(P0, growth).target(bears).go();
    let swerve = t.hand(P0, "Swerve");
    t.cast(P0, swerve).target(gg).go();
    t.answer_targets(P0, &[Entity::Object(hero)]);
    t.resolve();
    assert_eq!(valiant_on_stack(&t), 1);
    t.resolve_all();
    assert_eq!(t.counters(hero, counters::PLUS1), 1);
    assert_eq!(t.pt(hero), (5, 5));
    assert_eq!(t.pt(bears), (2, 2));

    // Already the target of P0's spell this turn: changing another target to it doesn't
    // trigger valiant again.
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Heartfire Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Giant Growth");
    let first = t.hand(P0, "Giant Growth");
    t.cast(P0, first).target(hero).go();
    t.resolve_all();
    assert_eq!(t.counters(hero, counters::PLUS1), 1);
    give_mana_for(&mut t, P0, "Giant Growth");
    give_mana_for(&mut t, P0, "Swerve");
    let growth = t.hand(P0, "Giant Growth");
    let gg = t.cast(P0, growth).target(bears).go();
    let swerve = t.hand(P0, "Swerve");
    t.cast(P0, swerve).target(gg).go();
    t.answer_targets(P0, &[Entity::Object(hero)]);
    t.resolve();
    assert_eq!(valiant_on_stack(&t), 0);
    t.resolve_all();
    assert_eq!(t.counters(hero, counters::PLUS1), 1);
    assert_eq!(t.pt(hero), (8, 8));

    // An opponent's spell changed to target it isn't a spell P0 controls.
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Heartfire Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P1, "Giant Growth");
    let growth = t.hand(P1, "Giant Growth");
    let gg = t.cast(P1, growth).target(bears).go();
    give_mana_for(&mut t, P0, "Swerve");
    let swerve = t.hand(P0, "Swerve");
    t.cast(P0, swerve).target(gg).go();
    t.answer_targets(P0, &[Entity::Object(hero)]);
    t.resolve();
    assert_eq!(valiant_on_stack(&t), 0);
    t.resolve_all();
    assert_eq!(t.counters(hero, counters::PLUS1), 0);
    assert_eq!(t.pt(hero), (4, 4));
}

#[test]
fn valiant_triggers_when_a_copy_of_a_spell_targets_the_creature() {
    cr!("707.10c", "603.2");
    ruling!(
        "Heartfire Hero",
        "If you create a copy of a spell on the stack and target a creature you control with a valiant ability, that ability will trigger as long as the creature hasn't yet been the target of a spell or ability you control this turn."
    );
    supported("Twincast");
    // Twincast copies P0's Giant Growth (targeting the Bears), and the copy targets the
    // Hero instead.
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Heartfire Hero");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Giant Growth");
    give_mana_for(&mut t, P0, "Twincast");
    let growth = t.hand(P0, "Giant Growth");
    let gg = t.cast(P0, growth).target(bears).go();
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(gg).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(hero)]);
    t.resolve();
    assert_eq!(valiant_on_stack(&t), 1);
    t.resolve_all();
    assert_eq!(t.counters(hero, counters::PLUS1), 1);
    assert_eq!(t.pt(hero), (5, 5));
    assert_eq!(t.pt(bears), (5, 5));
    // A copy targeting it after it was already targeted this turn doesn't trigger.
    give_mana_for(&mut t, P0, "Giant Growth");
    give_mana_for(&mut t, P0, "Twincast");
    let growth = t.hand(P0, "Giant Growth");
    let gg = t.cast(P0, growth).target(bears).go();
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(gg).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(hero)]);
    t.resolve();
    assert_eq!(valiant_on_stack(&t), 0);
    t.resolve_all();
    assert_eq!(t.counters(hero, counters::PLUS1), 1);
}

#[test]
fn valiant_resolves_before_the_spell_that_caused_it_to_trigger() {
    cr!("603.3", "405.2");
    ruling!(
        "Nettle Guard",
        "Valiant abilities will resolve before the spell or ability that caused them to trigger."
    );
    supported("Nettle Guard");
    // P0 targets their own Nettle Guard (3/1) with Shock: valiant ("it gets +0/+2 until
    // end of turn") resolves first, so the 3/3 Guard survives the 2 damage.
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P0, "Nettle Guard");
    give_mana_for(&mut t, P0, "Shock");
    let shock = t.hand(P0, "Shock");
    let spell = t.cast(P0, shock).target(guard).go();
    t.settle();
    assert_eq!(t.g.stack.len(), 2);
    assert_eq!(t.g.stack[0], spell);
    assert_eq!(valiant_on_stack(&t), 1);
    t.resolve();
    assert_eq!(t.pt(guard), (3, 3));
    assert_eq!(t.g.stack, vec![spell]);
    t.resolve_all();
    assert!(t.on_battlefield(guard));
    assert_eq!(t.obj_now(guard).damage, 2);
}
