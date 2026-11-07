//! Rulings batch S26 — copies of loyalty abilities (CR 707.10, 606): Chandra's Regulator
//! ("Whenever you activate a loyalty ability of a Chandra planeswalker, you may pay {1}.
//! If you do, copy that ability."). The loyalty cost was paid for the original; the copy
//! has no cost to pay (CR 707.10, 606.4).

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::abilities_from;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn copying_a_loyalty_ability_doesnt_change_loyalty() {
    cr!("707.10", "606.4", "602.2b");
    ruling!(
        "Chandra's Regulator",
        "Copying a loyalty ability doesn’t add or remove loyalty counters from any object."
    );
    supported("Chandra's Regulator");
    supported("Chandra Nalaar");
    // Chandra Nalaar: "+1: Chandra Nalaar deals 1 damage to target player or
    // planeswalker."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chandra's Regulator");
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    let loyalty = t.counters(chandra, counters::LOYALTY);
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, chandra, "deals 1 damage").expect("+1");
    assert_eq!(t.counters(chandra, counters::LOYALTY), loyalty + 1);
    t.settle();
    // Regulator: pay {1}, copy it (keeping its target).
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(abilities_from(&t, chandra).len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.counters(chandra, counters::LOYALTY), loyalty + 1);
}

#[test]
fn chandras_regulator_needs_a_loyalty_ability_of_a_chandra() {
    cr!("606.3", "602.2");
    supported("Chandra's Regulator");
    // Not a Chandra: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Chandra's Regulator");
    let other = t.battlefield(P0, "Prodigal Pyromancer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, other, "damage").expect("ping");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // A Chandra's loyalty ability: a trigger.
    let chandra = t.battlefield(P0, "Chandra Nalaar");
    t.resolve_all();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, chandra, "deals 1 damage").expect("+1");
    t.settle();
    assert_eq!(t.stack_len(), 2);
}

#[test]
fn chandras_regulator_discards_a_mountain_or_a_red_card() {
    cr!("118.3", "602.2b");
    supported("Chandra's Regulator");
    // "{1}, {T}, Discard a Mountain card or a red card: Draw a card."
    let mut t = TestGame::new(2);
    let regulator = t.battlefield(P0, "Chandra's Regulator");
    t.lands(P0, "Wastes", 1);
    t.hand(P0, "Grizzly Bears");
    assert!(activate_containing(&mut t, P0, regulator, "Draw a card").is_err());
    let mountain = t.hand(P0, "Mountain");
    t.answer_choose(P0, &[Entity::Object(mountain)]);
    activate_containing(&mut t, P0, regulator, "Draw a card").expect("activate");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mountain"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}
