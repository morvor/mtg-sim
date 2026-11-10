//! Rulings batch P049 — committing a crime (CR 700.13): one crime per spell or ability
//! however many targets it has, decided by the targets chosen as it's put on the stack
//! (changing them changes nothing), and its triggers go on the stack along with the
//! other triggers of casting the spell, above the spell (CR 603.3b).

use crate::r_p049_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_spell_with_several_targets_is_one_crime() {
    cr!("700.13", "603.2c");
    ruling!(
        "Patrolling Peacemaker",
        "A player can commit only one crime per spell or ability they control. Targeting multiple opponents, permanents, spells, abilities, and/or cards with the same spell or ability doesn’t constitute committing multiple crimes."
    );
    supported("Patrolling Peacemaker");
    supported("Arc Trail");
    let mut t = TestGame::new(2);
    // "Whenever an opponent commits a crime, proliferate."
    let pk = t.enter(P1, "Patrolling Peacemaker");
    t.settle();
    assert_eq!(t.counters(pk, counters::PLUS1), 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    // Arc Trail: 2 damage to one target and 1 damage to another: two of P1's creatures.
    cast_new(&mut t, P0, "Arc Trail", &[obj(a), obj(b)]);
    t.settle();
    assert_eq!(crimes(&t, P0), 1);
    assert_eq!(triggers_on_stack(&t, "proliferate"), 1);
}

#[test]
fn changing_a_spells_targets_doesnt_change_whether_a_crime_was_committed() {
    cr!("700.13", "115.7");
    ruling!(
        "Raven of Fell Omens",
        "Changing the target or targets of a spell or ability won’t affect whether or not the controller of that spell or ability has committed a crime. Only the initial targets chosen for that spell or ability are used to determine whether or not its controller committed a crime."
    );
    supported("Raven of Fell Omens");
    supported("Redirect");
    // Shock at P0's own creature: no crime. P1 redirects it to P1's creature: still none.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raven of Fell Omens");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    let shock = cast_new(&mut t, P0, "Shock", &[obj(mine)]);
    t.settle();
    assert_eq!(crimes(&t, P0), 0);
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 0);
    cast_new(&mut t, P1, "Redirect", &[obj(shock)]);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[obj(theirs)]);
    t.resolve();
    assert_eq!(t.obj(shock).stack.as_ref().unwrap().chosen[0].targets[0], vec![obj(theirs)]);
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).damage, 2);
    assert_eq!(crimes(&t, P0), 0);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Shock at P1's creature: a crime. Redirected to P0's own creature: it stays one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raven of Fell Omens");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    let shock = cast_new(&mut t, P0, "Shock", &[obj(theirs)]);
    t.settle();
    assert_eq!(crimes(&t, P0), 1);
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 1);
    cast_new(&mut t, P1, "Redirect", &[obj(shock)]);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[obj(mine)]);
    t.resolve_all();
    assert_eq!(t.obj_now(mine).damage, 2);
    assert_eq!(crimes(&t, P0), 1);
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn crime_triggers_go_on_the_stack_with_the_other_cast_triggers_above_the_spell() {
    cr!("700.13", "603.3b", "603.3");
    ruling!(
        "Raven of Fell Omens",
        "an ability that triggers when you cast a spell that targets an opponent will trigger at the same time as an ability that triggers whenever you commit a crime. Those abilities can be put on the stack in either order (if you control them both), and they’ll both resolve before the spell that caused them to trigger."
    );
    supported("Raven of Fell Omens");
    supported("Guttersnipe");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raven of Fell Omens");
    // "Whenever you cast an instant or sorcery spell, this creature deals 2 damage to each
    // opponent."
    t.battlefield(P0, "Guttersnipe");
    let shock = cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    // P0 orders the two: the Raven's first (it resolves last).
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![0, 1]));
    t.settle();
    let st = stack(&t);
    assert_eq!(st.len(), 3);
    assert_eq!(st[0], shock);
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 1);
    assert_eq!(triggers_on_stack(&t, "2 damage"), 1);
    // Both resolve before Shock.
    t.resolve();
    t.resolve();
    assert_eq!(stack(&t), vec![shock]);
    assert_eq!((t.life(P0), t.life(P1)), (21, 17));
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}
