//! Rulings batch S26 — Aboleth Spawn ("Whenever a creature entering under an opponent's
//! control causes a triggered ability of that creature to trigger, you may copy that
//! ability. You may choose new targets for the copy."): a trigger on another ability
//! triggering (CR 603.3b) whose copy keeps the mode (CR 707.10, 700.2g).

use crate::r_s01_common::supported;
use crate::r_s25_common::abilities_from;
use crate::r_s26_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

fn aboleth(t: &mut TestGame) -> ObjectId {
    supported("Aboleth Spawn");
    t.battlefield(P0, "Aboleth Spawn")
}

#[test]
fn a_copied_modal_enters_ability_keeps_its_mode() {
    cr!("603.3b", "707.10", "700.2g");
    ruling!(
        "Aboleth Spawn",
        "If the ability is modal (that is, it has a bulleted list of choices), the copy will have the same mode(s). You can't choose new ones."
    );
    supported("Charming Prince");
    // Charming Prince: "When this creature enters, choose one — • Scry 2. • You gain 3
    // life. • Exile another target creature you own. ..."
    let mut t = TestGame::new(2);
    aboleth(&mut t);
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    let prince = t.enter(P1, "Charming Prince");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P0 copies the Prince's ability; a different mode would be offered if asked.
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.resolve();
    let copies = abilities_from(&t, prince);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().all(|c| modes_on_stack(&t, *c) == vec![1]));
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(t.obj(copy).controller, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 23);
}

#[test]
fn only_abilities_of_the_entering_creature_are_copied() {
    cr!("603.3b", "603.6a");
    ruling!(
        "Aboleth Spawn",
        "Aboleth Spawn's ability cares only about triggered abilities of the creature that's entering, not abilities of other permanents that trigger when that creature enters the battlefield."
    );
    supported("Soul Warden");
    supported("Elvish Visionary");
    // Soul Warden: "Whenever another creature enters, you gain 1 life." Elvish Visionary:
    // "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let spawn = aboleth(&mut t);
    t.battlefield(P1, "Soul Warden");
    t.enter(P1, "Elvish Visionary");
    t.settle();
    // The Warden's and the Visionary's abilities, and one Aboleth Spawn trigger.
    assert_eq!(t.stack_len(), 3);
    assert_eq!(abilities_from(&t, spawn).len(), 1);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // A creature entering under P0's own control doesn't trigger it (only the Visionary's
    // and the Warden's abilities trigger).
    t.enter(P0, "Elvish Visionary");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(abilities_from(&t, spawn).is_empty());
}

#[test]
fn the_copy_goes_on_top_of_the_ability_and_resolves_first() {
    cr!("603.3b", "707.10", "405.5");
    ruling!(
        "Aboleth Spawn",
        "Aboleth Spawn's ability will always go on the stack on top of the entering creature's triggered ability that caused it to trigger. The copy it creates will be created on the stack on top of the entering creature's triggered ability and the copy will resolve before that ability."
    );
    let mut t = TestGame::new(2);
    let spawn = aboleth(&mut t);
    let visionary = t.enter(P1, "Elvish Visionary");
    t.settle();
    let original = abilities_from(&t, visionary);
    assert_eq!(original.len(), 1);
    assert_eq!(t.g.stack[0], original[0]);
    assert_eq!(abilities_from(&t, spawn), vec![t.g.stack[1]]);
    let (hand0, hand1) = (t.hand_size(P0), t.hand_size(P1));
    t.answer_yes(P0, true);
    t.resolve();
    // The copy is on top of the original, and resolves while the original waits.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.g.stack[0], original[0]);
    assert_eq!(t.obj(t.g.stack[1]).controller, P0);
    t.resolve();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (hand0 + 1, hand1));
    assert_eq!(t.g.stack, original);
    t.resolve();
    assert_eq!(t.hand_size(P1), hand1 + 1);
}

#[test]
fn entering_tapped_isnt_a_triggered_ability() {
    cr!("603.3b", "614.1d");
    ruling!(
        "Aboleth Spawn",
        "Effects that modify how a creature enters the battlefield are not triggered abilities and are not affected by Aboleth Spawn's ability."
    );
    supported("Shambling Ghoul");
    let mut t = TestGame::new(2);
    aboleth(&mut t);
    // Shambling Ghoul: "This creature enters tapped."
    let ghoul = t.enter(P1, "Shambling Ghoul");
    t.settle();
    assert!(t.obj_now(ghoul).tapped);
    assert_eq!(t.stack_len(), 0);
}
