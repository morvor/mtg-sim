//! Rulings batch S25 — abilities that copy an ability as it's activated (Ertha Jo,
//! Frontier Mentor: "Whenever you activate an ability that targets a creature or player,
//! copy that ability. You may choose new targets for the copy."). The copy isn't
//! activated (CR 707.10, 602.2), keeps the targets unless new ones are chosen (CR
//! 707.10c), and choices made on resolution are made separately for it (CR 608.2).

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s11_common::triggered_from;
use crate::r_s25_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 controls Ertha Jo and Cephalid Looter ("{T}: Target player draws a card, then
/// discards a card."), with an Island and a Swamp in hand and two Forests on top of the
/// library. Returns (Ertha Jo, the Looter).
fn ertha_and_looter(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Ertha Jo, Frontier Mentor");
    supported("Cephalid Looter");
    let ertha = t.battlefield(P0, "Ertha Jo, Frontier Mentor");
    let looter = t.battlefield(P0, "Cephalid Looter");
    t.hand(P0, "Island");
    t.hand(P0, "Swamp");
    t.library_top(P0, "Forest");
    t.library_top(P0, "Forest");
    (ertha, looter)
}

#[test]
fn choices_on_resolution_are_made_separately_for_ertha_jo_s_copy() {
    cr!("707.10", "608.2", "602.2");
    ruling!(
        "Ertha Jo, Frontier Mentor",
        "Any choices made when the ability resolves won't have been made yet when it's copied. Any such choices will be made separately when the copy resolves."
    );
    ruling!(
        "Ertha Jo, Frontier Mentor",
        "The copy made by Ertha Jo's triggered ability is created on the stack, so it's not \"activated.\""
    );
    let mut t = TestGame::new(2);
    let (ertha, looter) = ertha_and_looter(&mut t);
    let island = t.g.find_in_zone(mtg_engine::object::Zone::Hand(P0), "Island")[0];
    let swamp = t.g.find_in_zone(mtg_engine::object::Zone::Hand(P0), "Swamp")[0];
    t.answer_targets(P0, &[Entity::Player(P0)]);
    activate_containing(&mut t, P0, looter, "Target player").unwrap();
    t.settle();
    assert_eq!(triggered_from(&t, ertha), 1);
    let from = t.asked().len();
    keep_copy_targets(&mut t, P0);
    // The copy resolves first: discard the Island; then the original: the Swamp.
    t.answer_choose(P0, &[Entity::Object(island)]);
    t.answer_choose(P0, &[Entity::Object(swamp)]);
    t.resolve_all();
    let discards = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .count();
    assert_eq!(discards, 2);
    assert!(t.in_graveyard(P0, "Island"));
    assert!(t.in_graveyard(P0, "Swamp"));
    assert_eq!(t.g.find_in_zone(mtg_engine::object::Zone::Hand(P0), "Forest").len(), 2);
    // The copy wasn't activated: Ertha Jo triggered only once.
    assert_eq!(triggered_from(&t, ertha), 1);
}

#[test]
fn ertha_jo_s_copy_may_get_a_new_target() {
    cr!("707.10", "707.10c", "115.7");
    ruling!(
        "Ertha Jo, Frontier Mentor",
        "The copy will have the same targets as the ability it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    let mut t = TestGame::new(2);
    let (_, looter) = ertha_and_looter(&mut t);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.answer_targets(P0, &[Entity::Player(P0)]);
    activate_containing(&mut t, P0, looter, "Target player").unwrap();
    t.settle();
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, copy), vec![Entity::Player(P1)]);
    t.resolve_all();
    // Each player drew and discarded a card.
    assert_eq!(t.hand_size(P0), h0);
    assert_eq!(t.hand_size(P1), h1);
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.graveyard_size(P1), 1);
}

#[test]
fn ertha_jo_ignores_abilities_that_dont_target_a_creature_or_player() {
    cr!("602.2", "115.9b");
    // Merfolk Looter: "{T}: Draw a card, then discard a card." (no target); Cephalid
    // Looter's ability targets a player.
    let mut t = TestGame::new(2);
    let (ertha, _) = ertha_and_looter(&mut t);
    let merfolk = t.battlefield(P0, "Merfolk Looter");
    activate_containing(&mut t, P0, merfolk, "Draw a card").unwrap();
    t.settle();
    assert_eq!(triggered_from(&t, ertha), 0);
    t.resolve_all();
}

/// P0's Bill Potts ("Whenever you cast an instant or sorcery spell that targets only Bill
/// Potts or activate an ability that targets only Bill Potts, copy that spell or ability.
/// You may choose new targets for the copy. This ability triggers only once each turn.")
/// and Grizzly Bears. Returns (Bill, Bears).
fn bill_and_bears(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Bill Potts");
    let bill = t.battlefield(P0, "Bill Potts");
    let bears = t.battlefield(P0, "Grizzly Bears");
    (bill, bears)
}

#[test]
fn bill_potts_s_copy_may_get_new_legal_targets() {
    cr!("707.10", "707.10c", "115.9c");
    ruling!(
        "Bill Potts",
        "The copy will have the same targets as the spell or ability it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. The new targets must be legal."
    );
    let mut t = TestGame::new(2);
    let (bill, bears) = bill_and_bears(&mut t);
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(bill)]);
    t.settle();
    assert_eq!(triggered_from(&t, bill), 1);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(bears))]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(bill), (5, 7));
    // It triggers only once each turn.
    cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(bill)]);
    t.settle();
    assert_eq!(triggered_from(&t, bill), 1);
}

#[test]
fn bill_potts_s_copy_of_a_modal_spell_has_the_same_mode() {
    cr!("707.10", "700.2");
    ruling!(
        "Bill Potts",
        "If the spell or ability that's copied is modal (that is, it says \"Choose one —\" or the like), the copy will have the same mode or modes. You can't choose different ones."
    );
    supported("Boros Charm");
    // Boros Charm's third mode: "Target creature gains double strike until end of turn."
    let mut t = TestGame::new(2);
    let (bill, bears) = bill_and_bears(&mut t);
    lands_for_cost(&mut t, P0, "Boros Charm");
    let card = t.hand(P0, "Boros Charm");
    let charm = t.cast(P0, card).modes(&[2]).target(bill).go();
    t.settle();
    let from = t.asked().len();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(bears))]);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(
        crate::r_s07_common::chosen_modes(&t, copy),
        crate::r_s07_common::chosen_modes(&t, charm)
    );
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
    t.resolve_all();
    assert!(t
        .obj_now(bears)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
    assert!(t
        .obj_now(bill)
        .has_keyword(mtg_engine::keywords::KeywordKind::DoubleStrike));
}

#[test]
fn bill_potts_copies_an_activated_ability_that_targets_only_it() {
    cr!("707.10", "115.9c", "602.2");
    // Prodigal Pyromancer: "{T}: This creature deals 1 damage to any target."
    let mut t = TestGame::new(2);
    let (bill, _) = bill_and_bears(&mut t);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.answer_targets(P0, &[Entity::Object(bill)]);
    activate_containing(&mut t, P0, pyro, "damage").unwrap();
    t.settle();
    assert_eq!(triggered_from(&t, bill), 1);
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.obj_now(bill).damage, 1);
}
