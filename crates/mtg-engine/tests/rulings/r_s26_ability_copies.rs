//! Rulings batch S26 — copies of activated and triggered abilities (CR 707.10) made by
//! Adric, Mathematical Genius ("{2}{U}, {T}: Copy target activated or triggered ability you
//! control. You may choose new targets for the copy."): the copy isn't activated, resolves
//! first, keeps the mode, and its resolution choices and costs are its own.

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{abilities_from, change_copy_targets, keep_copy_targets};
use crate::r_s26_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's Adric (with mana for its ability) copies the ability `target` on the stack.
fn adric_copies(t: &mut TestGame, adric: ObjectId, target: ObjectId) {
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[Entity::Object(target)]);
    activate_containing(t, P0, adric, "Copy target").expect("Adric");
}

#[test]
fn a_copied_trigger_asks_for_its_own_payment() {
    cr!("707.10", "608.2", "603.5");
    ruling!(
        "Adric, Mathematical Genius",
        "Most notably, if a triggered ability asks its controller to pay a cost, you pay that cost for the copy if you wish to have it paid."
    );
    supported("Adric, Mathematical Genius");
    supported("Unassuming Sage");
    // Unassuming Sage: "When this creature enters, you may pay {2}. If you do, create a
    // Sorcerer Role token attached to it."
    let mut t = TestGame::new(2);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    let sage = t.enter(P0, "Unassuming Sage");
    t.settle();
    let trigger = *t.g.stack.last().unwrap();
    adric_copies(&mut t, adric, trigger);
    t.resolve();
    assert_eq!(abilities_from(&t, sage).len(), 2);
    // The copy: pay {2}. The original: don't (though it could).
    t.lands(P0, "Wastes", 4);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    let pays = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(pays, 2);
    let roles: Vec<ObjectId> = t.g.attachments_of(Entity::Object(t.g.current(sage)));
    assert_eq!(roles.len(), 1);
    assert_eq!(untapped(&t, P0), 2);
}

/// The untapped lands `p` controls.
fn untapped(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(mtg_engine::types::CardType::Land) && !o.tapped)
        .count()
}

#[test]
fn a_copied_ability_isnt_activated() {
    cr!("707.10", "602.2", "603.2");
    ruling!(
        "Adric, Mathematical Genius",
        "The copy is created on the stack, so it's not \"activated.\" Creating the copy won't cause abilities that trigger when a player activates an ability to trigger."
    );
    supported("Ertha Jo, Frontier Mentor");
    // Ertha Jo: "Whenever you activate an ability that targets a creature or player, copy
    // that ability. You may choose new targets for the copy." Prodigal Pyromancer: "{T}:
    // This creature deals 1 damage to any target."
    let mut t = TestGame::new(2);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    let ertha = t.battlefield(P0, "Ertha Jo, Frontier Mentor");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ping = activate_containing(&mut t, P0, pyro, "damage")
        .unwrap()
        .unwrap();
    t.settle();
    assert_eq!(abilities_from(&t, ertha).len(), 1);
    // Adric copies the Pyromancer's ability: Ertha Jo doesn't trigger again.
    adric_copies(&mut t, adric, ping);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    t.settle();
    assert_eq!(abilities_from(&t, pyro).len(), 2);
    assert_eq!(abilities_from(&t, ertha).len(), 1);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    // The original, Adric's copy and Ertha Jo's copy.
    assert_eq!(t.life(P1), 17);
}

#[test]
fn adrics_copy_resolves_before_the_original() {
    cr!("707.10", "405.5");
    ruling!(
        "Adric, Mathematical Genius",
        "The copy will resolve before the original ability does."
    );
    let mut t = TestGame::new(2);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    let ping = activate_containing(&mut t, P0, pyro, "damage")
        .unwrap()
        .unwrap();
    adric_copies(&mut t, adric, ping);
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    t.resolve();
    // The copy (at P1) resolves first; the original is still waiting.
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(elves));
    assert!(t.g.stack.contains(&ping));
    t.resolve();
    assert!(!t.on_battlefield(elves));
}

#[test]
fn a_copied_modal_trigger_keeps_its_mode() {
    cr!("707.10", "700.2g", "603.3c");
    ruling!(
        "Adric, Mathematical Genius",
        "If the ability that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode. A different mode can't be chosen."
    );
    supported("Charming Prince");
    // Charming Prince: "When this creature enters, choose one — • Scry 2. • You gain 3
    // life. • Exile another target creature you own. Return it ..."
    let mut t = TestGame::new(2);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    let prince = t.enter(P0, "Charming Prince");
    t.settle();
    let trigger = *t.g.stack.last().unwrap();
    assert_eq!(modes_on_stack(&t, trigger), vec![1]);
    adric_copies(&mut t, adric, trigger);
    // A different mode would be offered if asked: it isn't.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    let from = t.asked().len();
    t.resolve();
    let copies = abilities_from(&t, prince);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().all(|c| modes_on_stack(&t, *c) == vec![1]));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}
