//! Rulings batch S26 — Ivy, Gleeful Spellthief ("Whenever a player casts a spell that
//! targets only a single creature other than Ivy, you may copy that spell. The copy
//! targets Ivy."): a copy with a specified target (CR 707.10e), made from the spell as it
//! last existed if it has left the stack (CR 608.2h).

use crate::r_s01_common::supported;
use crate::r_s26_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn ivy_copies_a_spell_even_if_it_was_countered() {
    cr!("707.10e", "608.2h", "603.2");
    ruling!(
        "Ivy, Gleeful Spellthief",
        "The copy is created even if the spell that caused the ability to trigger has been countered by the time the ability resolves. The copy resolves before the original spell."
    );
    supported("Ivy, Gleeful Spellthief");
    let mut t = TestGame::new(2);
    let ivy = t.battlefield(P0, "Ivy, Gleeful Spellthief");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 1);
    t.lands(P0, "Island", 2);
    // P1's Giant Growth on their Bears: Ivy triggers.
    let growth = t.hand(P1, "Giant Growth");
    let growth = t.cast(P1, growth).target(bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P0 counters the Giant Growth in response; Ivy's copy is still made.
    let cancel = t.hand(P0, "Counterspell");
    t.cast(P0, cancel).target(growth).go();
    t.resolve();
    assert!(!t.g.stack.contains(&growth));
    t.answer_yes(P0, true);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).controller, P0);
    assert_eq!(targets_on_stack(&t, copies[0]), vec![Entity::Object(ivy)]);
    t.resolve_all();
    // Ivy is a 2/1: Giant Growth's copy made it 5/4.
    assert_eq!(t.pt(ivy), (5, 4));
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn ivys_copy_resolves_before_the_original() {
    cr!("707.10e", "405.5");
    ruling!(
        "Ivy, Gleeful Spellthief",
        "The copy resolves before the original spell."
    );
    let mut t = TestGame::new(2);
    let ivy = t.battlefield(P0, "Ivy, Gleeful Spellthief");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Mountain", 1);
    // P1 Shocks their own Bears; P0's copy deals 2 damage to Ivy (a 2/1) first.
    let shock = t.hand(P1, "Shock");
    let shock = t.cast(P1, shock).target(bears).go();
    t.answer_yes(P0, true);
    t.settle();
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(*t.g.stack.last().unwrap(), copies[0]);
    t.resolve();
    assert!(!t.g.is_live(ivy));
    assert!(t.g.stack.contains(&shock));
    assert!(t.on_battlefield(bears));
    t.resolve_all();
    assert!(!t.g.is_live(bears));
}
