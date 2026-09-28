//! Rulings batch S26 — Magus Lucea Kane ("{T}: Add {C}{C}. When you next cast a spell with
//! {X} in its mana cost or activate an ability with {X} in its activation cost this turn,
//! copy that spell or ability. You may choose new targets for the copy."): a once-only
//! delayed trigger (CR 603.7b, 603.7c) whose copy keeps the value of X (CR 707.10), isn't
//! cast or activated, and is made even if the original was countered (CR 608.2h).

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{abilities_from, keep_copy_targets, x_of};
use crate::r_s26_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0's untapped Magus Lucea Kane taps for {C}{C}, setting up its delayed trigger.
fn magus_taps(t: &mut TestGame) -> ObjectId {
    supported("Magus Lucea Kane");
    let magus = t.battlefield(P0, "Magus Lucea Kane");
    activate_containing(t, P0, magus, "Add {C}{C}").expect("Magus's mana ability");
    magus
}

/// P0 casts Blaze ({X}{R}: "Blaze deals X damage to any target.") at `target` with X = 2,
/// paying {X} with the Magus's {C}{C}.
fn blaze(t: &mut TestGame, target: impl Into<Entity>) -> ObjectId {
    t.lands(P0, "Mountain", 1);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(2).target(target).go()
}

#[test]
fn the_copy_of_an_x_spell_has_the_same_x() {
    cr!("707.10", "107.3", "603.7c");
    ruling!("Magus Lucea Kane", "The copy has the same value of X.");
    let mut t = TestGame::new(2);
    magus_taps(&mut t);
    let blaze = blaze(&mut t, P1);
    t.settle();
    // The delayed trigger is above Blaze.
    assert_eq!(t.stack_len(), 2);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(x_of(&t, copies[0]), Some(2));
    assert_eq!(x_of(&t, blaze), Some(2));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn the_copy_of_an_x_ability_has_the_same_x() {
    cr!("707.10", "107.3", "602.2b");
    ruling!("Magus Lucea Kane", "The copy has the same value of X.");
    supported("Oracle of Nectars");
    // Oracle of Nectars: "{X}, {T}: You gain X life."
    let mut t = TestGame::new(2);
    magus_taps(&mut t);
    let oracle = t.battlefield(P0, "Oracle of Nectars");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let gain = activate_containing(&mut t, P0, oracle, "gain X life")
        .unwrap()
        .unwrap();
    t.settle();
    t.resolve();
    let abilities = abilities_from(&t, oracle);
    assert_eq!(abilities.len(), 2);
    assert!(abilities.iter().all(|a| x_of(&t, *a) == Some(2)));
    assert!(abilities.contains(&gain));
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn the_copy_isnt_cast_or_activated() {
    cr!("707.10", "601.2i", "602.2", "603.2");
    ruling!(
        "Magus Lucea Kane",
        "The copy is created on the stack, so it's not \"cast\" or \"activated.\" Creating the copy won't cause abilities that trigger when a player casts a spell or activates an ability to trigger."
    );
    supported("Young Pyromancer");
    supported("Crackdown Construct");
    supported("Oracle of Nectars");
    // Young Pyromancer: "Whenever you cast an instant or sorcery spell, create a 1/1 red
    // Elemental creature token." Crackdown Construct: "Whenever you activate an ability of
    // an artifact or creature that isn't a mana ability, this creature gets +1/+1 until end
    // of turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Young Pyromancer");
    let before: Vec<ObjectId> = t.g.battlefield.clone();
    // A cast spell: its copy doesn't make a second Elemental.
    magus_taps(&mut t);
    blaze(&mut t, P1);
    t.settle();
    // Blaze, the Pyromancer's trigger and the Magus's delayed trigger.
    assert_eq!(t.stack_len(), 3);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(new_tokens(&t, P0, &before).len(), 1);

    // An activated ability: its copy doesn't pump the Construct a second time. (The
    // Magus's own ability is a mana ability, which doesn't trigger it either.)
    let mut t = TestGame::new(2);
    let construct = t.battlefield(P0, "Crackdown Construct");
    let (p, x) = t.pt(construct);
    magus_taps(&mut t);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    let oracle = t.battlefield(P0, "Oracle of Nectars");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    activate_containing(&mut t, P0, oracle, "gain X life").unwrap();
    t.settle();
    // The ability, the Construct's trigger and the Magus's delayed trigger.
    assert_eq!(t.stack_len(), 3);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.pt(construct), (p + 1, x + 1));
}

#[test]
fn the_copy_is_made_even_if_the_original_was_countered() {
    cr!("707.10", "608.2h", "603.7c");
    ruling!(
        "Magus Lucea Kane",
        "A copy is created even if the spell or ability that caused the delayed triggered ability to trigger has been countered by the time that delayed triggered ability resolves. The copy resolves before the original spell."
    );
    let mut t = TestGame::new(2);
    magus_taps(&mut t);
    let blaze = blaze(&mut t, P1);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters Blaze in response to the delayed trigger.
    t.lands(P1, "Island", 2);
    let cancel = t.hand(P1, "Counterspell");
    t.cast(P1, cancel).target(blaze).go();
    t.resolve();
    assert!(!t.g.stack.contains(&blaze));
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(x_of(&t, copies[0]), Some(2));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn the_delayed_trigger_copies_only_the_next_x_spell() {
    cr!("603.7c", "603.7b");
    let mut t = TestGame::new(2);
    magus_taps(&mut t);
    // A spell without {X} doesn't trigger it (and doesn't use it up).
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The next spell with {X} is copied ...
    blaze(&mut t, P1);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 13);
    // ... and the one after isn't.
    t.lands(P0, "Mountain", 3);
    let blaze2 = t.hand(P0, "Blaze");
    t.cast(P0, blaze2).x(2).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
}
