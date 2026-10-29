//! Rulings batch S09 — heroic ("Heroic — Whenever you cast a spell that targets this
//! creature, ..."): a triggered ability of casting a spell (CR 601.2i, 603.2).

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The heroic triggers on the stack.
fn heroic_triggers(t: &TestGame) -> usize {
    triggers_on_stack(t, "cast a spell that targets")
}

#[test]
fn heroic_triggers_once_per_spell_even_if_it_targets_the_creature_several_times() {
    cr!("603.2", "115.3");
    ruling!(
        "Hero of Iroas",
        "Heroic abilities will trigger only once per spell, even if that spell targets the creature with the heroic ability multiple times."
    );
    supported("Hero of Iroas");
    supported("Seeds of Strength");
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of Iroas");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 1);
    let seeds = t.hand(P0, "Seeds of Strength");
    // Each of the three "target creature" is the Hero.
    t.cast(P0, seeds)
        .target(hero)
        .target(hero)
        .target(hero)
        .go();
    t.settle();
    assert_eq!(heroic_triggers(&t), 1);
    t.resolve_all();
    // One +1/+1 counter, and +3/+3 from the spell: 2/2 → 6/6.
    assert_eq!(t.counters(hero, types::counters::PLUS1), 1);
    assert_eq!(t.pt(hero), (6, 6));
}

/// P0 casts Giant Growth on the Grizzly Bears, then copies it with Twincast choosing the
/// heroic creature as the copy's new target; P0 then changes the original's target to it
/// with Swerve. Returns the number of heroic triggers put on the stack.
fn copy_and_redirect_onto(t: &mut TestGame, heroic: ObjectId) -> usize {
    supported("Twincast");
    supported("Swerve");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 3);
    t.lands(P0, "Mountain", 1);
    let growth = t.hand(P0, "Giant Growth");
    let spell = t.cast(P0, growth).target(bears).go();
    t.settle();
    assert_eq!(heroic_triggers(t), 0);
    // Twincast: "Copy target instant or sorcery spell. You may choose new targets for
    // the copy."
    let twincast = t.hand(P0, "Twincast");
    t.cast(P0, twincast).target(spell).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(heroic)]);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    assert_eq!(
        t.g.obj(copy).stack.as_ref().unwrap().chosen[0].targets[0],
        vec![Entity::Object(heroic)]
    );
    let after_copy = heroic_triggers(t);
    // The copy resolves.
    t.resolve();
    // Swerve: "Change the target of target spell with a single target."
    let swerve = t.hand(P0, "Swerve");
    t.cast(P0, swerve).target(spell).go();
    t.answer_targets(P0, &[Entity::Object(heroic)]);
    t.resolve();
    assert_eq!(
        t.g.obj(spell).stack.as_ref().unwrap().chosen[0].targets[0],
        vec![Entity::Object(heroic)]
    );
    let after_change = heroic_triggers(t);
    t.resolve_all();
    after_copy + after_change
}

#[test]
fn heroic_doesnt_trigger_for_copies_or_changed_targets() {
    cr!("707.10", "115.7", "601.2i");
    ruling!(
        "Akroan Crusader",
        "Heroic abilities won't trigger when a copy of a spell is created on the stack or when a spell's targets are changed to include a creature with a heroic ability."
    );
    supported("Akroan Crusader");
    let mut t = TestGame::new(2);
    let crusader = t.battlefield(P0, "Akroan Crusader");
    assert_eq!(copy_and_redirect_onto(&mut t, crusader), 0);
    // No Soldier token; the Crusader got +3/+3 twice.
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(t.pt(crusader), (7, 7));
}

#[test]
fn heroic_doesnt_trigger_for_copies_or_changed_targets_vanguard_of_brimaz() {
    cr!("707.10", "115.7");
    ruling!(
        "Vanguard of Brimaz",
        "Heroic abilities won’t trigger when a copy of a spell is created on the stack or when a spell’s targets are changed to include a creature with a heroic ability."
    );
    supported("Vanguard of Brimaz");
    let mut t = TestGame::new(2);
    let vanguard = t.battlefield(P0, "Vanguard of Brimaz");
    assert_eq!(copy_and_redirect_onto(&mut t, vanguard), 0);
    assert!(with_subtype(&t, P0, "Cat").len() == 1);
    // Casting a spell that targets it does trigger it.
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(vanguard).go();
    t.settle();
    assert_eq!(heroic_triggers(&t), 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Cat").len(), 2);
}
